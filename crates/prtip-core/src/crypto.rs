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
/// a format-preserving encryption technique based on the paper:
/// "Ciphers with Arbitrary Finite Domains" by Black and Rogaway.
///
/// This is the same algorithm used by Masscan for stateless IP randomization.
///
/// # Algorithm
///
/// The key insight is splitting the domain into two factors a and b such that
/// a × b > range. The number m is represented as m = a*R + L where:
/// - L = m mod a (left half)
/// - R = m div a (right half)
///
/// Multiple rounds of Feistel transformation are applied:
/// 1. For odd rounds: L' = (L + F(R, round)) mod a
/// 2. For even rounds: L' = (L + F(R, round)) mod b
/// 3. Swap L and R
///
/// After all rounds, recombine based on round parity.
/// Use cycle-walking to handle values >= range.
///
/// # Properties
///
/// - Deterministic: Same input always produces same output
/// - Bijective: Every input maps to exactly one output (no collisions)
/// - Format-preserving: Output is in same range as input
/// - Stateless: No memory of which values have been generated
///
/// # Rounds
///
/// - 2 rounds: Masscan default (good balance of speed and randomization)
/// - 3 rounds: Better mixing for security-critical applications
/// - 4 rounds: Strongest mixing (diminishing returns beyond this)
#[derive(Debug, Clone)]
pub struct BlackRock {
    /// Size of the range to shuffle (e.g., 256 for /24 network)
    range: u64,

    /// Left domain factor (a × b > range)
    a: u64,

    /// Right domain factor (a × b > range)
    b: u64,

    /// Seed for the shuffle (determines randomization)
    seed: u64,

    /// Number of Feistel rounds (typically 2-4)
    rounds: u32,
}

impl BlackRock {
    /// Create a new Blackrock shuffler for a given range
    ///
    /// Calculates domain factors a and b such that a × b > range, following
    /// the Masscan algorithm. For small ranges (< 9), uses hardcoded values
    /// for better statistical properties.
    ///
    /// # Arguments
    ///
    /// * `range` - The size of the range to shuffle (must be > 0)
    /// * `seed` - Random seed (use different seeds for different scan sessions)
    /// * `rounds` - Number of Feistel rounds (2-4 recommended, Masscan uses 2)
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

        // Calculate a and b such that a × b > range
        // This algorithm matches Masscan's crypto-blackrock.c
        let (a, b) = match range {
            // Small ranges: use hardcoded values for better randomization
            0 => (0, 0), // Should never happen due to assert
            1 => (1, 1),
            2 => (1, 2),
            3 => (2, 2),
            4..=6 => (2, 3),
            7..=8 => (3, 3),
            _ => {
                // For larger ranges: a ≈ sqrt(range) - 2, b ≈ sqrt(range) + 3
                let sqrt = (range as f64).sqrt();
                let mut a = (sqrt - 2.0) as u64;
                let mut b = (sqrt + 3.0) as u64;

                // Ensure a is at least 1
                if a == 0 {
                    a = 1;
                }

                // Increment b until a × b > range
                while a * b <= range {
                    b += 1;
                }

                (a, b)
            }
        };

