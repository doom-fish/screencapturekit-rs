//! Output type enumeration for stream handlers
//!
//! Defines the types of output that can be received from a capture stream.

use std::fmt::{self, Display};

/// Type of output received from a capture stream
///
/// Used to distinguish between different types of captured data
/// when implementing [`SCStreamOutputTrait`](crate::stream::output_trait::SCStreamOutputTrait).
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::output_type::SCStreamOutputType;
///
/// fn handle_output(output_type: SCStreamOutputType) {
///     match output_type {
///         SCStreamOutputType::Screen => println!("Video frame"),
///         SCStreamOutputType::Audio => println!("Audio buffer"),
///         SCStreamOutputType::Microphone => println!("Microphone audio"),
///     }
/// }
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
#[repr(C)]
pub enum SCStreamOutputType {
    /// Video frame output
    Screen,
    /// System audio output
    Audio,
    /// Microphone audio output (macOS 15.0+)
    ///
    /// When using microphone capture, this output type allows separate handling
    /// of microphone audio from system audio.
    Microphone,
}

impl Display for SCStreamOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Screen => write!(f, "Screen"),
            Self::Audio => write!(f, "Audio"),
            Self::Microphone => write!(f, "Microphone"),
        }
    }
}
