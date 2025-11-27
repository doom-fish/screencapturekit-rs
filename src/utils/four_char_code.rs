//! Minimal FourCharCode implementation for pixel formats and color conversions
//!
//! A FourCharCode is a 4-byte code used in Core Video and Core Media to identify
//! pixel formats, codecs, and other media types.

use std::fmt;
use std::str::FromStr;

/// FourCharCode represents a 4-character code (used in Core Video/Media)
///
/// # Examples
///
/// ```
/// use screencapturekit::FourCharCode;
///
/// // Create from string
/// let code: FourCharCode = "BGRA".parse().unwrap();
/// assert_eq!(code.display(), "BGRA");
///
/// // Create from bytes
/// let code = FourCharCode::from_bytes(*b"420v");
/// assert_eq!(code.as_u32(), 0x34323076);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCharCode(u32);

impl FourCharCode {
    /// Create a FourCharCode from exactly 4 bytes (infallible)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::FourCharCode;
    ///
    /// let code = FourCharCode::from_bytes(*b"BGRA");
    /// assert_eq!(code.display(), "BGRA");
    /// ```
    #[inline]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }

    /// Create a FourCharCode from a byte slice
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 4 {
            return None;
        }
        
        let code = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Some(Self(code))
    }

    /// Get the u32 representation
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::FourCharCode;
    ///
    /// let code = FourCharCode::from_bytes(*b"BGRA");
    /// let value: u32 = code.as_u32();
    /// assert_eq!(value, 0x42475241);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Get the bytes as an array
    #[inline]
    pub const fn as_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Create from a u32 value (const version of From trait)
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        Self(value)
    }

    /// Compare with another FourCharCode at compile time
    #[inline]
    pub const fn equals(self, other: Self) -> bool {
        self.0 == other.0
    }

    /// Display the code as a string
    pub fn display(self) -> String {
        let bytes = self.0.to_be_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }
}

impl FromStr for FourCharCode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err("FourCharCode must be exactly 4 characters");
        }
        if !s.is_ascii() {
            return Err("FourCharCode must contain only ASCII characters");
        }
        
        let bytes = s.as_bytes();
        let code = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(Self(code))
    }
}

impl From<u32> for FourCharCode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<FourCharCode> for u32 {
    fn from(code: FourCharCode) -> Self {
        code.0
    }
}

impl fmt::Display for FourCharCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
