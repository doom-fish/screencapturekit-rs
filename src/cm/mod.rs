//! Core Media types and wrappers
//!
//! This module provides Rust wrappers for Core Media framework types used in
//! screen capture operations.
//!
//! ## Main Types
//!
//! - [`CMSampleBuffer`] - Container for media samples (audio/video frames)
//! - [`CMTime`] - Time value with rational timescale for precise timing
//! - [`IOSurface`] - Hardware-accelerated surface for zero-copy GPU access
//! - [`CMBlockBuffer`] - Block of contiguous data (audio/compressed video)
//! - [`AudioBuffer`] - Audio data buffer with sample data
//! - [`AudioBufferList`] - Collection of audio buffers for multi-channel audio
//! - [`SCFrameStatus`] - Status of a captured frame (complete, idle, dropped, etc.)
//!
//! ## Example
//!
//! ```rust,no_run
//! use screencapturekit::cm::{CMSampleBuffer, CMTime, SCFrameStatus};
//!
//! fn process_frame(sample: CMSampleBuffer) {
//!     // Check frame status
//!     if sample.frame_status() == Some(SCFrameStatus::Complete) {
//!         // Get timestamp
//!         let pts = sample.presentation_timestamp();
//!         println!("Frame at {:?}", pts);
//!
//!         // Access pixel buffer for CPU processing
//!         if let Some(pixel_buffer) = sample.image_buffer() {
//!             // Access IOSurface for GPU processing
//!             if let Some(surface) = pixel_buffer.io_surface() {
//!                 println!("Surface: {}x{}", surface.width(), surface.height());
//!             }
//!         }
//!     }
//! }
//! ```

mod audio;
mod block_buffer;
pub mod ffi;
mod format_description;
mod frame_status;
pub mod iosurface;
mod sample_buffer;
mod time;

// Re-export all public types
pub use audio::{
    AudioBuffer, AudioBufferList, AudioBufferListIter, AudioBufferListRaw, AudioBufferRef,
};
pub use block_buffer::CMBlockBuffer;
pub use format_description::CMFormatDescription;
pub use frame_status::SCFrameStatus;
pub use iosurface::{IOSurface, IOSurfaceLockGuard, IOSurfaceLockOptions, PlaneProperties};
pub use sample_buffer::CMSampleBuffer;
pub use time::{CMClock, CMSampleTimingInfo, CMTime};

// Re-export codec and media type modules from format_description
pub use format_description::codec_types;
pub use format_description::media_types;
