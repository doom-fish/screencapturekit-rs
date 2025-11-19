//! Delegate trait for stream lifecycle events
//!
//! Defines the interface for receiving stream state change notifications.

use crate::error::SCError;

/// Trait for handling stream lifecycle events
///
/// Implement this trait to receive notifications about stream state changes,
/// errors, and video effects.
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::delegate_trait::SCStreamDelegateTrait;
/// use screencapturekit::error::SCError;
///
/// struct MyDelegate;
///
/// impl SCStreamDelegateTrait for MyDelegate {
///     fn stream_did_stop(&self, error: Option<String>) {
///         if let Some(err) = error {
///             eprintln!("Stream stopped with error: {}", err);
///         } else {
///             println!("Stream stopped normally");
///         }
///     }
///
///     fn did_stop_with_error(&self, error: SCError) {
///         eprintln!("Stream error: {}", error);
///     }
/// }
/// ```
#[allow(clippy::module_name_repetitions)]
pub trait SCStreamDelegateTrait: Send {
    /// Called when video effects start
    fn output_video_effect_did_start_for_stream(&self) {}
    
    /// Called when video effects stop
    fn output_video_effect_did_stop_for_stream(&self) {}
    
    /// Called when stream stops with an error
    fn did_stop_with_error(&self, _error: SCError) {}
    
    /// Called when stream stops
    ///
    /// # Parameters
    ///
    /// - `error`: Optional error message if the stream stopped due to an error
    fn stream_did_stop(&self, _error: Option<String>) {}
}
