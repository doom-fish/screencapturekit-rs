//! GCD dispatch queue wrapper — re-exported from `apple-cf` to share the
//! same `DispatchQueue` type with the rest of the doom-fish suite.

pub use apple_cf::dispatch_queue::{DispatchQoS, DispatchQueue};
