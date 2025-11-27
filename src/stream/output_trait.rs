//! Output handler trait for stream callbacks
//!
//! Defines the interface for receiving captured frames and audio buffers.

use crate::cm::CMSampleBuffer;

use super::output_type::SCStreamOutputType;

/// Trait for handling stream output
///
/// Implement this trait to receive callbacks when the stream captures frames or audio.
///
/// # Examples
///
/// ## Using a struct
///
/// ```
/// use screencapturekit::stream::{
///     output_trait::SCStreamOutputTrait,
///     output_type::SCStreamOutputType,
/// };
/// use screencapturekit::cm::CMSampleBuffer;
///
/// struct MyHandler;
///
/// impl SCStreamOutputTrait for MyHandler {
///     fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
///         match of_type {
///             SCStreamOutputType::Screen => {
///                 println!("Received video frame");
///             }
///             SCStreamOutputType::Audio => {
///                 println!("Received audio buffer");
///             }
///             SCStreamOutputType::Microphone => {
///                 println!("Received microphone audio");
///             }
///         }
///     }
/// }
/// ```
///
/// ## Using a closure
///
/// Closures that match `Fn(CMSampleBuffer, SCStreamOutputType)` automatically
/// implement this trait:
///
/// ```rust,no_run
/// use screencapturekit::prelude::*;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let content = SCShareableContent::get()?;
/// # let display = &content.displays()[0];
/// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// # let config = SCStreamConfiguration::builder().build();
/// let mut stream = SCStream::new(&filter, &config);
///
/// stream.add_output_handler(
///     |_sample, _output_type| println!("Got frame!"),
///     SCStreamOutputType::Screen
/// );
/// # Ok(())
/// # }
/// ```
pub trait SCStreamOutputTrait: Send {
    /// Called when a new sample buffer is available
    ///
    /// # Parameters
    ///
    /// - `sample_buffer`: The captured sample (video frame or audio buffer)
    /// - `of_type`: Type of output (Screen, Audio, or Microphone)
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}

/// Blanket implementation for closures
///
/// Any closure matching `Fn(CMSampleBuffer, SCStreamOutputType) + Send + 'static`
/// can be used directly as an output handler.
impl<F> SCStreamOutputTrait for F
where
    F: Fn(CMSampleBuffer, SCStreamOutputType) + Send + 'static,
{
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        self(sample_buffer, of_type);
    }
}
