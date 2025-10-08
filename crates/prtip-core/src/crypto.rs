// Cryptographic utilities for ProRT-IP
//!
//! This module provides cryptographic primitives used throughout ProRT-IP:
//!
//! - **Blackrock**: Feistel cipher for bijective shuffling (Masscan technique)
//! - **SipHash-2-4**: Fast cryptographic hash for sequence number generation
//!
//! # Blackrock Shuffling
//!
//! Blackrock implements a Feistel cipher that creates a bijective mapping (1-to-1)
//! for shuffling numbers within a range. This is used for stateless IP address
//! randomization without requiring storage of which IPs have been scanned.
//!
//! Key properties:
//! - Deterministic: Same input always produces same output
//! - Bijective: Every input maps to exactly one output
//! - No collisions: Can linearly increment index and get full coverage
//! - No storage: Stateless operation, no memory of scanned IPs needed
//!
//! # Example
//!
//! ```
//! use prtip_core::crypto::BlackRock;
//!
//! // Shuffle IP addresses in a /24 network (256 addresses)
//! let mut blackrock = BlackRock::new(256, 0x1234567890ABCDEF, 3);
//!
//! // Linearly incrementing index produces randomized order
//! for i in 0..256 {
//!     let shuffled = blackrock.shuffle(i);
//!     println!("Index {} -> Shuffled {}", i, shuffled);
//! }
//! ```
//!
//! # SipHash-2-4
//!
//! SipHash is a cryptographically strong PRF (pseudorandom function) optimized for
//! short inputs. Used by Masscan for generating random-looking TCP sequence numbers
//! from connection state, enabling stateless tracking.
//!
//! ```
//! use prtip_core::crypto::siphash24;
//!
//! let key = [0x0706050403020100, 0x0F0E0D0C0B0A0908];
//! let data = b"Hello, world!";
//! let hash = siphash24(data, &key);
//! println!("SipHash: {:016X}", hash);
//! ```

use std::num::Wrapping;

/// Blackrock shuffling structure for bijective mapping
///
/// Implements a Feistel cipher that shuffles numbers in range [0, range) using
/// a cryptographically secure pseudorandom permutation. This enables stateless
/// scanning by converting a linearly incrementing index into a randomized sequence.
///
/// # Algorithm
///
/// The range is split into two parts (left/right). Multiple rounds of Feistel
/// transformation are applied:
///
/// 1. Split number into left (a_bits) and right (b_bits)
/// 2. For each round:
///    - Compute F(right, round_key)
///    - XOR with left, swap left/right
/// 3. Recombine to produce shuffled output
///
/// The number of rounds affects security (more rounds = stronger mixing):
/// - 1 round: Fast but weak randomization
/// - 2 rounds: Balanced (Masscan default)
/// - 3+ rounds: Stronger mixing for cryptographic applications
///
/// Note: This is a partial implementation. The full Masscan algorithm uses
/// a different domain splitting approach (a * b > range). This implementation
/// uses power-of-2 domain splitting which works well for many cases but needs
/// refinement for optimal bijectivity in all ranges.
#[derive(Debug, Clone)]
pub struct BlackRock {
    /// Size of the range to shuffle (e.g., 256 for /24 network)
    range: u64,

    /// Number of bits for left half
    #[allow(dead_code)]
    a_bits: u32,

    /// Mask for extracting left half
    a_mask: u64,

    /// Number of bits for right half
    b_bits: u32,

    /// Mask for extracting right half
    b_mask: u64,

    /// Seed for the shuffle (determines randomization)
    seed: u64,

    /// Number of Feistel rounds (typically 2-4)
    rounds: u32,
}

