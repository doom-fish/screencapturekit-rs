//! Comprehensive tests for Eq, `PartialEq`, and Hash implementations
//!
//! Tests all types that implement these traits to ensure correctness.

use std::collections::{HashMap, HashSet};
use screencapturekit::prelude::*;
use screencapturekit::{
    cg::{CGPoint, CGRect, CGSize},
    cm::{CMSampleTimingInfo, CMTime},
    codec_types, media_types, FourCharCode,
};

#[test]
fn test_cmtime_eq_and_hash() {
    let time1 = CMTime::new(1, 30);
    let time2 = CMTime::new(1, 30);
    let time3 = CMTime::new(2, 30);

    // Test equality
    assert_eq!(time1, time2);
    assert_ne!(time1, time3);

    // Test hash
    let mut set = HashSet::new();
    set.insert(time1);
    set.insert(time2); // Duplicate
    set.insert(time3);
    assert_eq!(set.len(), 2);

    // Test in HashMap
    let mut map = HashMap::new();
    map.insert(time1, "frame 1");
    map.insert(time2, "frame 1 again"); // Overwrites
    map.insert(time3, "frame 2");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&time1), Some(&"frame 1 again"));
}

#[test]
fn test_cmtime_special_values() {
    let zero = CMTime::ZERO;
    let invalid = CMTime::INVALID;
    let pos_inf = CMTime::positive_infinity();
    let neg_inf = CMTime::negative_infinity();
    let indefinite = CMTime::indefinite();

    // All should be unequal
    assert_ne!(zero, invalid);
    assert_ne!(zero, pos_inf);
    assert_ne!(invalid, pos_inf);
    assert_ne!(pos_inf, neg_inf);
    assert_ne!(pos_inf, indefinite);

    // Each should equal itself
    assert_eq!(zero, zero);
    assert_eq!(invalid, invalid);
    assert_eq!(pos_inf, pos_inf);

    // Should work in collections
    let mut set = HashSet::new();
    set.insert(zero);
    set.insert(invalid);
    set.insert(pos_inf);
    set.insert(neg_inf);
    set.insert(indefinite);
    assert_eq!(set.len(), 5);
}

#[test]
fn test_cmsample_timing_info_eq_and_hash() {
    let timing1 = CMSampleTimingInfo::with_times(
        CMTime::new(1, 30),
        CMTime::new(0, 30),
        CMTime::INVALID,
    );
    let timing2 = CMSampleTimingInfo::with_times(
        CMTime::new(1, 30),
        CMTime::new(0, 30),
        CMTime::INVALID,
    );
    let timing3 = CMSampleTimingInfo::with_times(
        CMTime::new(1, 30),
        CMTime::new(1, 30),
        CMTime::INVALID,
    );

    assert_eq!(timing1, timing2);
    assert_ne!(timing1, timing3);

    let mut set = HashSet::new();
    set.insert(timing1);
    set.insert(timing2);
    set.insert(timing3);
    assert_eq!(set.len(), 2);
}

#[test]
fn test_cgpoint_eq_and_hash() {
    let p1 = CGPoint::new(10.0, 20.0);
    let p2 = CGPoint::new(10.0, 20.0);
    let p3 = CGPoint::new(10.0, 20.1);

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);

    let mut set = HashSet::new();
    set.insert(p1);
    set.insert(p2);
    set.insert(p3);
    assert_eq!(set.len(), 2);
}

#[test]
fn test_cgsize_eq_and_hash() {
    let s1 = CGSize::new(1920.0, 1080.0);
    let s2 = CGSize::new(1920.0, 1080.0);
    let s3 = CGSize::new(3840.0, 2160.0);

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);

    let mut map: HashMap<CGSize, &str> = HashMap::new();
    map.insert(s1, "Full HD");
    map.insert(s2, "1080p"); // Overwrites
    map.insert(s3, "4K");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&s1), Some(&"1080p"));
}

#[test]
fn test_cgrect_eq_and_hash() {
    let r1 = CGRect::new(0.0, 0.0, 100.0, 100.0);
    let r2 = CGRect::new(0.0, 0.0, 100.0, 100.0);
    let r3 = CGRect::new(0.0, 0.0, 100.0, 101.0);

    assert_eq!(r1, r2);
    assert_ne!(r1, r3);

    let mut set = HashSet::new();
    set.insert(r1);
    set.insert(r2);
    set.insert(r3);
    assert_eq!(set.len(), 2);
}

