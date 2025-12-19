//! Screen capture stream functionality
//!
//! This module provides the core streaming API for capturing screen content.
//!
//! ## Main Components
//!
//! - [`SCStream`] - The main capture stream that manages the capture session
//! - [`configuration::SCStreamConfiguration`] - Stream configuration (resolution, FPS, pixel format, audio)
//! - [`content_filter::SCContentFilter`] - Filter for selecting what to capture (display, window, app)
//! - [`output_trait::SCStreamOutputTrait`] - Trait for receiving captured frames
//! - [`output_type::SCStreamOutputType`] - Type of output (screen, audio, microphone)
//! - [`delegate_trait::SCStreamDelegateTrait`] - Trait for stream lifecycle events
//!
//! ## Workflow
//!
//! 1. Query available content with [`SCShareableContent`](crate::shareable_content::SCShareableContent)
//! 2. Create a content filter with [`SCContentFilter::create()`](content_filter::SCContentFilter::create)
//! 3. Configure the stream with [`SCStreamConfiguration::new()`](configuration::SCStreamConfiguration::new)
//! 4. Create and start the stream with [`SCStream::new()`](SCStream::new)
//!
//! ## Example
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # let content = SCShareableContent::get().unwrap();
//! # let display = &content.displays()[0];
//! let filter = SCContentFilter::create()
//!     .with_display(display)
//!     .with_excluding_windows(&[])
//!     .build();
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_output_handler(
//!     |sample, output_type| println!("Got frame!"),
//!     SCStreamOutputType::Screen
//! );
//! stream.start_capture()?;
//! # Ok::<(), screencapturekit::error::SCError>(())
//! ```

pub mod configuration;
pub mod content_filter;
pub mod delegate_trait;
pub mod output_trait;
pub mod output_type;
pub mod sc_stream;

pub use delegate_trait::ErrorHandler;
pub use delegate_trait::SCStreamDelegateTrait as SCStreamDelegate;
pub use delegate_trait::StreamCallbacks;
pub use output_trait::SCStreamOutputTrait as SCStreamOutput;
pub use sc_stream::SCStream;

#[cfg(feature = "macos_14_0")]
pub use content_filter::{SCShareableContentStyle, SCStreamType};