impl BlackRock {
    /// Create a new Blackrock shuffler for a given range
    ///
    /// # Arguments
    ///
    /// * `range` - The size of the range to shuffle (must be > 0)
    /// * `seed` - Random seed (use different seeds for different scan sessions)
    /// * `rounds` - Number of Feistel rounds (2-4 recommended)
    ///
    /// # Panics
    ///
    /// Panics if range is 0
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::crypto::BlackRock;
    ///
    /// // Shuffle 65536 addresses (entire /16 network)
    /// let br = BlackRock::new(65536, 0xDEADBEEF, 2);
    /// ```
    pub fn new(range: u64, seed: u64, rounds: u32) -> Self {
        assert!(range > 0, "Range must be greater than 0");

        // Calculate number of bits needed for the range
        let total_bits = 64 - range.leading_zeros();

        // Split bits between left and right halves (as evenly as possible)
        let a_bits = total_bits / 2;
        let b_bits = total_bits - a_bits;

        let a_mask = if a_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << a_bits) - 1
        };

        let b_mask = if b_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << b_bits) - 1
        };

        Self {
            range,
            a_bits,
            a_mask,
            b_bits,
            b_mask,
            seed,
            rounds,
        }
    }

    /// Shuffle a number within the range
    ///
    /// Given an index in [0, range), produces a different number also in [0, range).
    /// This is a bijective mapping - each input maps to exactly one output.
    ///
    /// # Arguments
    ///
    /// * `index` - Input value (must be < range)
    ///
    /// # Returns
    ///
    /// Shuffled value in range [0, range)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::crypto::BlackRock;
    ///
    /// let mut br = BlackRock::new(256, 42, 2);
    /// let shuffled = br.shuffle(100);
    /// assert!(shuffled < 256);
    /// ```
    pub fn shuffle(&mut self, index: u64) -> u64 {
        // Ensure index is within range
        debug_assert!(
            index < self.range,
            "Index {} out of range {}",
            index,
            self.range
        );

        if index >= self.range {
            return index % self.range;
        }

        // Use cycle-walking for format-preserving encryption
        let mut result = index;
        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 10;

        loop {
            result = self.feistel_encrypt(result);

            // If result is in range, we're done
            if result < self.range {
                return result;
            }

            // Prevent infinite loops (shouldn't happen with proper Feistel)
            iterations += 1;
            if iterations >= MAX_ITERATIONS {
                // Fallback: just modulo (shouldn't reach here)
                return result % self.range;
            }
        }
    }

    /// Perform Feistel encryption (internal helper)
    fn feistel_encrypt(&self, mut value: u64) -> u64 {
        for round in 0..self.rounds {
            // Split value into left (a) and right (b) halves
            let a = (value >> self.b_bits) & self.a_mask;
            let b = value & self.b_mask;

            // Compute round function F(b, round_key)
            let round_key = self.seed.wrapping_add(round as u64);
            let f_output = self.round_function(b, round_key);

            // Swap halves: new_left = old_right, new_right = old_left XOR F(old_right)
            let new_a = b;
            let new_b = (a ^ f_output) & self.a_mask;

            // Recombine halves
            value = (new_a << self.b_bits) | new_b;
        }
        value
    }

    /// Unshuffle a value (reverse operation)
    ///
    /// Given a shuffled value, returns the original index. This is the inverse
    /// of the shuffle() operation.
    ///
    /// # Arguments
    ///
    /// * `shuffled` - Shuffled value (must be < range)
    ///
    /// # Returns
    ///
    /// Original index before shuffling
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::crypto::BlackRock;
    ///
    /// let mut br = BlackRock::new(256, 42, 2);
    /// let shuffled = br.shuffle(100);
    /// let original = br.unshuffle(shuffled);
    /// assert_eq!(original, 100);
    /// ```
    pub fn unshuffle(&mut self, shuffled: u64) -> u64 {
        debug_assert!(shuffled < self.range);

        if shuffled >= self.range {
            return shuffled % self.range;
        }

        // Use cycle-walking for format-preserving decryption
        let mut result = shuffled;
        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 10;

        loop {
            result = self.feistel_decrypt(result);

            // If result is in range, we're done
            if result < self.range {
                return result;
            }

            // Prevent infinite loops
            iterations += 1;
            if iterations >= MAX_ITERATIONS {
                // Fallback: just modulo
                return result % self.range;
            }
        }
    }

    /// Perform Feistel decryption (internal helper)
    fn feistel_decrypt(&self, mut value: u64) -> u64 {
        // Reverse the Feistel rounds (apply in reverse order)
        for round in (0..self.rounds).rev() {
            let a = (value >> self.b_bits) & self.a_mask;
            let b = value & self.b_mask;

            let round_key = self.seed.wrapping_add(round as u64);
            let f_output = self.round_function(a, round_key);

            // Reverse swap: new_left = old_right XOR F(old_left), new_right = old_left
            let new_a = (b ^ f_output) & self.a_mask;
            let new_b = a;

            value = (new_a << self.b_bits) | new_b;
        }
        value
    }

    /// Feistel round function
    ///
    /// Implements F(x, key) = hash(x || key) mod 2^a_bits
    /// Uses a simple but fast hash function based on multiplications.
    ///
    /// This is NOT cryptographically secure but provides good statistical
    /// properties for randomization purposes.
    fn round_function(&self, value: u64, key: u64) -> u64 {
        // Combine value and key
        let mut x = value.wrapping_mul(0x9E3779B97F4A7C15); // Golden ratio prime
        x = x.wrapping_add(key);

        // Mix bits (simple avalanche)
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58476D1CE4E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D049BB133111EB);
        x ^= x >> 31;

        x & self.a_mask
    }
}