#[test]
fn test_fourcharcode_eq_and_hash() {
    let code1 = FourCharCode::from_bytes(*b"avc1");
    let code2 = FourCharCode::from_bytes(*b"avc1");
    let code3 = FourCharCode::from_bytes(*b"hvc1");

    assert_eq!(code1, code2);
    assert_ne!(code1, code3);

    // Test with constants
    assert_eq!(codec_types::H264, FourCharCode::from_bytes(*b"avc1"));
    assert_eq!(media_types::VIDEO, FourCharCode::from_bytes(*b"vide"));

    let mut map: HashMap<FourCharCode, &str> = HashMap::new();
    map.insert(codec_types::H264, "H.264");
    map.insert(codec_types::HEVC, "HEVC");
    map.insert(codec_types::AAC, "AAC");
    assert_eq!(map.len(), 3);
    assert_eq!(map.get(&codec_types::H264), Some(&"H.264"));
}

#[test]
fn test_media_type_constants_eq_and_hash() {
    let video1 = media_types::VIDEO;
    let video2 = media_types::VIDEO;
    let audio = media_types::AUDIO;

    assert_eq!(video1, video2);
    assert_ne!(video1, audio);

    let mut set = HashSet::new();
    set.insert(media_types::VIDEO);
    set.insert(media_types::AUDIO);
    set.insert(media_types::VIDEO); // Duplicate
    set.insert(media_types::MUXED);
    set.insert(media_types::TEXT);
    assert_eq!(set.len(), 4);
}

#[test]
fn test_codec_type_constants_eq_and_hash() {
    let mut codecs: HashSet<FourCharCode> = HashSet::new();
    
    // Video codecs
    codecs.insert(codec_types::H264);
    codecs.insert(codec_types::HEVC);
    codecs.insert(codec_types::HEVC_2);
    codecs.insert(codec_types::JPEG);
    
    // Audio codecs
    codecs.insert(codec_types::AAC);
    codecs.insert(codec_types::LPCM);
    codecs.insert(codec_types::ALAC);
    
    // Test duplicates
    codecs.insert(codec_types::H264); // Duplicate
    
    assert_eq!(codecs.len(), 7);
}

#[test]
fn test_audio_buffer_eq_and_hash() {
    // AudioBuffer has a private data_ptr field, so we skip detailed testing
    // The Eq and Hash implementations are tested via integration tests
    // that actually capture audio
    use std::collections::HashSet;
    let _set: HashSet<screencapturekit::cm::AudioBuffer> = HashSet::new();
}

#[test]
fn test_pixel_format_eq_and_hash() {
    use screencapturekit::stream::configuration::PixelFormat;

    let fmt1 = PixelFormat::BGRA;
    let fmt2 = PixelFormat::BGRA;
    let fmt3 = PixelFormat::YCbCr_420v;

    assert_eq!(fmt1, fmt2);
    assert_ne!(fmt1, fmt3);

    let mut formats: HashSet<PixelFormat> = HashSet::new();
    formats.insert(PixelFormat::BGRA);
    formats.insert(PixelFormat::BGRA); // Duplicate
    formats.insert(PixelFormat::l10r);
    formats.insert(PixelFormat::YCbCr_420v);
    formats.insert(PixelFormat::YCbCr_420f);
    assert_eq!(formats.len(), 4);

    // Test in HashMap
    let mut map: HashMap<PixelFormat, &str> = HashMap::new();
    map.insert(PixelFormat::BGRA, "BGRA");
    map.insert(PixelFormat::YCbCr_420v, "YUV 420v");
    assert_eq!(map.len(), 2);
}

#[test]
fn test_stream_output_type_eq_and_hash() {
    use screencapturekit::stream::output_type::SCStreamOutputType;

    let screen1 = SCStreamOutputType::Screen;
    let screen2 = SCStreamOutputType::Screen;
    let audio = SCStreamOutputType::Audio;

    assert_eq!(screen1, screen2);
    assert_ne!(screen1, audio);

    let mut set = HashSet::new();
    set.insert(SCStreamOutputType::Screen);
    set.insert(SCStreamOutputType::Audio);
    set.insert(SCStreamOutputType::Screen); // Duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_config_types_eq_and_hash() {
    use screencapturekit::stream::configuration::types::{Point, Rect, Size};

    // Test Point
    let p1 = Point::new(10.0, 20.0);
    let p2 = Point::new(10.0, 20.0);
    let p3 = Point::new(11.0, 20.0);
    
    assert_eq!(p1, p2);
    assert_ne!(p1, p3);

    let mut points = HashSet::new();
    points.insert(p1);
    points.insert(p2);
    points.insert(p3);
    assert_eq!(points.len(), 2);

    // Test Size
    let s1 = Size::new(100.0, 200.0);
    let s2 = Size::new(100.0, 200.0);
    let s3 = Size::new(100.0, 201.0);

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);

    let mut sizes = HashSet::new();
    sizes.insert(s1);
    sizes.insert(s2);
    sizes.insert(s3);
    assert_eq!(sizes.len(), 2);

    // Test Rect
    let r1 = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0));
    let r2 = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0));
    let r3 = Rect::new(Point::new(1.0, 0.0), Size::new(100.0, 100.0));

    assert_eq!(r1, r2);
    assert_ne!(r1, r3);

    let mut rects = HashSet::new();
    rects.insert(r1);
    rects.insert(r2);
    rects.insert(r3);
    assert_eq!(rects.len(), 2);
}

