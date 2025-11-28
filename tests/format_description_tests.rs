//! `CMFormatDescription` tests

use screencapturekit::cm::{codec_types, media_types, CMFormatDescription};
use screencapturekit::FourCharCode;

#[test]
fn test_media_type_constants() {
    assert_eq!(media_types::VIDEO, FourCharCode::from_bytes(*b"vide"));
    assert_eq!(media_types::AUDIO, FourCharCode::from_bytes(*b"soun"));
    assert_eq!(media_types::MUXED, FourCharCode::from_bytes(*b"mux "));
    assert_eq!(media_types::TEXT, FourCharCode::from_bytes(*b"text"));
    assert_eq!(media_types::CLOSED_CAPTION, FourCharCode::from_bytes(*b"clcp"));
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
