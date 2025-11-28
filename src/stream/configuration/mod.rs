mod builder;
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
pub use builder::SCStreamConfigurationBuilder;
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
    /// let config = SCStreamConfiguration::builder()
    ///     .width(1920)
    ///     .height(1080)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> SCStreamConfigurationBuilder {
        SCStreamConfigurationBuilder::new()
    }

    /// Creates a new stream configuration builder (deprecated alias)
    #[deprecated(since = "1.1.0", note = "Use `builder()` instead")]
    pub fn build() -> Self {
        Self::internal_init()
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::internal_init()
    }
}
