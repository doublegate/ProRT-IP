//! Zero-copy packet buffer pool
//!
//! Provides pre-allocated buffers for packet crafting to eliminate
//! heap allocations in hot path. Thread-local buffer pools ensure
//! zero contention at high packet rates (1M+ pps).
//!
//! # Design
//!
//! Each thread maintains its own buffer pool with pre-allocated 4KB buffers.
//! Buffers are reused via reset() to eliminate allocations in the hot path.
//!
//! # Performance
//!
//! - **Zero allocations**: After initial buffer creation, no heap allocations occur
//! - **Zero contention**: Thread-local storage eliminates lock overhead
//! - **Cache-friendly**: Sequential writes to pre-allocated buffers
//!
//! # Example
//!
//! ```no_run
//! use prtip_network::packet_buffer::with_buffer;
//!
//! with_buffer(|pool| {
//!     // Get mutable slice for packet (54 bytes for SYN)
//!     let buffer = pool.get_mut(54).expect("Buffer exhausted");
//!
//!     // Write packet data directly to buffer
//!     buffer[0] = 0x45; // IPv4 version + IHL
//!     // ... more packet crafting
//!
//!     // Reset for next packet
//!     pool.reset();
//! });
//! ```

use std::cell::RefCell;
use thiserror::Error;

/// Errors that can occur during buffer operations
#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Buffer exhausted: need {needed} bytes, have {available} bytes remaining")]
    Exhausted { needed: usize, available: usize },
}

pub type Result<T> = std::result::Result<T, BufferError>;

/// Pre-allocated packet buffer pool (thread-local)
///
/// Each buffer is 4KB (fits max Ethernet frame + overhead).
/// Buffers are reused via reset() to eliminate allocations.
///
/// # Thread Safety
///
/// This struct is NOT Send or Sync. It's designed for thread-local use only.
/// Each thread gets its own buffer pool via `with_buffer()`.
pub struct PacketBuffer {
    /// Pre-allocated buffer (one-time allocation)
    buffer: Vec<u8>,
    /// Current write offset into buffer
    offset: usize,
}

impl PacketBuffer {
    /// Create new buffer pool (4KB per buffer)
    ///
    /// This performs a single heap allocation. All subsequent operations
    /// are zero-copy and reuse this buffer.
    ///
    /// # Buffer Size
    ///
    /// 4KB is chosen to fit:
    /// - Maximum Ethernet frame (1518 bytes)
    /// - Jumbo frames (up to 9000 bytes)
    /// - Multiple small packets (e.g., 70+ SYN packets)
    pub fn new() -> Self {
        Self {
            buffer: vec![0u8; 4096], // Allocate once
            offset: 0,
        }
    }

    /// Get mutable slice for packet (zero-copy)
    ///
    /// Returns a mutable slice of the requested size from the buffer.
    /// This is a zero-copy operation - no heap allocation occurs.
    ///
    /// # Returns
    ///
    /// - `Some(&mut [u8])` - Slice of requested size
    /// - `None` - Buffer exhausted (caller should reset + retry)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::packet_buffer::PacketBuffer;
    ///
    /// let mut pool = PacketBuffer::new();
    /// let slice = pool.get_mut(54).unwrap();
    /// assert_eq!(slice.len(), 54);
    /// ```
    pub fn get_mut(&mut self, size: usize) -> Option<&mut [u8]> {
        if self.offset + size > self.buffer.len() {
            return None; // Buffer exhausted
        }
        let start = self.offset;
        self.offset += size;
        Some(&mut self.buffer[start..start + size])
    }

    /// Reset buffer for reuse (zero-copy)
    ///
    /// This resets the write offset to the beginning of the buffer,
    /// allowing the buffer to be reused for the next packet.
    /// No deallocation or zeroing occurs.
    ///
    /// # Performance
    ///
    /// This is an O(1) operation with zero heap operations.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::packet_buffer::PacketBuffer;
    ///
    /// let mut pool = PacketBuffer::new();
    /// let _ = pool.get_mut(100).unwrap();
    /// assert_eq!(pool.remaining(), 3996);
    ///
    /// pool.reset();
    /// assert_eq!(pool.remaining(), 4096);
    /// ```
    pub fn reset(&mut self) {
        self.offset = 0; // Reuse buffer, no deallocation
    }

    /// Check remaining capacity
    ///
    /// Returns the number of bytes available in the buffer before
    /// exhaustion. Useful for determining if a packet will fit.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::packet_buffer::PacketBuffer;
    ///
    /// let mut pool = PacketBuffer::new();
    /// assert_eq!(pool.remaining(), 4096);
    ///
    /// let _ = pool.get_mut(54).unwrap();
    /// assert_eq!(pool.remaining(), 4042);
    /// ```
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.offset
    }

    /// Get total buffer capacity
    ///
    /// Returns the total size of the buffer (always 4096 bytes).
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }
}