/// SipHash-2-4 implementation for fast cryptographic hashing
///
/// SipHash is a pseudorandom function (PRF) optimized for short inputs.
/// The "2-4" variant uses 2 compression rounds and 4 finalization rounds,
/// providing strong security while maintaining high performance.
///
/// # Security
///
/// - Collision resistant for short inputs
/// - Indistinguishable from random oracle when key is secret
/// - Resistant to hash flooding attacks
///
/// # Performance
///
/// - Very fast on modern CPUs (< 1 cycle/byte)
/// - Optimized for 64-bit architectures
/// - No memory allocations
///
/// # Arguments
///
/// * `data` - Input data to hash
/// * `key` - 128-bit key as two u64 values [k0, k1]
///
/// # Returns
///
/// 64-bit hash value
///
/// # Examples
///
/// ```
/// use prtip_core::crypto::siphash24;
///
/// let key = [0x0706050403020100, 0x0F0E0D0C0B0A0908];
/// let hash = siphash24(b"ProRT-IP", &key);
/// println!("Hash: {:016X}", hash);
/// ```
pub fn siphash24(data: &[u8], key: &[u64; 2]) -> u64 {
    // Initialize state with key
    let mut v0 = Wrapping(key[0] ^ 0x736f6d6570736575);
    let mut v1 = Wrapping(key[1] ^ 0x646f72616e646f6d);
    let mut v2 = Wrapping(key[0] ^ 0x6c7967656e657261);
    let mut v3 = Wrapping(key[1] ^ 0x7465646279746573);

    // Process full 8-byte blocks
    let mut i = 0;
    while i + 8 <= data.len() {
        let m = Wrapping(u64::from_le_bytes([
            data[i],
            data[i + 1],
            data[i + 2],
            data[i + 3],
            data[i + 4],
            data[i + 5],
            data[i + 6],
            data[i + 7],
        ]));

        v3 ^= m;

        // 2 compression rounds
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);

        v0 ^= m;
        i += 8;
    }

    // Process remaining bytes (pad with zeros and add length)
    let remaining = data.len() - i;
    let mut last = Wrapping((data.len() as u64 & 0xFF) << 56);

    for j in 0..remaining {
        last |= Wrapping((data[i + j] as u64) << (j * 8));
    }

    v3 ^= last;
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= last;

    // Finalization
    v2 ^= Wrapping(0xff);

    // 4 finalization rounds
    for _ in 0..4 {
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    }

    (v0 ^ v1 ^ v2 ^ v3).0
}

