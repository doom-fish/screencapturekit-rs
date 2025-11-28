//! # ScreenCaptureKit-rs
//!
//! Rust bindings for macOS `ScreenCaptureKit` framework.
//!
//! This crate provides safe, idiomatic Rust bindings for capturing screen content,
//! windows, and applications on macOS 12.3+.
//!
//! ## Features
//!
//! - **Screen and window capture** - Capture displays, windows, or applications
//! - **Audio capture** - System audio and microphone input
//! - **Real-time frame processing** - High-performance callbacks
//! - **Async support** - Runtime-agnostic async API (requires `async` feature)
//! - **Zero-copy GPU access** - IOSurface for Metal/OpenGL integration
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! // Get shareable content
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//!
//! // Create filter and configuration
//! let filter = SCContentFilter::builder()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//!
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! // Create and start stream
//! let mut stream = SCStream::new(&filter, &config);
//! stream.start_capture()?;
//! # Ok::<(), screencapturekit::error::SCError>(())
//! ```
//!
//! ## Configuration
//!
//! Use the builder pattern for fluent configuration:
//!
//! ```rust
//! use screencapturekit::prelude::*;
//!
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080)
//!     .with_pixel_format(PixelFormat::BGRA)
//!     .with_captures_audio(true)
//!     .with_sample_rate(48000)
//!     .with_channel_count(2)
//!     .with_shows_cursor(true);
//! ```
//!
//! ## Module Organization
//!
//! - [`stream`] - Stream configuration and management
//!   - [`stream::configuration`] - Stream settings (resolution, format, audio)
//!   - [`stream::content_filter`] - Filter for selecting capture content
//!   - [`stream::sc_stream`] - Main capture stream
//! - [`shareable_content`] - Display and window enumeration
//! - [`cm`] - Core Media types (`CMSampleBuffer`, `CMTime`, etc.)
//! - [`cg`] - Core Graphics types (`CGRect`, `CGSize`, etc.)
//! - [`output`] - Frame buffer and pixel access
//! - [`error`] - Error types and result aliases
//! - [`dispatch_queue`] - Custom dispatch queues for callbacks
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async` | Runtime-agnostic async API |
//! | `macos_13_0` | macOS 13.0+ APIs |
//! | `macos_14_0` | macOS 14.0+ APIs (content picker, screenshots) |
//! | `macos_15_0` | macOS 15.0+ APIs (recording output, HDR) |
//!
//! ## Platform Requirements
//!
//! - macOS 12.3+ (Monterey)
//! - Screen recording permission required

#![doc(html_root_url = "https://docs.rs/screencapturekit")]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

pub mod cg;
pub mod cm;
#[cfg(feature = "macos_14_0")]
pub mod content_sharing_picker;
pub mod dispatch_queue;
pub mod error;
pub mod ffi;
pub mod output;
#[cfg(feature = "macos_15_0")]
pub mod recording_output;
pub mod screenshot_manager;
pub mod shareable_content;
pub mod stream;
pub mod utils;

#[cfg(feature = "async")]
pub mod async_api;

// Re-export commonly used types
pub use cm::{
    codec_types, media_types, AudioBuffer, AudioBufferList, CMFormatDescription, CMSampleBuffer,
    CMSampleTimingInfo, CMTime, CVPixelBuffer, CVPixelBufferPool, IOSurface, SCFrameStatus,
};
pub use utils::four_char_code::FourCharCode;

/// Prelude module for convenient imports
///
/// Import everything you need with:
/// ```rust
/// use screencapturekit::prelude::*;
/// ```
pub mod prelude {
    pub use crate::cm::{CMSampleBuffer, CMTime};
    pub use crate::dispatch_queue::{DispatchQoS, DispatchQueue};
    pub use crate::error::{SCError, SCResult};
    pub use crate::shareable_content::{
        SCDisplay, SCRunningApplication, SCShareableContent, SCWindow,
    };
    pub use crate::stream::{
        configuration::{PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        sc_stream::SCStream,
        ErrorHandler,
    };
}
