//! Pixel format enumeration
//!
//! Defines the available pixel formats for captured frames.

use core::fmt;
use std::fmt::{Display, Formatter};

use crate::utils::four_char_code::FourCharCode;

/// Pixel format for captured video frames
///
/// Specifies the layout and encoding of pixel data in captured frames.
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::configuration::PixelFormat;
///
/// let format = PixelFormat::BGRA;
/// println!("Format: {}", format); // Prints "BGRA"
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Packed little endian ARGB8888 (most common)
    BGRA,
    /// Packed little endian ARGB2101010 (10-bit color)
    l10r,
    /// Two-plane "video" range YCbCr 4:2:0
    YCbCr_420v,
    /// Two-plane "full" range YCbCr 4:2:0
    YCbCr_420f,
}
impl Display for PixelFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let c: FourCharCode = (*self).into();
        write!(f, "{}", c.display())
    }
}

impl From<PixelFormat> for FourCharCode {
    fn from(val: PixelFormat) -> Self {
        // Use infallible byte array constructor for compile-time constants
        match val {
            PixelFormat::BGRA => Self::from_bytes(*b"BGRA"),
            PixelFormat::l10r => Self::from_bytes(*b"l10r"),
            PixelFormat::YCbCr_420v => Self::from_bytes(*b"420v"),
            PixelFormat::YCbCr_420f => Self::from_bytes(*b"420f"),
        }
    }
}
impl From<u32> for PixelFormat {
    fn from(value: u32) -> Self {
        // Use infallible byte array constructor
        let c = FourCharCode::from_bytes(value.to_le_bytes());
        c.into()
    }
}
impl From<FourCharCode> for PixelFormat {
    fn from(val: FourCharCode) -> Self {
        match val.display().as_str() {
            "BGRA" => Self::BGRA,
            "l10r" => Self::l10r,
            "420v" => Self::YCbCr_420v,
            "420f" => Self::YCbCr_420f,
            _ => unreachable!("Unknown pixel format"),
        }
    }
}
