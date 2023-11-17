use objc::{msg_send, runtime::Class, *};

use objc_foundation::INSObject;
use objc_id::Id;

use crate::os_types::{
    base::{CMTime, OSType, UInt32, BOOL},
    geometry::CGRect,
    graphics::CGColor,
};

#[derive(Debug)]
pub struct UnsafeStreamConfigurationRef;
unsafe impl Message for UnsafeStreamConfigurationRef {}
impl From<UnsafeStreamConfiguration> for Id<UnsafeStreamConfigurationRef> {
    fn from(config: UnsafeStreamConfiguration) -> Self {
        let unsafe_ref = UnsafeStreamConfigurationRef::new();
        unsafe {
            // Specifying dimensions
            if let Some(width) = config.width {
                let _: () = msg_send![unsafe_ref, setWidth: width];
            }
            if let Some(height) = config.height {
                let _: () = msg_send![unsafe_ref, setHeight: height];
            }
            if let Some(scales_to_fit) = config.scales_to_fit {
                let _: () = msg_send![unsafe_ref, scalesToFit: scales_to_fit];
            }
            if let Some(source_rect) = config.source_rect {
                let _: () = msg_send![unsafe_ref, setSourceRect: source_rect];
            }
            if let Some(destination_rect) = config.destination_rect {
                let _: () = msg_send![unsafe_ref, setDestinationRect: destination_rect];
            }

            // Configuring colors
            if let Some(pixel_format) = config.pixel_format {
                let _: () = msg_send![unsafe_ref, setPixelFormat: pixel_format];
            }
            if let Some(color_matrix) = config.color_matrix {
                let _: () = msg_send![unsafe_ref, setColorMatrix: color_matrix];
            }
            if let Some(color_space_name) = config.color_space_name {
                let _: () = msg_send![unsafe_ref, setColorSpaceName: color_space_name];
            }
            if let Some(background_color) = config.background_color {
                let _: () = msg_send![unsafe_ref, setBackgroundColor: background_color];
            }

            // Controlling visibility
            if let Some(shows_cursor) = config.shows_cursor {
                let _: () = msg_send![unsafe_ref, setShowsCursor: shows_cursor];
            }

            // Optimizing Performance
            if let Some(queue_depth) = config.queue_depth {
                let _: () = msg_send![unsafe_ref, setQueueDepth: queue_depth];
            }
            if let Some(minimum_frame_interval) = config.minimum_frame_interval {
                let _: () = msg_send![unsafe_ref, setMinimumFrameInterval: minimum_frame_interval];
            }

            // Configuring audio
            if let Some(captures_audio) = config.captures_audio {
                let _: () = msg_send![unsafe_ref, setCapturesAudio: captures_audio];
            }
            if let Some(sample_rate) = config.sample_rate {
                let _: () = msg_send![unsafe_ref, setSampleRate: sample_rate];
            }
            if let Some(channel_count) = config.channel_count {
                let _: () = msg_send![unsafe_ref, setChannelCount: channel_count];
            }
            if let Some(excludes_current_process_audio) = config.excludes_current_process_audio {
                let _: () = msg_send![unsafe_ref, setExcludesCurrentProcessAudio: excludes_current_process_audio];
            }

            // instance properties
            // TODO: To be defined: see https://developer.apple.com/documentation/screencapturekit/scstreamconfiguration?language=objc
            // if let Some(capture_resolution) = config.capture_resolution {
            //     let _: () = msg_send![unsafe_ref, setExcludesCurrentProcessAudio: excludes_current_process_audio];
            // }

        }

        unsafe_ref
    }
}

impl INSObject for UnsafeStreamConfigurationRef {
    fn class() -> &'static Class {
        Class::get("SCStreamConfiguration")
                .expect("Missing SCStreamConfiguration class, check that the binary is linked with ScreenCaptureKit")
    }
}

#[derive(Debug, Default)]
pub struct UnsafeStreamConfiguration {
    // The width of the output.
    pub width: Option<UInt32>,
    //   The height of the output.
    pub height: Option<UInt32>,
    // A boolean value that indicates whether to scale the output to fit the configured width and height.
    pub scales_to_fit: Option<BOOL>,
    // A rectangle that specifies the source area to capture.
    pub source_rect: Option<CGRect>,
    // A rectangle that specifies a destination into which to write the output.
    pub destination_rect: Option<CGRect>,
    // Configuring Colors

    // A pixel format for sample buffers that a stream outputs.
    pub pixel_format: Option<OSType>,
    // A color matrix to apply to the output surface.
    pub color_matrix: Option<String>,
    // A color space to use for the output buffer.
    pub color_space_name: Option<String>,
    // A background color for the output.
    // Controlling Visibility
    pub background_color: Option<CGColor>,

    // A boolean value that determines whether the cursor is visible in the stream.
    pub shows_cursor: Option<BOOL>,
    // Optimizing Performance
    // The maximum number of frames for the queue to store.
    pub queue_depth: Option<UInt32>,
    // The desired minimum time between frame updates>, in seconds.
    pub minimum_frame_interval: Option<CMTime>,
    // Configuring Audio
    // A boolean value that indicates whether to capture audio.
    pub captures_audio: Option<BOOL>,
    // The sample rate for audio capture.
    pub sample_rate: Option<UInt32>,
    // The number of audio channels to capture.
    pub channel_count: Option<UInt32>,
    // A boolean value that indicates whether to exclude a
    pub excludes_current_process_audio: Option<BOOL>,
}

#[cfg(test)]
mod get_shareable_content {

    use super::*;
    #[test]
    fn test_from() {
        let _: Id<UnsafeStreamConfigurationRef> = UnsafeStreamConfiguration::default().into();
    }
}
