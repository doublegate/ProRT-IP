//! Large buffer pool for zero-copy packet handling
//!
//! Provides tiered buffer pools optimized for different packet sizes,
//! eliminating heap allocations in hot paths for packets >10KB.
//!
//! # Design
//!
//! Three-tier buffer pool architecture:
//! - **Tier 1 (4KB)**: Small packets (standard MTU, most common)
//! - **Tier 2 (16KB)**: Medium packets (jumbo frames, service probes)
//! - **Tier 3 (64KB)**: Large packets (max IP packet, large responses)
//!
//! # Thread Safety
//!
//! Uses `parking_lot::Mutex` for thread-safe pool access with minimal
//! contention. Thread-local pools can be used for hot paths.
//!
//! # Example
//!
//! ```
//! use prtip_network::large_buffer_pool::{LargeBufferPool, BufferTier};
//!
//! let pool = LargeBufferPool::new();
//!
//! // Acquire buffer for 12KB packet
//! let mut buffer = pool.acquire(12000);
//! assert!(buffer.capacity() >= 12000);
//!
//! // Use buffer...
//! buffer.extend_from_slice(&[0u8; 100]);
//!
//! // Return buffer to pool (automatic via Drop)
//! drop(buffer);
//! ```

use bytes::{Bytes, BytesMut};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Buffer tier sizes (in bytes)
pub const TIER_1_SIZE: usize = 4 * 1024; // 4KB - small packets
pub const TIER_2_SIZE: usize = 16 * 1024; // 16KB - medium packets
pub const TIER_3_SIZE: usize = 64 * 1024; // 64KB - max IP packet

/// Maximum buffers per tier
pub const MAX_BUFFERS_PER_TIER: usize = 64;

/// Buffer tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferTier {
    /// Small packets (<=4KB)
    Small,
    /// Medium packets (4KB-16KB)
    Medium,
    /// Large packets (16KB-64KB)
    Large,
}

impl BufferTier {
    /// Get buffer size for this tier
    pub fn size(&self) -> usize {
        match self {
            BufferTier::Small => TIER_1_SIZE,
            BufferTier::Medium => TIER_2_SIZE,
            BufferTier::Large => TIER_3_SIZE,
        }
    }

    /// Determine appropriate tier for given size
    pub fn for_size(size: usize) -> Self {
        if size <= TIER_1_SIZE {
            BufferTier::Small
        } else if size <= TIER_2_SIZE {
            BufferTier::Medium
        } else {
            BufferTier::Large
        }
    }
}

/// Pool statistics for monitoring and optimization
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Number of buffer acquisitions from pool (reused)
    pub hits: AtomicU64,
    /// Number of new buffer allocations (pool empty)
    pub misses: AtomicU64,
    /// Number of buffers returned to pool
    pub returns: AtomicU64,
    /// Number of buffers dropped (pool full)
    pub drops: AtomicU64,
}

impl PoolStats {
    /// Get hit rate as percentage (0.0 - 100.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.returns.store(0, Ordering::Relaxed);
        self.drops.store(0, Ordering::Relaxed);
    }
}

/// Single-tier buffer pool
struct TierPool {
    /// Available buffers
    buffers: Vec<BytesMut>,
    /// Target buffer size for this tier
    buffer_size: usize,
}

impl TierPool {
    fn new(buffer_size: usize) -> Self {
        Self {
            buffers: Vec::with_capacity(MAX_BUFFERS_PER_TIER),
            buffer_size,
        }
    }

    /// Try to get a buffer from the pool
    fn acquire(&mut self) -> Option<BytesMut> {
        self.buffers.pop().map(|mut buf| {
            buf.clear(); // Reset for reuse
            buf
        })
    }

    /// Return a buffer to the pool
    fn release(&mut self, buf: BytesMut) -> bool {
        if self.buffers.len() < MAX_BUFFERS_PER_TIER {
            self.buffers.push(buf);
            true
        } else {
            false // Pool full, buffer will be dropped
        }
    }

    /// Allocate a new buffer for this tier
    fn allocate(&self) -> BytesMut {
        BytesMut::with_capacity(self.buffer_size)
    }
}

