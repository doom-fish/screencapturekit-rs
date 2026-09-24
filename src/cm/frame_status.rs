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
    Unknown(i32),
}

impl SCFrameStatus {
    /// Create from raw i32 value
    pub const fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Complete,
            1 => Self::Idle,
            2 => Self::Blank,
            3 => Self::Suspended,
            4 => Self::Started,
            5 => Self::Stopped,
            other => Self::Unknown(other),
        }
    }

    pub const fn raw(self) -> i32 {
        match self {
            Self::Complete => 0,
            Self::Idle => 1,
            Self::Blank => 2,
            Self::Suspended => 3,
            Self::Started => 4,
            Self::Stopped => 5,
            Self::Unknown(raw) => raw,
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
            Self::Unknown(raw) => write!(f, "Unknown({raw})"),
        }
    }
}
