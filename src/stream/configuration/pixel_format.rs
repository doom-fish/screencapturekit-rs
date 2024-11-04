use core::fmt;
use std::fmt::{Display, Formatter};

use core_utils_rs::four_char_code::FourCharCode;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    // Packed little endian ARGB8888.
    BGRA,
    // Packed little endian ARGB2101010.
    l10r,
    // Two-plane “video” range YCbCr 4:2:0.
    YCbCr_420v,
    // Two-plane “full” range YCbCr 4:2:0.
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
        match val {
            PixelFormat::BGRA => Self::from_str("BGRA").expect("should be valid four char"),
            PixelFormat::l10r => Self::from_str("l10r").expect("should be valid four char"),
            PixelFormat::YCbCr_420v => {
                Self::from_str("420v").expect("should be valid four char")
            }
            PixelFormat::YCbCr_420f => {
                Self::from_str("420f").expect("should be valid four char")
            }
        }
    }
}
impl From<u32> for PixelFormat {
    fn from(value: u32) -> Self {
        let c = FourCharCode::from_slice(&value.to_le_bytes()).expect("should be valid four char");
        c.into()
    }
}
impl From<FourCharCode> for PixelFormat {
    fn from(val: FourCharCode) -> Self {
        match val.display().to_string().as_str() {
            "BGRA" => Self::BGRA,
            "l10r" => Self::l10r,
            "420v" => Self::YCbCr_420v,
            "420f" => Self::YCbCr_420f,
            _ => unreachable!("Unknown pixel format"),
        }
    }
}