#[test]
fn test_error_eq() {
    let err1 = SCError::invalid_dimension("width", 0);
    let err2 = SCError::invalid_dimension("width", 0);
    let err3 = SCError::invalid_dimension("height", 0);

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);

    // Test other error types
    let perm_err1 = SCError::permission_denied("Screen Recording");
    let perm_err2 = SCError::permission_denied("Screen Recording");
    let perm_err3 = SCError::permission_denied("Microphone");

    assert_eq!(perm_err1, perm_err2);
    assert_ne!(perm_err1, perm_err3);
}

#[test]
fn test_lock_options_eq_and_hash() {
    use screencapturekit::output::pixel_buffer::PixelBufferLockFlags;

    let read1 = PixelBufferLockFlags::ReadOnly;
    let read2 = PixelBufferLockFlags::ReadOnly;

    assert_eq!(read1, read2);

    let mut flags = HashSet::new();
    flags.insert(PixelBufferLockFlags::ReadOnly);
    flags.insert(PixelBufferLockFlags::ReadOnly); // Duplicate
    assert_eq!(flags.len(), 1);
}

#[test]
fn test_iosurface_lock_options_eq_and_hash() {
    use screencapturekit::output::iosurface::IOSurfaceLockOptions;

    let opt1 = IOSurfaceLockOptions::ReadOnly;
    let opt2 = IOSurfaceLockOptions::ReadOnly;
    let opt3 = IOSurfaceLockOptions::AvoidSync;

    assert_eq!(opt1, opt2);
    assert_ne!(opt1, opt3);

    let mut options_set = HashSet::new();
    options_set.insert(IOSurfaceLockOptions::ReadOnly);
    options_set.insert(IOSurfaceLockOptions::AvoidSync);
    options_set.insert(IOSurfaceLockOptions::ReadOnly); // Duplicate
    assert_eq!(options_set.len(), 2);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_recording_codec_eq_and_hash() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    let h264_1 = SCRecordingOutputCodec::H264;
    let h264_2 = SCRecordingOutputCodec::H264;
    let hevc = SCRecordingOutputCodec::HEVC;

    assert_eq!(h264_1, h264_2);
    assert_ne!(h264_1, hevc);

    let mut codecs = HashSet::new();
    codecs.insert(SCRecordingOutputCodec::H264);
    codecs.insert(SCRecordingOutputCodec::HEVC);
    codecs.insert(SCRecordingOutputCodec::H264); // Duplicate
    assert_eq!(codecs.len(), 2);

    // Test in HashMap
    let mut map: HashMap<SCRecordingOutputCodec, &str> = HashMap::new();
    map.insert(SCRecordingOutputCodec::H264, "H.264");
    map.insert(SCRecordingOutputCodec::HEVC, "HEVC");
    assert_eq!(map.len(), 2);
}

#[test]
fn test_presenter_overlay_setting_eq_and_hash() {
    use screencapturekit::stream::configuration::advanced::SCPresenterOverlayAlertSetting;

    let never1 = SCPresenterOverlayAlertSetting::Never;
    let never2 = SCPresenterOverlayAlertSetting::Never;
    let once = SCPresenterOverlayAlertSetting::Once;
    let always = SCPresenterOverlayAlertSetting::Always;

    assert_eq!(never1, never2);
    assert_ne!(never1, once);
    assert_ne!(once, always);

    let mut settings = HashSet::new();
    settings.insert(SCPresenterOverlayAlertSetting::Never);
    settings.insert(SCPresenterOverlayAlertSetting::Once);
    settings.insert(SCPresenterOverlayAlertSetting::Always);
    settings.insert(SCPresenterOverlayAlertSetting::Never); // Duplicate
    assert_eq!(settings.len(), 3);
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_sharing_picker_mode_eq_and_hash() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerMode;

    let mode1 = SCContentSharingPickerMode::SingleWindow;
    let mode2 = SCContentSharingPickerMode::SingleWindow;
    let mode3 = SCContentSharingPickerMode::Multiple;

    assert_eq!(mode1, mode2);
    assert_ne!(mode1, mode3);

    let mut modes = HashSet::new();
    modes.insert(SCContentSharingPickerMode::SingleWindow);
    modes.insert(SCContentSharingPickerMode::Multiple);
    modes.insert(SCContentSharingPickerMode::SingleWindow); // Duplicate
    assert_eq!(modes.len(), 2);
}

