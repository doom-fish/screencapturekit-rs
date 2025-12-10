//! Core Media types and wrappers
//!
//! This module provides Rust wrappers for Core Media framework types used in
//! screen capture operations.
//!
//! ## Main Types
//!
//! - [`CMSampleBuffer`] - Container for media samples (audio/video frames)
//! - [`CMTime`] - Time value with rational timescale
//! - [`CVPixelBuffer`] - Video pixel buffer
//! - [`IOSurface`] - Hardware-accelerated surface
//! - [`AudioBuffer`] - Audio data buffer
//! - [`AudioBufferList`] - Collection of audio buffers
//! - [`SCFrameStatus`] - Status of a captured frame

mod audio;
mod block_buffer;
pub mod ffi;
mod format_description;
mod frame_status;
mod iosurface;
mod pixel_buffer;
mod sample_buffer;
mod time;

// Re-export all public types
pub use audio::{
    AudioBuffer, AudioBufferList, AudioBufferListIter, AudioBufferListRaw, AudioBufferRef,
};
pub use block_buffer::CMBlockBuffer;
pub use format_description::CMFormatDescription;
pub use frame_status::SCFrameStatus;
pub use iosurface::{IOSurface, IOSurfaceLockGuard};
pub use pixel_buffer::{CVPixelBuffer, CVPixelBufferLockGuard, CVPixelBufferPool};
pub use sample_buffer::CMSampleBuffer;
pub use time::{CMClock, CMSampleTimingInfo, CMTime};

// Re-export codec and media type modules from format_description
pub use format_description::codec_types;
pub use format_description::media_types;