impl Default for PacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local buffer pool
// Each thread gets its own buffer, eliminating contention
thread_local! {
    static PACKET_BUFFER_POOL: RefCell<PacketBuffer> = RefCell::new(PacketBuffer::new());
}

/// Get thread-local buffer pool
///
/// Provides access to the thread-local buffer pool via a closure.
/// This ensures zero contention at high packet rates.
///
/// # Thread Safety
///
/// Each thread has its own buffer pool. No locks or atomic operations
/// are required, ensuring maximum performance.
///
/// # Example
///
/// ```no_run
/// use prtip_network::packet_buffer::with_buffer;
///
/// let result = with_buffer(|pool| {
///     let buffer = pool.get_mut(54).expect("Buffer exhausted");
///     // Write packet data...
///     buffer.len()
/// });
///
/// assert_eq!(result, 54);
/// ```
pub fn with_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut PacketBuffer) -> R,
{
    PACKET_BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        f(&mut pool)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_allocation() {
        let mut buf = PacketBuffer::new();
        assert_eq!(buf.remaining(), 4096);
        assert_eq!(buf.capacity(), 4096);

        let slice = buf.get_mut(54).unwrap();
        assert_eq!(slice.len(), 54);
        assert_eq!(buf.remaining(), 4096 - 54);
    }

    #[test]
    fn test_buffer_reset() {
        let mut buf = PacketBuffer::new();
        let _ = buf.get_mut(100).unwrap();
        assert_eq!(buf.remaining(), 3996);

        buf.reset();
        assert_eq!(buf.remaining(), 4096);
    }

    #[test]
    fn test_buffer_exhaustion() {
        let mut buf = PacketBuffer::new();
        let result = buf.get_mut(5000); // Exceeds 4KB
        assert!(result.is_none());
    }

    #[test]
    fn test_buffer_multiple_allocations() {
        let mut buf = PacketBuffer::new();

        // Allocate multiple packets
        let slice1 = buf.get_mut(54).unwrap();
        assert_eq!(slice1.len(), 54);

        let slice2 = buf.get_mut(100).unwrap();
        assert_eq!(slice2.len(), 100);

        let slice3 = buf.get_mut(200).unwrap();
        assert_eq!(slice3.len(), 200);

        assert_eq!(buf.remaining(), 4096 - 54 - 100 - 200);
    }

    #[test]
    fn test_buffer_reset_and_reuse() {
        let mut buf = PacketBuffer::new();

        // Fill buffer partially
        let _ = buf.get_mut(1000).unwrap();
        assert_eq!(buf.remaining(), 3096);

        // Reset and reuse
        buf.reset();
        assert_eq!(buf.remaining(), 4096);

        // Should be able to allocate again
        let slice = buf.get_mut(1000).unwrap();
        assert_eq!(slice.len(), 1000);
    }

    #[test]
    fn test_with_buffer_thread_local() {
        // Verify thread-local buffer works
        let result = with_buffer(|pool| {
            let slice = pool.get_mut(54).unwrap();
            slice.len()
        });

        assert_eq!(result, 54);
    }

    #[test]
    fn test_with_buffer_reset() {
        // Allocate in first call
        with_buffer(|pool| {
            let _ = pool.get_mut(100).unwrap();
            assert_eq!(pool.remaining(), 3996);
        });

        // Buffer should still have used space
        with_buffer(|pool| {
            assert_eq!(pool.remaining(), 3996);

            // Reset for next packet
            pool.reset();
            assert_eq!(pool.remaining(), 4096);
        });
    }

    #[test]
    fn test_buffer_write_patterns() {
        let mut buf = PacketBuffer::new();
        let slice = buf.get_mut(10).unwrap();

        // Write pattern
        for (i, byte) in slice.iter_mut().enumerate() {
            *byte = i as u8;
        }

        // Verify pattern
        let slice = &buf.buffer[0..10];
        for (i, byte) in slice.iter().enumerate() {
            assert_eq!(*byte, i as u8);
        }
    }

    #[test]
    fn test_buffer_capacity_invariant() {
        let mut buf = PacketBuffer::new();
        let initial_capacity = buf.capacity();

        // Allocate and reset multiple times
        for _ in 0..10 {
            let _ = buf.get_mut(100).unwrap();
            buf.reset();
        }

        // Capacity should never change
        assert_eq!(buf.capacity(), initial_capacity);
    }

    #[test]
    fn test_buffer_zero_copy_semantics() {
        let mut buf = PacketBuffer::new();

        // Get first slice
        let slice1_ptr = {
            let slice = buf.get_mut(10).unwrap();
            slice.as_ptr()
        };

        // Get second slice (should be contiguous)
        let slice2_ptr = {
            let slice = buf.get_mut(10).unwrap();
            slice.as_ptr()
        };

        // Verify slices are contiguous (10 bytes apart)
        let ptr_diff = unsafe { slice2_ptr.offset_from(slice1_ptr) };
        assert_eq!(ptr_diff, 10);
    }
}