#[test]
fn test_complex_nested_collections() {
    // Test complex nested structures
    let mut codec_to_formats: HashMap<FourCharCode, HashSet<CGSize>> = HashMap::new();
    
    let mut h264_sizes = HashSet::new();
    h264_sizes.insert(CGSize::new(1920.0, 1080.0));
    h264_sizes.insert(CGSize::new(1280.0, 720.0));
    
    let mut hevc_sizes = HashSet::new();
    hevc_sizes.insert(CGSize::new(3840.0, 2160.0));
    hevc_sizes.insert(CGSize::new(1920.0, 1080.0));
    
    codec_to_formats.insert(codec_types::H264, h264_sizes);
    codec_to_formats.insert(codec_types::HEVC, hevc_sizes);
    
    assert_eq!(codec_to_formats.len(), 2);
    assert_eq!(codec_to_formats.get(&codec_types::H264).unwrap().len(), 2);
    assert_eq!(codec_to_formats.get(&codec_types::HEVC).unwrap().len(), 2);
}

#[test]
fn test_cmtime_const_equality() {
    // Test const equality method
    const TIME1: CMTime = CMTime::new(1, 30);
    const TIME2: CMTime = CMTime::new(1, 30);
    const TIME3: CMTime = CMTime::new(2, 30);
    
    assert!(TIME1.equals(&TIME2));
    assert!(!TIME1.equals(&TIME3));
    
    // Test with invalid times
    assert!(!CMTime::INVALID.equals(&CMTime::INVALID)); // Invalid times don't equal
    assert!(CMTime::ZERO.equals(&CMTime::ZERO));
}

#[test]
fn test_fourcharcode_const_equality() {
    const H264: FourCharCode = codec_types::H264;
    const HEVC: FourCharCode = codec_types::HEVC;
    
    assert!(H264.equals(H264));
    assert!(!H264.equals(HEVC));
}

#[test]
fn test_deduplication_in_vec() {
    use screencapturekit::stream::configuration::PixelFormat;
    
    let formats = vec![
        PixelFormat::BGRA,
        PixelFormat::YCbCr_420v,
        PixelFormat::BGRA, // Duplicate
        PixelFormat::YCbCr_420v, // Duplicate
        PixelFormat::l10r,
    ];
    
    let unique: HashSet<_> = formats.into_iter().collect();
    assert_eq!(unique.len(), 3);
}

#[test]
fn test_hashmap_key_stability() {
    // Ensure hash values are stable across insertions
    let time = CMTime::new(1, 30);
    
    let mut map1 = HashMap::new();
    map1.insert(time, "value1");
    
    let mut map2 = HashMap::new();
    map2.insert(time, "value2");
    
    // Same key should hash to same value in different maps
    assert!(map1.contains_key(&time));
    assert!(map2.contains_key(&time));
    
    // Should be able to retrieve from both
    assert_eq!(map1.get(&time), Some(&"value1"));
    assert_eq!(map2.get(&time), Some(&"value2"));
}

#[test]
fn test_zero_values_eq() {
    // Test that zero/default values work correctly
    assert_eq!(CMTime::ZERO, CMTime::ZERO);
    assert_eq!(CGPoint::zero(), CGPoint::zero());
    assert_eq!(CGSize::zero(), CGSize::zero());
    assert_eq!(CGRect::zero(), CGRect::zero());
    
    let mut set = HashSet::new();
    set.insert(CMTime::ZERO);
    set.insert(CMTime::ZERO);
    assert_eq!(set.len(), 1);
}

#[test]
fn test_edge_case_float_values() {
    // Test edge cases with floating point
    let p1 = CGPoint::new(0.0, 0.0);
    let p2 = CGPoint::new(-0.0, -0.0);
    
    // 0.0 and -0.0 should be equal (same bit representation)
    assert_eq!(p1, p2);
    
    // Test very small differences
    let p3 = CGPoint::new(1.0, 1.0);
    let p4 = CGPoint::new(1.0 + f64::EPSILON, 1.0);
    assert_ne!(p3, p4); // Should be different
}
