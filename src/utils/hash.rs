//! Hash utilities
//!
//! Provides helper functions for hashing Core Foundation and other types.

use std::hash::{self, DefaultHasher, Hasher};

/// Compute a hash value for any hashable type
///
/// This is a convenience wrapper around Rust's standard hashing.
///
/// # Examples
///
/// ```
/// use screencapturekit::utils::hash::hash;
///
/// let value = 42;
/// let hash_value = hash(&value);
/// assert_ne!(hash_value, 0);
/// ```
pub fn hash<T: hash::Hash>(t: T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
