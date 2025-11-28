//! Screen capture stream functionality
//!
//! This module provides the core streaming API for capturing screen content.
//!
//! ## Main Components
//!
//! - [`SCStream`] - The main capture stream
//! - [`configuration::SCStreamConfiguration`] - Stream configuration (resolution, FPS, etc.)
//! - [`content_filter::SCContentFilter`] - Filter for selecting what to capture
//! - [`output_trait::SCStreamOutputTrait`] - Trait for receiving captured frames
//! - [`output_type::SCStreamOutputType`] - Type of output (screen, audio)
//!
//! ## Example
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # let content = SCShareableContent::get().unwrap();
//! # let display = &content.displays()[0];
//! let filter = SCContentFilter::builder()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//! let config = SCStreamConfiguration::builder()
//!     .width(1920)
//!     .height(1080)
//!     .build();
//!
//! let mut stream = SCStream::new(&filter, &config);
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
pub use output_trait::SCStreamOutputTrait as SCStreamOutput;
pub use sc_stream::SCStream;
