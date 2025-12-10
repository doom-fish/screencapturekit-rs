//! Core Graphics types for screen coordinates and dimensions
//!
//! This module provides Rust equivalents of Core Graphics types used in
//! `ScreenCaptureKit` for representing screen coordinates, sizes, and rectangles.

mod point;
mod rect;
mod size;

pub use point::CGPoint;
pub use rect::CGRect;
pub use size::CGSize;

/// `CGDisplayID` type alias
pub type CGDisplayID = u32;
