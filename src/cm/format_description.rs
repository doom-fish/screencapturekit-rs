//! `CMFormatDescription` - Media format description

#![allow(dead_code)]

use super::ffi;
use std::fmt;

pub struct CMFormatDescription(*mut std::ffi::c_void);

impl PartialEq for CMFormatDescription {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMFormatDescription {}

impl std::hash::Hash for CMFormatDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_format_description_hash(self.0);
            hash_value.hash(state);
        }
    }
}

/// Common media type constants
pub mod media_types {
    use crate::utils::four_char_code::FourCharCode;

    /// Video media type ('vide')
    pub const VIDEO: FourCharCode = FourCharCode::from_bytes(*b"vide");
    /// Audio media type ('soun')
    pub const AUDIO: FourCharCode = FourCharCode::from_bytes(*b"soun");
    /// Muxed media type ('mux ')
    pub const MUXED: FourCharCode = FourCharCode::from_bytes(*b"mux ");
    /// Text/subtitle media type ('text')
    pub const TEXT: FourCharCode = FourCharCode::from_bytes(*b"text");
    /// Closed caption media type ('clcp')
    pub const CLOSED_CAPTION: FourCharCode = FourCharCode::from_bytes(*b"clcp");
    /// Metadata media type ('meta')
    pub const METADATA: FourCharCode = FourCharCode::from_bytes(*b"meta");
    /// Timecode media type ('tmcd')
    pub const TIMECODE: FourCharCode = FourCharCode::from_bytes(*b"tmcd");
}

/// Common codec type constants
pub mod codec_types {
    use crate::utils::four_char_code::FourCharCode;

    // Video codecs
    /// H.264/AVC ('avc1')
    pub const H264: FourCharCode = FourCharCode::from_bytes(*b"avc1");
    /// HEVC/H.265 ('hvc1')
    pub const HEVC: FourCharCode = FourCharCode::from_bytes(*b"hvc1");
    /// HEVC/H.265 alternative ('hev1')
    pub const HEVC_2: FourCharCode = FourCharCode::from_bytes(*b"hev1");
    /// JPEG ('jpeg')
    pub const JPEG: FourCharCode = FourCharCode::from_bytes(*b"jpeg");
    /// Apple `ProRes` 422 ('apcn')
    pub const PRORES_422: FourCharCode = FourCharCode::from_bytes(*b"apcn");
    /// Apple `ProRes` 4444 ('ap4h')
    pub const PRORES_4444: FourCharCode = FourCharCode::from_bytes(*b"ap4h");

    // Audio codecs
    /// AAC ('aac ')
    pub const AAC: FourCharCode = FourCharCode::from_bytes(*b"aac ");
    /// Linear PCM ('lpcm')
    pub const LPCM: FourCharCode = FourCharCode::from_bytes(*b"lpcm");
    /// Apple Lossless ('alac')
    pub const ALAC: FourCharCode = FourCharCode::from_bytes(*b"alac");
    /// Opus ('opus')
    pub const OPUS: FourCharCode = FourCharCode::from_bytes(*b"opus");
    /// FLAC ('flac')
    pub const FLAC: FourCharCode = FourCharCode::from_bytes(*b"flac");
}

impl CMFormatDescription {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMFormatDescription` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Get the media type as a raw u32 value
    pub fn media_type_raw(&self) -> u32 {
        unsafe { ffi::cm_format_description_get_media_type(self.0) }
    }

    /// Get the media type as `FourCharCode`
    pub fn media_type(&self) -> crate::utils::four_char_code::FourCharCode {
        crate::utils::four_char_code::FourCharCode::from(self.media_type_raw())
    }

    /// Get the media subtype (codec type) as a raw u32 value
    pub fn media_subtype_raw(&self) -> u32 {
        unsafe { ffi::cm_format_description_get_media_subtype(self.0) }
    }

    /// Get the media subtype as `FourCharCode`
    pub fn media_subtype(&self) -> crate::utils::four_char_code::FourCharCode {
        crate::utils::four_char_code::FourCharCode::from(self.media_subtype_raw())
    }

    /// Get format description extensions
    pub fn extensions(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cm_format_description_get_extensions(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Check if this is a video format description
    pub fn is_video(&self) -> bool {
        self.media_type() == media_types::VIDEO
    }

    /// Check if this is an audio format description
    pub fn is_audio(&self) -> bool {
        self.media_type() == media_types::AUDIO
    }

    /// Check if this is a muxed format description
    pub fn is_muxed(&self) -> bool {
        self.media_type() == media_types::MUXED
    }

    /// Check if this is a text/subtitle format description
    pub fn is_text(&self) -> bool {
        self.media_type() == media_types::TEXT
    }

    /// Check if this is a closed caption format description
    pub fn is_closed_caption(&self) -> bool {
        self.media_type() == media_types::CLOSED_CAPTION
    }

    /// Check if this is a metadata format description
    pub fn is_metadata(&self) -> bool {
        self.media_type() == media_types::METADATA
    }

    /// Check if this is a timecode format description
    pub fn is_timecode(&self) -> bool {
        self.media_type() == media_types::TIMECODE
    }

    /// Get a human-readable string for the media type
    pub fn media_type_string(&self) -> String {
        self.media_type().display()
    }

    /// Get a human-readable string for the media subtype (codec)
    pub fn media_subtype_string(&self) -> String {
        self.media_subtype().display()
    }

    /// Check if the codec is H.264
    pub fn is_h264(&self) -> bool {
        self.media_subtype() == codec_types::H264
    }

    /// Check if the codec is HEVC/H.265
    pub fn is_hevc(&self) -> bool {
        let subtype = self.media_subtype();
        subtype == codec_types::HEVC || subtype == codec_types::HEVC_2
    }

    /// Check if the codec is AAC
    pub fn is_aac(&self) -> bool {
        self.media_subtype() == codec_types::AAC
    }

    /// Check if the codec is PCM
    pub fn is_pcm(&self) -> bool {
        self.media_subtype() == codec_types::LPCM
    }

    /// Check if the codec is `ProRes`
    pub fn is_prores(&self) -> bool {
        let subtype = self.media_subtype();
        subtype == codec_types::PRORES_422 || subtype == codec_types::PRORES_4444
    }

    /// Check if the codec is Apple Lossless (ALAC)
    pub fn is_alac(&self) -> bool {
        self.media_subtype() == codec_types::ALAC
    }
}

impl Clone for CMFormatDescription {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cm_format_description_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CMFormatDescription {
    fn drop(&mut self) {
        unsafe {
            ffi::cm_format_description_release(self.0);
        }
    }
}

unsafe impl Send for CMFormatDescription {}
unsafe impl Sync for CMFormatDescription {}

impl fmt::Display for CMFormatDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CMFormatDescription(type: 0x{:08X}, subtype: 0x{:08X})",
            self.media_type_raw(),
            self.media_subtype_raw()
        )
    }
}