/// Tiered buffer pool for zero-copy packet handling
///
/// Provides pre-allocated buffers of different sizes to minimize
/// heap allocations during packet processing.
pub struct LargeBufferPool {
    /// Small buffer pool (4KB)
    tier1: Mutex<TierPool>,
    /// Medium buffer pool (16KB)
    tier2: Mutex<TierPool>,
    /// Large buffer pool (64KB)
    tier3: Mutex<TierPool>,
    /// Pool statistics
    stats: Arc<PoolStats>,
}

impl LargeBufferPool {
    /// Create a new buffer pool
    ///
    /// Pools start empty and grow on demand up to `MAX_BUFFERS_PER_TIER`.
    pub fn new() -> Self {
        Self {
            tier1: Mutex::new(TierPool::new(TIER_1_SIZE)),
            tier2: Mutex::new(TierPool::new(TIER_2_SIZE)),
            tier3: Mutex::new(TierPool::new(TIER_3_SIZE)),
            stats: Arc::new(PoolStats::default()),
        }
    }

    /// Create a pool with pre-allocated buffers
    ///
    /// # Arguments
    ///
    /// * `tier1_count` - Number of 4KB buffers to pre-allocate
    /// * `tier2_count` - Number of 16KB buffers to pre-allocate
    /// * `tier3_count` - Number of 64KB buffers to pre-allocate
    pub fn with_preallocation(tier1_count: usize, tier2_count: usize, tier3_count: usize) -> Self {
        let pool = Self::new();

        // Pre-allocate tier 1 buffers
        {
            let mut tier = pool.tier1.lock();
            for _ in 0..tier1_count.min(MAX_BUFFERS_PER_TIER) {
                tier.buffers.push(BytesMut::with_capacity(TIER_1_SIZE));
            }
        }

        // Pre-allocate tier 2 buffers
        {
            let mut tier = pool.tier2.lock();
            for _ in 0..tier2_count.min(MAX_BUFFERS_PER_TIER) {
                tier.buffers.push(BytesMut::with_capacity(TIER_2_SIZE));
            }
        }

        // Pre-allocate tier 3 buffers
        {
            let mut tier = pool.tier3.lock();
            for _ in 0..tier3_count.min(MAX_BUFFERS_PER_TIER) {
                tier.buffers.push(BytesMut::with_capacity(TIER_3_SIZE));
            }
        }

        pool
    }

    /// Acquire a buffer of at least the specified size
    ///
    /// Returns a buffer from the appropriate tier pool, or allocates
    /// a new one if the pool is empty.
    ///
    /// # Arguments
    ///
    /// * `min_size` - Minimum required buffer capacity
    ///
    /// # Returns
    ///
    /// A `PooledBuffer` that automatically returns to the pool on drop.
    pub fn acquire(&self, min_size: usize) -> PooledBuffer<'_> {
        let tier = BufferTier::for_size(min_size);
        let buffer = self.acquire_from_tier(tier);

        PooledBuffer {
            buffer: Some(buffer),
            tier,
            pool: self,
        }
    }

    /// Acquire a buffer for a specific tier
    fn acquire_from_tier(&self, tier: BufferTier) -> BytesMut {
        let pool = match tier {
            BufferTier::Small => &self.tier1,
            BufferTier::Medium => &self.tier2,
            BufferTier::Large => &self.tier3,
        };

        let mut guard = pool.lock();
        if let Some(buf) = guard.acquire() {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
            buf
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
            guard.allocate()
        }
    }

    /// Return a buffer to the pool
    fn release(&self, tier: BufferTier, buf: BytesMut) {
        let pool = match tier {
            BufferTier::Small => &self.tier1,
            BufferTier::Medium => &self.tier2,
            BufferTier::Large => &self.tier3,
        };

        let mut guard = pool.lock();
        if guard.release(buf) {
            self.stats.returns.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Get current pool sizes
    pub fn pool_sizes(&self) -> (usize, usize, usize) {
        (
            self.tier1.lock().buffers.len(),
            self.tier2.lock().buffers.len(),
            self.tier3.lock().buffers.len(),
        )
    }
}

impl Default for LargeBufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A buffer borrowed from the pool
///
/// Automatically returns to the pool when dropped.
pub struct PooledBuffer<'a> {
    buffer: Option<BytesMut>,
    tier: BufferTier,
    pool: &'a LargeBufferPool,
}

impl<'a> PooledBuffer<'a> {
    /// Get the underlying BytesMut
    pub fn inner(&self) -> &BytesMut {
        self.buffer.as_ref().unwrap()
    }

