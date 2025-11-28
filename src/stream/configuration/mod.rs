mod internal;

pub mod advanced;
pub mod audio;
pub mod captured_elements;
pub mod captured_frames;
pub mod colors;
pub mod dimensions;
pub mod pixel_format;
pub mod stream_properties;
pub mod types;

pub use advanced::SCPresenterOverlayAlertSetting;
pub use internal::SCStreamConfiguration;
pub use pixel_format::PixelFormat;
pub use stream_properties::SCCaptureDynamicRange;
pub use types::{ConfigError, Point, Rect, Size};

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::internal_init()
    }
}

impl SCStreamConfiguration {
    /// Create a new stream configuration with default values
    ///
    /// This is equivalent to `SCStreamConfiguration::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
