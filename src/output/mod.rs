//! Frame buffer and pixel access
//!
//! This module provides types for accessing captured frame data at the pixel level.
//! All buffer types now use `std::io::Cursor` for data access, providing standard
//! `Read` and `Seek` traits for maximum compatibility.
//!
//! ## Main Types
//!
//! - [`CVPixelBuffer`] - Pixel buffer containing frame data
//! - [`IOSurface`] - IOSurface-backed buffer for zero-copy access
//! - [`PixelBufferLockGuard`] - RAII guard for locked pixel buffer access
//! - [`IOSurfaceLockGuard`] - RAII guard for locked `IOSurface` access
//! - [`PixelBufferCursorExt`] - Extension trait for pixel-specific cursor operations
//!
//! ## Examples
//!
//! ### Basic Usage with Cursor
//!
//! ```no_run
//! use std::io::{Read, Seek, SeekFrom};
//! use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags, PixelBufferCursorExt};
//!
//! # fn example(sample: screencapturekit::cm::CMSampleBuffer) -> Result<(), Box<dyn std::error::Error>> {
//! if let Some(pixel_buffer) = sample.get_image_buffer() {
//!     let guard = pixel_buffer.lock(PixelBufferLockFlags::ReadOnly)?;
//!     
//!     // Get a standard io::Cursor
//!     let mut cursor = guard.cursor();
//!     
//!     // Read pixels using standard Read trait
//!     let mut pixel = [0u8; 4];
//!     cursor.read_exact(&mut pixel)?;
//!     
//!     // Or use the extension trait for pixel operations
//!     cursor.seek(SeekFrom::Start(0))?;
//!     let pixel = cursor.read_pixel()?;
//!     
//!     // Seek to specific coordinates
//!     cursor.seek_to_pixel(50, 50, guard.bytes_per_row())?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Direct Slice Access
//!
//! ```no_run
//! use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags};
//!
//! # fn example(sample: screencapturekit::cm::CMSampleBuffer) -> Result<(), Box<dyn std::error::Error>> {
//! if let Some(pixel_buffer) = sample.get_image_buffer() {
//!     let guard = pixel_buffer.lock(PixelBufferLockFlags::ReadOnly)?;
//!     
//!     // Direct slice access (no cursor overhead)
//!     let data = guard.as_slice();
//!     println!("Frame size: {} bytes", data.len());
//!     
//!     // Access specific rows
//!     if let Some(row) = guard.row(100) {
//!         println!("Row 100 has {} bytes", row.len());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod iosurface;
pub mod pixel_buffer;

pub use crate::cm::{CMSampleBuffer, CMTime, CVPixelBuffer};
pub use iosurface::{IOSurface, IOSurfaceLockOptions, IOSurfaceLockGuard, CVPixelBufferIOSurface};
pub use pixel_buffer::{PixelBufferLockFlags, PixelBufferLockGuard, PixelBufferCursorExt, CVImageBufferLockExt};
