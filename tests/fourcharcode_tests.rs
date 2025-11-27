//! FourCharCode tests
//!
//! Tests for FourCharCode type and codec/media type constants

use screencapturekit::{codec_types, media_types, FourCharCode};

#[test]
fn test_fourcharcode_from_bytes() {
    let code = FourCharCode::from_bytes(*b"avc1");
    assert_eq!(code.as_u32(), 0x61766331);
}

#[test]
fn test_fourcharcode_from_u32() {
    let code = FourCharCode::from_u32(0x61766331);
    assert_eq!(code, FourCharCode::from_bytes(*b"avc1"));
}

#[test]
fn test_fourcharcode_parse() {
    let code: FourCharCode = "avc1".parse().unwrap();
    assert_eq!(code, codec_types::H264);
}

#[test]
fn test_fourcharcode_parse_invalid() {
    let result = "abc".parse::<FourCharCode>();
    assert!(result.is_err());
    
    let result = "abcde".parse::<FourCharCode>();
    assert!(result.is_err());
}

#[test]
fn test_fourcharcode_display() {
    let code = codec_types::H264;
    let display = format!("{}", code);
    assert_eq!(display, "avc1");
}

#[test]
fn test_fourcharcode_const_equality() {
    const H264: FourCharCode = codec_types::H264;
    const HEVC: FourCharCode = codec_types::HEVC;
    
    assert!(H264.equals(codec_types::H264));
    assert!(!H264.equals(HEVC));
}

#[test]
fn test_media_type_constants() {
    assert_eq!(media_types::VIDEO, FourCharCode::from_bytes(*b"vide"));
    assert_eq!(media_types::AUDIO, FourCharCode::from_bytes(*b"soun"));
    assert_eq!(media_types::MUXED, FourCharCode::from_bytes(*b"mux "));
    assert_eq!(media_types::TEXT, FourCharCode::from_bytes(*b"text"));
}

#[test]
fn test_video_codec_constants() {
    assert_eq!(codec_types::H264, FourCharCode::from_bytes(*b"avc1"));
    assert_eq!(codec_types::HEVC, FourCharCode::from_bytes(*b"hvc1"));
    assert_eq!(codec_types::HEVC_2, FourCharCode::from_bytes(*b"hev1"));
    assert_eq!(codec_types::PRORES_422, FourCharCode::from_bytes(*b"apcn"));
    assert_eq!(codec_types::JPEG, FourCharCode::from_bytes(*b"jpeg"));
}

#[test]
fn test_audio_codec_constants() {
    assert_eq!(codec_types::AAC, FourCharCode::from_bytes(*b"aac "));
    assert_eq!(codec_types::LPCM, FourCharCode::from_bytes(*b"lpcm"));
    assert_eq!(codec_types::ALAC, FourCharCode::from_bytes(*b"alac"));
}

#[test]
fn test_fourcharcode_special_characters() {
    // Test with space character
    let aac = codec_types::AAC;
    let string = format!("{}", aac);
    assert_eq!(string, "aac ");
}

#[test]
fn test_from_str() {
    let code: FourCharCode = "BGRA".parse().unwrap();
    assert_eq!(code.display(), "BGRA");
}

#[test]
fn test_roundtrip() {
    let original: FourCharCode = "420v".parse().unwrap();
    let as_u32: u32 = original.into();
    let back = FourCharCode::from(as_u32);
    assert_eq!(original, back);
}
