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
pub trait SCStreamDelegateTrait: Send {
    /// Called when video effects start (macOS 14.0+)
    ///
    /// Notifies when the stream's overlay video effect (presenter overlay) has started.
    fn output_video_effect_did_start_for_stream(&self) {}

    /// Called when video effects stop (macOS 14.0+)
    ///
    /// Notifies when the stream's overlay video effect (presenter overlay) has stopped.
    fn output_video_effect_did_stop_for_stream(&self) {}

    /// Called when the stream becomes active (macOS 15.2+)
    ///
    /// Notifies the first time any window that was being shared in the stream
    /// is re-opened after all the windows being shared were closed.
    /// When all the windows being shared are closed, the client will receive
    /// `stream_did_become_inactive`.
    fn stream_did_become_active(&self) {}

    /// Called when the stream becomes inactive (macOS 15.2+)
    ///
    /// Notifies when all the windows that are currently being shared are exited.
    /// This callback occurs for all content filter types.
    fn stream_did_become_inactive(&self) {}

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
/// # let config = SCStreamConfiguration::default();
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
