use core_utils_rs::four_char_code::FourCharCode;

#[allow(non_camel_case_types)]
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

impl Into<FourCharCode> for PixelFormat {
    fn into(self) -> FourCharCode {
        match self {
            PixelFormat::BGRA => FourCharCode::from("BGRA"),
            PixelFormat::l10r => FourCharCode::from("l10r"),
            PixelFormat::YCbCr_420v => FourCharCode::from("420v"),
            PixelFormat::YCbCr_420f => FourCharCode::from("420f"),
        }
    }
}
impl From<u32> for PixelFormat {
    fn from(value: u32) -> PixelFormat {
        FourCharCode::from(value).into()
    }
}
impl Into<PixelFormat> for FourCharCode {
    fn into(self) -> PixelFormat {
        match self.display() {
            "BGRA" => PixelFormat::BGRA,
            "l10r" => PixelFormat::l10r,
            "420v" => PixelFormat::YCbCr_420v,
            "420f" => PixelFormat::YCbCr_420f,
            _ => unreachable!("Unknown pixel format: {}", self),
        }
    }
}
