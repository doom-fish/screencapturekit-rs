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
#[allow(clippy::module_name_repetitions)]
pub trait SCStreamOutputTrait: Send {
    /// Called when a new sample buffer is available
    ///
    /// # Parameters
    ///
    /// - `sample_buffer`: The captured sample (video frame or audio buffer)
    /// - `of_type`: Type of output (Screen, Audio, or Microphone)
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}
