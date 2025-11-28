//! # ScreenCaptureKit-rs
//!
//! Rust bindings for macOS `ScreenCaptureKit` framework.
//!
//! This crate provides safe, idiomatic Rust bindings for capturing screen content,
//! windows, and applications on macOS 12.3+.
//!
//! ## Features
//!
//! - Screen and window capture
//! - Audio capture
//! - Real-time frame processing
//! - Configurable capture settings
//! - Zero external dependencies (uses custom Swift bridge)
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
//! let mut config = SCStreamConfiguration::default();
//! config.set_width(1920);
//! config.set_height(1080);
//!
//! // Create and start stream
//! let mut stream = SCStream::new(&filter, &config);
//! stream.start_capture()?;
//! # Ok::<(), screencapturekit::error::SCError>(())
//! ```
//!
//! ## Module Organization
//!
//! - [`cm`] - Core Media types (`CMSampleBuffer`, `CMTime`, etc.)
//! - [`cg`] - Core Graphics types (`CGRect`, `CGSize`, etc.)
//! - [`stream`] - Stream configuration and management
//! - [`shareable_content`] - Display and window information
//! - [`output`] - Frame buffer and pixel access
//! - [`error`] - Error types
//! - [`utils`] - Utility functions

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