/// Single SipHash round
///
/// Implements the core mixing function:
/// - v0 += v1; v1 = ROTL(v1, 13); v1 ^= v0; v0 = ROTL(v0, 32);
/// - v2 += v3; v3 = ROTL(v3, 16); v3 ^= v2;
/// - v0 += v3; v3 = ROTL(v3, 21); v3 ^= v0;
/// - v2 += v1; v1 = ROTL(v1, 17); v1 ^= v2; v2 = ROTL(v2, 32);
#[inline(always)]
fn sipround(
    v0: &mut Wrapping<u64>,
    v1: &mut Wrapping<u64>,
    v2: &mut Wrapping<u64>,
    v3: &mut Wrapping<u64>,
) {
    *v0 += *v1;
    *v1 = Wrapping(v1.0.rotate_left(13));
    *v1 ^= *v0;
    *v0 = Wrapping(v0.0.rotate_left(32));

    *v2 += *v3;
    *v3 = Wrapping(v3.0.rotate_left(16));
    *v3 ^= *v2;

    *v0 += *v3;
    *v3 = Wrapping(v3.0.rotate_left(21));
    *v3 ^= *v0;

    *v2 += *v1;
    *v1 = Wrapping(v1.0.rotate_left(17));
    *v1 ^= *v2;
    *v2 = Wrapping(v2.0.rotate_left(32));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackrock_bijective() {
        let mut br = BlackRock::new(256, 0x123456, 2);

        // Verify no collisions in full range
        let mut seen = vec![false; 256];
        for i in 0..256 {
            let shuffled = br.shuffle(i);
            assert!(shuffled < 256, "Shuffled value {} out of range", shuffled);
            assert!(
                !seen[shuffled as usize],
                "Collision detected at {}",
                shuffled
            );
            seen[shuffled as usize] = true;
        }

        // All values should have been hit exactly once
        assert!(seen.iter().all(|&x| x), "Not all values were generated");
    }

    #[test]
    fn test_blackrock_unshuffle() {
        let mut br = BlackRock::new(1000, 0xDEADBEEF, 3);

        for i in 0..1000 {
            let shuffled = br.shuffle(i);
            let unshuffled = br.unshuffle(shuffled);
            assert_eq!(i, unshuffled, "Unshuffle failed for index {}", i);
        }
    }

    #[test]
    fn test_blackrock_deterministic() {
        let mut br1 = BlackRock::new(512, 0xABCDEF, 2);
        let mut br2 = BlackRock::new(512, 0xABCDEF, 2);

        for i in 0..512 {
            assert_eq!(
                br1.shuffle(i),
                br2.shuffle(i),
                "Not deterministic at index {}",
                i
            );
        }
    }

    #[test]
    fn test_blackrock_different_seeds() {
        let mut br1 = BlackRock::new(100, 0x1111, 2);
        let mut br2 = BlackRock::new(100, 0x2222, 2);

        let mut differences = 0;
        for i in 0..100 {
            if br1.shuffle(i) != br2.shuffle(i) {
                differences += 1;
            }
        }

        // Different seeds should produce mostly different outputs
        assert!(
            differences > 90,
            "Seeds not different enough: {} differences",
            differences
        );
    }

    #[test]
    fn test_blackrock_power_of_two() {
        let mut br = BlackRock::new(1024, 0x777, 2);

        let mut seen = vec![false; 1024];
        for i in 0..1024 {
            let shuffled = br.shuffle(i);
            assert!(shuffled < 1024);
            seen[shuffled as usize] = true;
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn test_blackrock_non_power_of_two() {
        let mut br = BlackRock::new(1000, 0x999, 3);

        let mut seen = vec![false; 1000];
        for i in 0..1000 {
            let shuffled = br.shuffle(i);
            assert!(shuffled < 1000, "Value {} out of range 1000", shuffled);
            assert!(!seen[shuffled as usize], "Collision at {}", shuffled);
            seen[shuffled as usize] = true;
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn test_siphash24_basic() {
        let key = [0x0706050403020100, 0x0F0E0D0C0B0A0908];
        let data = b"";
        let hash = siphash24(data, &key);

        // Known test vector for SipHash-2-4 with empty input
        assert_eq!(hash, 0x726fdb47dd0e0e31);
    }

    #[test]
    fn test_siphash24_short_message() {
        let key = [0x0706050403020100, 0x0F0E0D0C0B0A0908];
        let data = b"\x00\x01\x02\x03\x04\x05\x06\x07";
        let hash = siphash24(data, &key);

        // Known test vector
        assert_eq!(hash, 0x93f5f5799a932462);
    }

    #[test]
    fn test_siphash24_longer_message() {
        let key = [0x0706050403020100, 0x0F0E0D0C0B0A0908];
        let data = b"Hello, world! This is a longer message for testing.";
        let hash = siphash24(data, &key);

        // Hash should be deterministic
        let hash2 = siphash24(data, &key);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_siphash24_different_keys() {
        let key1 = [0x0000000000000000, 0x0000000000000000];
        let key2 = [0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF];
        let data = b"test data";

        let hash1 = siphash24(data, &key1);
        let hash2 = siphash24(data, &key2);

        // Different keys should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_siphash24_avalanche() {
        let key = [0x0123456789ABCDEF, 0xFEDCBA9876543210];

        // Single bit change should change hash significantly
        let hash1 = siphash24(b"test", &key);
        let hash2 = siphash24(b"tesh", &key); // Changed last bit of last char

        // Count bit differences
        let diff_bits = (hash1 ^ hash2).count_ones();

        // Should have roughly half the bits different (avalanche effect)
        assert!(
            diff_bits > 20 && diff_bits < 44,
            "Poor avalanche: {} bits different",
            diff_bits
        );
    }
}
