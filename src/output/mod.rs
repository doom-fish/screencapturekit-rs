//! Frame buffer and pixel access
//!
//! This module provides types for accessing captured frame data at the pixel level.
//!
//! ## Main Types
//!
//! - [`CVPixelBuffer`] - Pixel buffer containing frame data
//! - [`IOSurface`] - IOSurface-backed buffer for zero-copy access
//! - [`PixelBufferLockGuard`] - RAII guard for locked pixel buffer access
//! - [`IOSurfaceLockGuard`] - RAII guard for locked IOSurface access
//!
//! ## Examples
//!
//! ```no_run
//! use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags};
//!
//! # fn example(sample: screencapturekit::cm::CMSampleBuffer) -> Result<(), Box<dyn std::error::Error>> {
//! if let Some(pixel_buffer) = sample.get_image_buffer() {
//!     let guard = pixel_buffer.lock(PixelBufferLockFlags::ReadOnly)?;
//!     let data = guard.as_slice();
//!     println!("Frame size: {} bytes", data.len());
//! }
//! # Ok(())
//! # }
//! ```

pub mod iosurface;
pub mod pixel_buffer;

pub use crate::cm::{CMSampleBuffer, CMTime, CVPixelBuffer};
pub use iosurface::{IOSurface, IOSurfaceLockOptions, IOSurfaceLockGuard, CVPixelBufferIOSurface};
pub use pixel_buffer::{PixelBufferLockFlags, PixelBufferLockGuard, BufferCursor, CVImageBufferLockExt};
