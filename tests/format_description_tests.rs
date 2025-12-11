//! `CMFormatDescription` tests

use screencapturekit::cm::{codec_types, media_types, CMFormatDescription};
use screencapturekit::FourCharCode;

#[test]
fn test_media_type_constants() {
    assert_eq!(media_types::VIDEO, FourCharCode::from_bytes(*b"vide"));
    assert_eq!(media_types::AUDIO, FourCharCode::from_bytes(*b"soun"));
    assert_eq!(media_types::MUXED, FourCharCode::from_bytes(*b"mux "));
    assert_eq!(media_types::TEXT, FourCharCode::from_bytes(*b"text"));
    assert_eq!(
        media_types::CLOSED_CAPTION,
        FourCharCode::from_bytes(*b"clcp")
    );
    assert_eq!(media_types::METADATA, FourCharCode::from_bytes(*b"meta"));
    assert_eq!(media_types::TIMECODE, FourCharCode::from_bytes(*b"tmcd"));
}

#[test]
fn test_codec_type_constants() {
    // Video codecs
    assert_eq!(codec_types::H264, FourCharCode::from_bytes(*b"avc1"));
    assert_eq!(codec_types::HEVC, FourCharCode::from_bytes(*b"hvc1"));
    assert_eq!(codec_types::HEVC_2, FourCharCode::from_bytes(*b"hev1"));
    assert_eq!(codec_types::JPEG, FourCharCode::from_bytes(*b"jpeg"));
    assert_eq!(codec_types::PRORES_422, FourCharCode::from_bytes(*b"apcn"));
    assert_eq!(codec_types::PRORES_4444, FourCharCode::from_bytes(*b"ap4h"));

    // Audio codecs
    assert_eq!(codec_types::AAC, FourCharCode::from_bytes(*b"aac "));
    assert_eq!(codec_types::LPCM, FourCharCode::from_bytes(*b"lpcm"));
    assert_eq!(codec_types::ALAC, FourCharCode::from_bytes(*b"alac"));
    assert_eq!(codec_types::OPUS, FourCharCode::from_bytes(*b"opus"));
    assert_eq!(codec_types::FLAC, FourCharCode::from_bytes(*b"flac"));
}

#[test]
fn test_media_type_display() {
    assert_eq!(media_types::VIDEO.display(), "vide");
    assert_eq!(media_types::AUDIO.display(), "soun");
}

#[test]
fn test_codec_type_display() {
    assert_eq!(codec_types::H264.display(), "avc1");
    assert_eq!(codec_types::HEVC.display(), "hvc1");
    assert_eq!(codec_types::AAC.display(), "aac ");
}

#[test]
fn test_format_description_from_raw_null() {
    let desc = CMFormatDescription::from_raw(std::ptr::null_mut());
    assert!(desc.is_none());
}

#[test]
fn test_media_type_equality() {
    let video1 = media_types::VIDEO;
    let video2 = media_types::VIDEO;
    let audio = media_types::AUDIO;

    assert_eq!(video1, video2);
    assert_ne!(video1, audio);
}

#[test]
fn test_codec_type_in_collections() {
    use std::collections::HashSet;

    let mut codecs = HashSet::new();
    codecs.insert(codec_types::H264);
    codecs.insert(codec_types::HEVC);
    codecs.insert(codec_types::AAC);

    assert!(codecs.contains(&codec_types::H264));
    assert!(codecs.contains(&codec_types::HEVC));
    assert!(!codecs.contains(&codec_types::JPEG));
}

#[test]
fn test_format_description_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CMFormatDescription>();
    assert_sync::<CMFormatDescription>();
}

#[test]
fn test_format_description_debug_impl() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<CMFormatDescription>();
}

#[test]
fn test_format_description_display_impl() {
    fn assert_display<T: std::fmt::Display>() {}
    assert_display::<CMFormatDescription>();
}

#[test]
fn test_format_description_clone_impl() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<CMFormatDescription>();
}

#[test]
fn test_format_description_eq_hash() {
    fn assert_eq_impl<T: PartialEq + Eq>() {}
    fn assert_hash_impl<T: std::hash::Hash>() {}
    assert_eq_impl::<CMFormatDescription>();
    assert_hash_impl::<CMFormatDescription>();
}

#[test]
fn test_media_type_raw_values() {
    // Test raw u32 values are consistent with FourCharCode
    assert_eq!(media_types::VIDEO.as_u32(), u32::from_be_bytes(*b"vide"));
    assert_eq!(media_types::AUDIO.as_u32(), u32::from_be_bytes(*b"soun"));
}

#[test]
fn test_codec_fourcc_matching() {
    // Test that common codec FourCC values match expectations
    let h264 = codec_types::H264;
    let hevc = codec_types::HEVC;

    // Different codecs should not be equal
    assert_ne!(h264, hevc);

    // Same codec should be equal
    assert_eq!(h264, codec_types::H264);
}

#[test]
fn test_hevc_variants() {
    // HEVC has two variant FourCC codes
    assert_ne!(codec_types::HEVC, codec_types::HEVC_2);
    assert_eq!(codec_types::HEVC.display(), "hvc1");
    assert_eq!(codec_types::HEVC_2.display(), "hev1");
}

#[test]
fn test_prores_variants() {
    // ProRes has multiple variants
    assert_ne!(codec_types::PRORES_422, codec_types::PRORES_4444);
    assert_eq!(codec_types::PRORES_422.display(), "apcn");
    assert_eq!(codec_types::PRORES_4444.display(), "ap4h");
}