    /// Get mutable access to the underlying BytesMut
    pub fn inner_mut(&mut self) -> &mut BytesMut {
        self.buffer.as_mut().unwrap()
    }

    /// Get the buffer's current capacity
    pub fn capacity(&self) -> usize {
        self.buffer.as_ref().unwrap().capacity()
    }

    /// Get the buffer's current length
    pub fn len(&self) -> usize {
        self.buffer.as_ref().unwrap().len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.as_ref().unwrap().is_empty()
    }

    /// Freeze the buffer into immutable Bytes (consumes the pooled buffer)
    ///
    /// Note: This prevents the buffer from being returned to the pool.
    pub fn freeze(mut self) -> Bytes {
        self.buffer.take().unwrap().freeze()
    }

    /// Get the tier this buffer belongs to
    pub fn tier(&self) -> BufferTier {
        self.tier
    }
}

impl std::ops::Deref for PooledBuffer<'_> {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledBuffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap()
    }
}

impl Drop for PooledBuffer<'_> {
    fn drop(&mut self) {
        if let Some(buf) = self.buffer.take() {
            self.pool.release(self.tier, buf);
        }
    }
}

/// Shared packet wrapper using Arc for multi-consumer scenarios
///
/// Allows zero-copy sharing of packet data across multiple consumers
/// (e.g., service detection + logging + pcap capture).
#[derive(Clone)]
pub struct SharedPacket {
    /// Immutable packet data
    data: Bytes,
    /// Original packet length (may differ from data.len() if truncated)
    original_len: usize,
}

impl SharedPacket {
    /// Create from Bytes
    pub fn from_bytes(data: Bytes) -> Self {
        let len = data.len();
        Self {
            data,
            original_len: len,
        }
    }

    /// Create from Vec<u8> (copies data once)
    pub fn from_vec(data: Vec<u8>) -> Self {
        let len = data.len();
        Self {
            data: Bytes::from(data),
            original_len: len,
        }
    }

    /// Create from slice (copies data)
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: Bytes::copy_from_slice(data),
            original_len: data.len(),
        }
    }

    /// Get packet data
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// Get packet data as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get original packet length
    pub fn original_len(&self) -> usize {
        self.original_len
    }

    /// Get current data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if packet is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Create a zero-copy slice of the packet
    ///
    /// This is O(1) and does not copy data.
    pub fn slice(&self, range: std::ops::Range<usize>) -> Bytes {
        self.data.slice(range)
    }

    /// Clone the packet data into a Vec<u8>
    ///
    /// This performs a copy. Prefer using the Bytes directly when possible.
    pub fn to_vec(&self) -> Vec<u8> {
        self.data.to_vec()
    }
}

impl std::ops::Deref for SharedPacket {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<[u8]> for SharedPacket {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for SharedPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedPacket")
            .field("len", &self.data.len())
            .field("original_len", &self.original_len)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_tier_for_size() {
        assert_eq!(BufferTier::for_size(100), BufferTier::Small);
        assert_eq!(BufferTier::for_size(4096), BufferTier::Small);
        assert_eq!(BufferTier::for_size(4097), BufferTier::Medium);
        assert_eq!(BufferTier::for_size(16384), BufferTier::Medium);
        assert_eq!(BufferTier::for_size(16385), BufferTier::Large);
        assert_eq!(BufferTier::for_size(65536), BufferTier::Large);
    }

    #[test]
    fn test_buffer_tier_sizes() {
        assert_eq!(BufferTier::Small.size(), 4096);
        assert_eq!(BufferTier::Medium.size(), 16384);
        assert_eq!(BufferTier::Large.size(), 65536);
    }

    #[test]
    fn test_pool_acquire_small() {
        let pool = LargeBufferPool::new();

        let buf = pool.acquire(100);
        assert!(buf.capacity() >= 100);
        assert_eq!(buf.tier(), BufferTier::Small);
    }

    #[test]
    fn test_pool_acquire_medium() {
        let pool = LargeBufferPool::new();

        let buf = pool.acquire(10000);
        assert!(buf.capacity() >= 10000);
        assert_eq!(buf.tier(), BufferTier::Medium);
    }

    #[test]
    fn test_pool_acquire_large() {
        let pool = LargeBufferPool::new();

        let buf = pool.acquire(50000);
        assert!(buf.capacity() >= 50000);
        assert_eq!(buf.tier(), BufferTier::Large);
    }

