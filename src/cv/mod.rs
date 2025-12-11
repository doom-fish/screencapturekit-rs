//! Core Video types and wrappers
//!
//! This module provides Rust wrappers for Core Video framework types used in
//! screen capture operations.
//!
//! ## Main Types
//!
//! - [`CVPixelBuffer`] - Video pixel buffer
//! - [`CVPixelBufferPool`] - Pool for reusing pixel buffers
//! - [`CVPixelBufferLockGuard`] - RAII guard for locked pixel buffer access
//! - [`CVPixelBufferLockFlags`] - Lock flags for pixel buffer access
//! - [`PixelBufferCursorExt`] - Extension trait for cursor-based pixel access

pub mod ffi;
mod pixel_buffer;

pub use pixel_buffer::{
    CVPixelBuffer, CVPixelBufferLockFlags, CVPixelBufferLockGuard, CVPixelBufferPool,
    PixelBufferCursorExt,
};
