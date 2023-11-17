use crate::sc_types::base::CMTime;
use crate::sc_types::four_char_code::FourCharCode;
use crate::sc_types::geometry::CGRect;
use crate::sc_types::graphics::CGColor;
use screencapturekit_sys::{
    os_types::rc::Id,
    stream_configuration::{UnsafeStreamConfiguration, UnsafeStreamConfigurationRef},
};

pub static PIXEL_FORMATS: [PixelFormat; 4] = [
    PixelFormat::ARGB8888,
    PixelFormat::ARGB2101010,
    PixelFormat::YCbCr420f,
    PixelFormat::YCbCr420v,
];

#[derive(Copy, Clone, Debug, Default)]
pub enum PixelFormat {
    ARGB8888,
    ARGB2101010,
    #[default]
    YCbCr420v,
    YCbCr420f,
}

impl From<FourCharCode> for PixelFormat {
    fn from(val: FourCharCode) -> Self {
        let code_str = val.to_string();
        match code_str.as_str() {
            "BGRA" => PixelFormat::ARGB8888,
            "l10r" => PixelFormat::ARGB2101010,
            "420v" => PixelFormat::YCbCr420v,
            "420f" => PixelFormat::YCbCr420f,
            _ => unreachable!(),
        }
    }
}
impl From<PixelFormat> for FourCharCode {
    fn from(val: PixelFormat) -> Self {
        match val {
            PixelFormat::ARGB8888 => FourCharCode::from_chars(*b"BGRA"),
            PixelFormat::ARGB2101010 => FourCharCode::from_chars(*b"l10r"),
            PixelFormat::YCbCr420v => FourCharCode::from_chars(*b"420v"),
            PixelFormat::YCbCr420f => FourCharCode::from_chars(*b"420f"),
        }
    }
}

#[derive(Debug, Default)]
pub struct SCStreamConfiguration {
    //   The width of the output.
    width: Option<u32>,
    //   The height of the output.
    height: Option<u32>,
    // A boolean value that indicates whether to scale the output to fit the configured width and height.
    scales_to_fit: Option<bool>,
    // A rectangle that specifies the source area to capture.
    source_rect: Option<CGRect>,
    // A rectangle that specifies a destination into which to write the output.
    destination_rect: Option<CGRect>,
    // A boolean value that determines whether the cursor is visible in the stream.
    shows_cursor: Option<bool>,
    // Optimizing Performance
    // The maximum number of frames for the queue to store.
    queue_depth: Option<u32>,
    // The desired minimum time between frame updates, in seconds.
    minimum_frame_interval: Option<CMTime>,
    // Configuring Audi
    // A boolean value that indicates whether to capture audio.
    captures_audio: Option<bool>,
    // The sample rate for audio capture.
    sample_rate: Option<u32>,
    // The number of audio channels to capture.
    channel_count: Option<u32>,
    // A boolean value that indicates whether to exclude a
    excludes_current_process_audio: Option<bool>,
    // Configuring Colors
    // A pixel format for sample buffers that a stream outputs.
    pixel_format: Option<PixelFormat>,
    // A color matrix to apply to the output surface.
    color_matrix: Option<String>,
    // A color space to use for the output buffer.
    color_space_name: Option<String>,
    // A background color for the output.
    // Controlling Visibility
    background_color: Option<CGColor>,
}

impl SCStreamConfiguration {
    pub fn empty() -> Self {
        Default::default()
    }
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
    pub fn scales_to_fit(mut self, scales_to_fit: bool) -> Self {
        self.scales_to_fit = Some(scales_to_fit);
        self
    }
    pub fn source_rect(mut self, source_rect: CGRect) -> Self {
        self.source_rect = Some(source_rect);
        self
    }
    pub fn destination_rect(mut self, destination_rect: CGRect) -> Self {
        self.destination_rect = Some(destination_rect);
        self
    }
    pub fn pixel_format(mut self, pixel_format: PixelFormat) -> Self {
        self.pixel_format = Some(pixel_format);
        self
    }
    pub fn color_matrix(mut self, color_matrix: String) -> Self {
        self.color_matrix = Some(color_matrix);
        self
    }
    pub fn color_space_name(mut self, color_space_name: String) -> Self {
        self.color_space_name = Some(color_space_name);
        self
    }
    pub fn background_color(mut self, background_color: CGColor) -> Self {
        self.background_color = Some(background_color);
        self
    }

    pub fn shows_cursor(mut self, shows_cursor: bool) -> Self {
        self.shows_cursor = Some(shows_cursor);
        self
    }

    pub fn queue_depth(mut self, queue_depth: u32) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }
    pub fn minimum_frame_interval(mut self, minimum_frame_interval: CMTime) -> Self {
        self.minimum_frame_interval = Some(minimum_frame_interval);
        self
    }

    pub fn captures_audio(mut self, captures_audio: bool) -> Self {
        self.captures_audio = Some(captures_audio);
        self
    }
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }
    pub fn channel_count(mut self, channel_count: u32) -> Self {
        self.channel_count = Some(channel_count);
        self
    }
    pub fn excludes_current_process_audio(mut self, excludes_current_process_audio: bool) -> Self {
        self.excludes_current_process_audio = Some(excludes_current_process_audio);
        self
    }
}

impl From<SCStreamConfiguration> for UnsafeStreamConfiguration {
    fn from(value: SCStreamConfiguration) -> Self {
        UnsafeStreamConfiguration {
            width: value.width,
            height: value.height,
            scales_to_fit: value.scales_to_fit.map(|x| x as _),
            source_rect: value.source_rect,
            destination_rect: value.destination_rect,
            pixel_format: value.pixel_format.map(|x| x.into()),
            color_matrix: value.color_matrix,
            color_space_name: value.color_space_name,
            background_color: value.background_color,
            shows_cursor: value.shows_cursor.map(|x| x as _),
            queue_depth: value.queue_depth,
            minimum_frame_interval: value.minimum_frame_interval,
            captures_audio: value.captures_audio.map(|x| x as _),
            sample_rate: value.sample_rate,
            channel_count: value.channel_count,
            excludes_current_process_audio: value.excludes_current_process_audio.map(|x| x as _),
        }
    }
}

impl From<SCStreamConfiguration> for Id<UnsafeStreamConfigurationRef> {
    fn from(value: SCStreamConfiguration) -> Self {
        let unsafe_config: UnsafeStreamConfiguration = value.into();
        unsafe_config.into()
    }
}

#[cfg(test)]
mod get_configuration {

    use super::*;
    #[test]
    fn test_configuration() {
        SCStreamConfiguration::empty().size(100, 100);
    }
}
