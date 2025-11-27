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
impl SCStreamConfiguration {
    /// Creates a new stream configuration builder
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use screencapturekit::stream::configuration::SCStreamConfiguration;
    /// 
    /// let config = SCStreamConfiguration::build()
    ///     .set_width(1920)
    ///     .unwrap()
    ///     .set_height(1080)
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn build() -> Self {
        Self::internal_init()
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::build()
    }
}
