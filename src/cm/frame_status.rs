//! Frame status for captured screen content

use std::fmt;

/// Frame status for captured screen content
///
/// Indicates the state of a frame captured by `ScreenCaptureKit`.
/// This maps to Apple's `SCFrameStatus` enum.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCFrameStatus {
    /// Frame contains complete content
    #[default]
    Complete = 0,
    /// Frame is idle (no changes)
    Idle = 1,
    /// Frame is blank
    Blank = 2,
    /// Frame is suspended
    Suspended = 3,
    /// Started (first frame)
    Started = 4,
    /// Stopped (last frame)
    Stopped = 5,
}

impl SCFrameStatus {
    /// Create from raw i32 value
    pub const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Complete),
            1 => Some(Self::Idle),
            2 => Some(Self::Blank),
            3 => Some(Self::Suspended),
            4 => Some(Self::Started),
            5 => Some(Self::Stopped),
            _ => None,
        }
    }

    /// Returns true if the frame contains actual content
    pub const fn has_content(self) -> bool {
        matches!(self, Self::Complete | Self::Started)
    }

    /// Returns true if the frame is complete
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl fmt::Display for SCFrameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "Complete"),
            Self::Idle => write!(f, "Idle"),
            Self::Blank => write!(f, "Blank"),
            Self::Suspended => write!(f, "Suspended"),
            Self::Started => write!(f, "Started"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Keys for accessing frame information from `CMSampleBuffer` attachments
///
/// These keys correspond to Apple's `SCStreamFrameInfo` struct and can be used
/// to retrieve metadata about captured frames from the sample buffer attachments.
///
/// # Example
/// ```rust,ignore
/// use screencapturekit::cm::{CMSampleBuffer, SCStreamFrameInfoKey};
///
/// fn process_frame(buffer: &CMSampleBuffer) {
///     // Frame info is typically accessed via the buffer's status method
///     if let Some(status) = buffer.frame_status() {
///         println!("Frame status: {:?}", status);
///     }
/// }
/// ```
pub struct SCStreamFrameInfoKey;

impl SCStreamFrameInfoKey {
    /// Key for the frame status (`SCFrameStatus`)
    pub const STATUS: &'static str = "SCStreamFrameInfoStatus";

    /// Key for the display time (mach absolute time)
    pub const DISPLAY_TIME: &'static str = "SCStreamFrameInfoDisplayTime";

    /// Key for the scale factor (point-to-pixel ratio)
    pub const SCALE_FACTOR: &'static str = "SCStreamFrameInfoScaleFactor";

    /// Key for the content scale
    pub const CONTENT_SCALE: &'static str = "SCStreamFrameInfoContentScale";

    /// Key for the content rectangle
    pub const CONTENT_RECT: &'static str = "SCStreamFrameInfoContentRect";

    /// Key for the bounding rectangle
    pub const BOUNDING_RECT: &'static str = "SCStreamFrameInfoBoundingRect";

    /// Key for the screen rectangle
    pub const SCREEN_RECT: &'static str = "SCStreamFrameInfoScreenRect";

    /// Key for dirty rectangles (areas that changed)
    pub const DIRTY_RECTS: &'static str = "SCStreamFrameInfoDirtyRects";

    /// Key for the presenter overlay content rectangle (macOS 14.0+)
    pub const PRESENTER_OVERLAY_CONTENT_RECT: &'static str =
        "SCStreamFrameInfoPresenterOverlayContentRect";
}