    #[test]
    fn test_pool_reuse() {
        let pool = LargeBufferPool::new();

        // Acquire and release
        {
            let mut buf = pool.acquire(100);
            buf.extend_from_slice(b"test data");
        }

        // Stats should show 1 miss (new allocation), 1 return
        assert_eq!(pool.stats().misses.load(Ordering::Relaxed), 1);
        assert_eq!(pool.stats().returns.load(Ordering::Relaxed), 1);

        // Next acquire should hit the pool
        {
            let buf = pool.acquire(100);
            assert!(buf.is_empty()); // Buffer was cleared
        }

        assert_eq!(pool.stats().hits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_pool_stats_hit_rate() {
        let pool = LargeBufferPool::new();

        // Initial hit rate should be 0
        assert_eq!(pool.stats().hit_rate(), 0.0);

        // Acquire (miss)
        let _buf1 = pool.acquire(100);
        assert_eq!(pool.stats().hit_rate(), 0.0);

        // Return and acquire again (hit)
        drop(_buf1);
        let _buf2 = pool.acquire(100);
        assert_eq!(pool.stats().hit_rate(), 50.0);
    }

    #[test]
    fn test_preallocation() {
        let pool = LargeBufferPool::with_preallocation(10, 5, 2);

        let (t1, t2, t3) = pool.pool_sizes();
        assert_eq!(t1, 10);
        assert_eq!(t2, 5);
        assert_eq!(t3, 2);
    }

    #[test]
    fn test_pooled_buffer_deref() {
        let pool = LargeBufferPool::new();

        let mut buf = pool.acquire(100);
        buf.extend_from_slice(b"hello");
        assert_eq!(&buf[..], b"hello");
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn test_pooled_buffer_freeze() {
        let pool = LargeBufferPool::new();

        let mut buf = pool.acquire(100);
        buf.extend_from_slice(b"frozen data");

        let frozen = buf.freeze();
        assert_eq!(&frozen[..], b"frozen data");

        // Buffer was consumed, not returned to pool
        assert_eq!(pool.stats().returns.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_shared_packet_from_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let packet = SharedPacket::from_vec(data);

        assert_eq!(packet.len(), 5);
        assert_eq!(packet.original_len(), 5);
        assert_eq!(packet.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_shared_packet_slice() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let packet = SharedPacket::from_vec(data);

        let slice = packet.slice(2..6);
        assert_eq!(&slice[..], &[3, 4, 5, 6]);
    }

    #[test]
    fn test_shared_packet_clone() {
        let packet = SharedPacket::from_slice(b"shared data");
        let cloned = packet.clone();

        // Both should point to same underlying data
        assert_eq!(packet.as_slice(), cloned.as_slice());
    }

    #[test]
    fn test_10kb_packet_handling() {
        let pool = LargeBufferPool::new();

        // Acquire buffer for >10KB packet
        let mut buf = pool.acquire(12000);
        assert!(buf.capacity() >= 12000);
        assert_eq!(buf.tier(), BufferTier::Medium);

        // Write 12KB of data
        let data = vec![0xAB; 12000];
        buf.extend_from_slice(&data);
        assert_eq!(buf.len(), 12000);
    }

    #[test]
    fn test_concurrent_pool_access() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(LargeBufferPool::with_preallocation(10, 5, 2));
        let mut handles = vec![];

        for _ in 0..4 {
            let pool_clone = Arc::clone(&pool);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let mut buf = pool_clone.acquire(1000);
                    buf.extend_from_slice(b"test");
                    // Buffer returned on drop
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have processed 400 acquire/release cycles
        let total =
            pool.stats().hits.load(Ordering::Relaxed) + pool.stats().misses.load(Ordering::Relaxed);
        assert_eq!(total, 400);
    }

    #[test]
    fn test_pool_overflow() {
        let pool = LargeBufferPool::new();

        // Create more buffers than pool can hold
        let buffers: Vec<_> = (0..MAX_BUFFERS_PER_TIER + 10)
            .map(|_| pool.acquire(100))
            .collect();

        // Drop all buffers - some should be dropped (pool full)
        drop(buffers);

        let (t1_size, _, _) = pool.pool_sizes();
        assert_eq!(t1_size, MAX_BUFFERS_PER_TIER);
        assert!(pool.stats().drops.load(Ordering::Relaxed) >= 10);
    }
}
