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
//! let filter = SCContentFilter::build()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//! let config = SCStreamConfiguration::build()
//!     .set_width(1920)?
//!     .set_height(1080)?;
//!
//! let mut stream = SCStream::new(&filter, &config);
//! stream.start_capture()?;
//! # Ok::<(), screencapturekit::error::SCError>(())
//! ```

pub mod content_filter;
pub mod sc_stream;
pub mod configuration;
pub mod delegate_trait;
pub mod output_trait;
pub mod output_type;

pub use sc_stream::SCStream;
pub use output_trait::SCStreamOutputTrait as SCStreamOutput;
pub use delegate_trait::SCStreamDelegateTrait as SCStreamDelegate;

