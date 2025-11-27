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

/// A simple error handler wrapper for closures
///
/// Allows using a closure as a stream delegate that only handles errors.
///
/// # Examples
///
/// ```rust,no_run
/// use screencapturekit::prelude::*;
/// use screencapturekit::stream::delegate_trait::ErrorHandler;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let content = SCShareableContent::get()?;
/// # let display = &content.displays()[0];
/// # let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();
/// # let config = SCStreamConfiguration::build();
///
/// let error_handler = ErrorHandler::new(|error| {
///     eprintln!("Stream error: {}", error);
/// });
///
/// let stream = SCStream::new_with_delegate(&filter, &config, error_handler);
/// # Ok(())
/// # }
/// ```
pub struct ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    handler: F,
}

impl<F> ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    /// Create a new error handler from a closure
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> SCStreamDelegateTrait for ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    fn did_stop_with_error(&self, error: SCError) {
        (self.handler)(error);
    }
}
