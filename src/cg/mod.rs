//! CoreGraphics value types — re-exported from `apple-cf` so that downstream
//! crates and other doom-fish bindings see the same `CGRect` / `CGPoint` /
//! `CGSize` types.
//!
//! This module used to vendor its own copies; the canonical implementations
//! now live in `apple_cf::cg` and this re-export preserves the
//! `screencapturekit::cg::CGRect` (etc.) public path for backward compatibility.

pub use apple_cf::cg::{CGPoint, CGRect, CGSize};