        Self {
            range,
            a,
            b,
            seed,
            rounds,
        }
    }

    /// Shuffle a number within the range
    ///
    /// Given an index in [0, range), produces a different number also in [0, range).
    /// This is a bijective mapping - each input maps to exactly one output.
    ///
    /// Uses cycle-walking: if encrypted value is >= range, re-encrypt until valid.
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
    pub fn shuffle(&self, index: u64) -> u64 {
        debug_assert!(
            index < self.range,
            "Index {} out of range {}",
            index,
            self.range
        );

        if index >= self.range {
            return index % self.range;
        }

        // Cycle-walking: re-encrypt until result is in range
        let mut result = self.encrypt(index);
        while result >= self.range {
            result = self.encrypt(result);
        }
        result
    }

    /// Encrypt a value using Feistel network (Masscan algorithm)
    ///
    /// This implements the ENCRYPT function from Black and Rogaway's paper.
    /// The value m is split as m = a*R + L, then transformed through rounds.
    fn encrypt(&self, m: u64) -> u64 {
        // Split m = a*R + L
        let mut left = m % self.a;
        let mut right = m / self.a;

        // Apply rounds
        for round in 1..=self.rounds {
            let tmp = if round & 1 == 1 {
                // Odd round: use modulo a
                (left + self.round_function(round, right)) % self.a
            } else {
                // Even round: use modulo b
                (left + self.round_function(round, right)) % self.b
            };
            left = right;
            right = tmp;
        }

        // Recombine based on round parity
        if self.rounds & 1 == 1 {
            self.a * left + right
        } else {
            self.a * right + left
        }
    }

    /// Unshuffle a value (reverse operation)
    ///
    /// Given a shuffled value, returns the original index. This is the inverse
    /// of the shuffle() operation.
    ///
    /// Uses cycle-walking: if decrypted value is >= range, re-decrypt until valid.
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
    pub fn unshuffle(&self, shuffled: u64) -> u64 {
        debug_assert!(shuffled < self.range);

        if shuffled >= self.range {
            return shuffled % self.range;
        }

        // Cycle-walking: re-decrypt until result is in range
        let mut result = self.decrypt(shuffled);
        while result >= self.range {
            result = self.decrypt(result);
        }
        result
    }

    /// Decrypt a value using Feistel network (Masscan algorithm)
    ///
    /// This implements the UNENCRYPT function from Black and Rogaway's paper.
    /// Reverses the encryption process.
    fn decrypt(&self, m: u64) -> u64 {
        // Split based on round parity (opposite of encrypt)
        let (mut left, mut right) = if self.rounds & 1 == 1 {
            (m / self.a, m % self.a) // Odd rounds: reverse split
        } else {
            (m % self.a, m / self.a) // Even rounds: normal split
        };

        // Apply rounds in reverse
        for round in (1..=self.rounds).rev() {
            let f_value = self.round_function(round, left);

            let tmp = if round & 1 == 1 {
                // Odd round: subtract mod a
                if f_value > right {
                    let diff = f_value - right;
                    let remainder = diff % self.a;
                    if remainder == 0 {
                        0
                    } else {
                        self.a - remainder
                    }
                } else {
                    (right - f_value) % self.a
                }
            } else {
                // Even round: subtract mod b
                if f_value > right {
                    let diff = f_value - right;
                    let remainder = diff % self.b;
                    if remainder == 0 {
                        0
                    } else {
                        self.b - remainder
                    }
                } else {
                    (right - f_value) % self.b
                }
            };

            right = left;
            left = tmp;
        }

        // Recombine
        self.a * right + left
    }

    /// Feistel round function
    ///
    /// Simplified version inspired by Masscan's READ() function but adapted
    /// for Rust. Provides good statistical mixing without cryptographic overhead.
    ///
    /// # Arguments
    ///
    /// * `round` - Round number (1-based)
    /// * `value` - Input value to mix
    ///
    /// # Returns
    ///
    /// Mixed value
    fn round_function(&self, round: u32, value: u64) -> u64 {
        // Combine round, value, and seed
        let mut x = value;
        x ^= (self.seed << (round as u64 % 64)) ^ (self.seed >> (64 - (round as u64 % 64)));

        // Mix using golden ratio prime (same as MurmurHash3)
        x = x.wrapping_mul(0x9E3779B97F4A7C15);
        x ^= x >> 33;
        x = x.wrapping_mul(0xFF51AFD7ED558CCD);
        x ^= x >> 33;
        x = x.wrapping_mul(0xC4CEB9FE1A85EC53);
        x ^= x >> 33;

        x
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
        let br = BlackRock::new(256, 0x123456, 2);

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
        let br = BlackRock::new(1000, 0xDEADBEEF, 3);

        for i in 0..1000 {
            let shuffled = br.shuffle(i);
            let unshuffled = br.unshuffle(shuffled);
            assert_eq!(i, unshuffled, "Unshuffle failed for index {}", i);
        }
    }

    #[test]
    fn test_blackrock_deterministic() {
        let br1 = BlackRock::new(512, 0xABCDEF, 2);
        let br2 = BlackRock::new(512, 0xABCDEF, 2);

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
        let br1 = BlackRock::new(100, 0x1111, 2);
        let br2 = BlackRock::new(100, 0x2222, 2);

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
        let br = BlackRock::new(1024, 0x777, 2);

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
        let br = BlackRock::new(1000, 0x999, 3);

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
