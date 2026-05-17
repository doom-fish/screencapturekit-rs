//! Stream configuration tests
//!
//! Tests for `SCStreamConfiguration` and related types

use screencapturekit::stream::configuration::{PixelFormat, SCStreamConfiguration};

#[test]
fn test_default_configuration() {
    let _config = SCStreamConfiguration::default();
    // Just verify it doesn't crash
}

/// Regression test for issue #145.
///
/// Apple's stock `SCStreamConfiguration()` no longer defaults to BGRA on
/// macOS 26 / Apple Silicon — the runtime delivers `420v` unless the caller
/// overrides `pixelFormat`. The Swift bridge pins BGRA at construction time
/// to restore the long-standing crate default; this test locks in that
/// guarantee so a future regression in the bridge surfaces immediately.
#[test]
fn test_default_pixel_format_is_bgra() {
    let config = SCStreamConfiguration::new();
    assert_eq!(
        config.pixel_format(),
        PixelFormat::BGRA,
        "SCStreamConfiguration::new() must default to BGRA across macOS \
         versions (see issue #145)",
    );
}

#[test]
fn test_set_dimensions() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);

    assert_eq!(config.width(), 1920);
    assert_eq!(config.height(), 1080);
}

#[test]
fn test_set_dimensions_multiple() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_width(1280);
    config.set_height(720);

    assert_eq!(config.width(), 1280);
    assert_eq!(config.height(), 720);
}

#[test]
fn test_set_pixel_format() {
    let formats = [
        PixelFormat::BGRA,
        PixelFormat::YCbCr_420v,
        PixelFormat::YCbCr_420f,
        PixelFormat::l10r,
        PixelFormat::xf44,
        PixelFormat::RGhA,
    ];

    for format in formats {
        let mut config = SCStreamConfiguration::default();
        config.set_pixel_format(format);
        // Just verify it doesn't crash
    }
}

#[test]
fn test_audio_configuration() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_audio(true);
    // Just verify it doesn't crash
}

#[test]
fn test_audio_sample_rate() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    // Just verify it doesn't crash
}

#[test]
fn test_audio_channel_count() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_audio(true);
    config.set_channel_count(2);
    // Just verify it doesn't crash
}

#[test]
fn test_complete_audio_configuration() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    config.set_channel_count(2);
    // Just verify it doesn't crash
}

#[test]
fn test_complete_video_audio_configuration() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_pixel_format(PixelFormat::BGRA);
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    config.set_channel_count(2);
    // Just verify it doesn't crash
}

#[test]
fn test_various_resolutions() {
    let resolutions = [
        (640, 480),
        (1280, 720),
        (1920, 1080),
        (2560, 1440),
        (3840, 2160),
    ];

    for (width, height) in resolutions {
        let mut config = SCStreamConfiguration::default();
        config.set_width(width);
        config.set_height(height);

        assert_eq!(config.width(), width, "Width mismatch for {width}x{height}");
        assert_eq!(
            config.height(),
            height,
            "Height mismatch for {width}x{height}"
        );
    }
}

#[test]
fn test_common_sample_rates() {
    let sample_rates = [44100, 48000, 96000];

    for rate in sample_rates {
        let mut config = SCStreamConfiguration::default();
        config.set_captures_audio(true);
        config.set_sample_rate(rate);
        // Just verify it doesn't crash
    }
}

#[test]
fn test_channel_counts() {
    let channels = [1, 2];

    for count in channels {
        let mut config = SCStreamConfiguration::default();
        config.set_captures_audio(true);
        config.set_channel_count(count);
        // Just verify it doesn't crash
    }
}

#[test]
fn test_pixel_format_equality() {
    assert_eq!(PixelFormat::BGRA, PixelFormat::BGRA);
    assert_ne!(PixelFormat::BGRA, PixelFormat::YCbCr_420v);
}

#[test]
fn test_pixel_format_in_collections() {
    use std::collections::HashSet;

    let mut formats = HashSet::new();
    formats.insert(PixelFormat::BGRA);
    formats.insert(PixelFormat::YCbCr_420v);
    formats.insert(PixelFormat::BGRA); // Duplicate

    assert_eq!(formats.len(), 2);
}

/// Regression test for the `Unknown(known_code)` normalisation.
///
/// `PixelFormat::Unknown(FourCharCode::from_bytes(*b"BGRA"))` is a
/// "well-formed but redundant" value: it carries the same wire-level
/// format as `PixelFormat::BGRA`. Without normalisation, the two
/// values would compare unequal and hash differently, producing
/// `HashMap` / `HashSet` bugs and confusing equality assertions in
/// downstream code.
///
/// This test pins the contract that `PixelFormat::PartialEq` and
/// `PixelFormat::Hash` normalise through `FourCharCode`. A future
/// refactor that re-derives `PartialEq`/`Hash` from the enum
/// discriminant would silently re-introduce the bug; this test
/// catches it.
#[test]
fn test_pixel_format_unknown_normalises_to_named_variant() {
    use screencapturekit::FourCharCode;
    use std::collections::HashSet;

    // Equality across variant boundaries.
    let synonym = PixelFormat::Unknown(FourCharCode::from_bytes(*b"BGRA"));
    assert_eq!(
        PixelFormat::BGRA,
        synonym,
        "Unknown(BGRA) must compare equal to BGRA"
    );

    let yuv_synonym = PixelFormat::Unknown(FourCharCode::from_bytes(*b"420v"));
    assert_eq!(
        PixelFormat::YCbCr_420v,
        yuv_synonym,
        "Unknown(420v) must compare equal to YCbCr_420v"
    );

    // Hash must agree with Eq.
    let mut formats = HashSet::new();
    formats.insert(PixelFormat::BGRA);
    formats.insert(synonym);
    assert_eq!(
        formats.len(),
        1,
        "BGRA and Unknown(BGRA) must collide in a HashSet"
    );

    // Genuinely-unknown codes still compare distinctly.
    let truly_unknown = PixelFormat::Unknown(FourCharCode::from_bytes(*b"zzzz"));
    assert_ne!(PixelFormat::BGRA, truly_unknown);
}

#[test]
fn test_configuration_set_width() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    assert_eq!(config.width(), 1920);
}

#[test]
fn test_multiple_configurations() {
    // Test creating multiple configurations
    let _config1 = SCStreamConfiguration::default();
    let _config2 = SCStreamConfiguration::default();
    let _config3 = SCStreamConfiguration::default();

    // Should not crash or leak memory
}

#[test]
fn test_configuration_modification_order() {
    // Test that order of modifications doesn't matter for final result
    let mut config1 = SCStreamConfiguration::default();
    config1.set_width(1920);
    config1.set_height(1080);

    let mut config2 = SCStreamConfiguration::default();
    config2.set_height(1080);
    config2.set_width(1920);

    assert_eq!(config1.width(), config2.width());
    assert_eq!(config1.height(), config2.height());
}

#[test]
fn test_audio_without_video() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    // Just verify it doesn't crash
}

#[test]
fn test_video_without_audio() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_captures_audio(false);
    // Just verify it doesn't crash
}

#[test]
fn test_stream_name() {
    let mut config = SCStreamConfiguration::default();
    config.set_stream_name(Some("test-stream"));

    // The getter may not work on all macOS versions
    let _ = config.stream_name();
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_dynamic_range() {
    use screencapturekit::stream::configuration::SCCaptureDynamicRange;

    let mut config = SCStreamConfiguration::default();
    config.set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay);

    // May return SDR on macOS < 15.0
    let _ = config.capture_dynamic_range();
}

#[test]
fn test_queue_depth_and_frame_interval() {
    use screencapturekit::cm::CMTime;

    let cm_time = CMTime {
        value: 4,
        timescale: 1,
        flags: 1,
        epoch: 1,
    };
    let queue_depth = 10;
    let mut config = SCStreamConfiguration::default();
    config.set_queue_depth(queue_depth);
    config.set_minimum_frame_interval(&cm_time);

    assert_eq!(config.queue_depth(), queue_depth);

    let acquired_cm_time = config.minimum_frame_interval();
    // Note: minimum_frame_interval may not be supported on all macOS versions
    // If supported, values should match
    if acquired_cm_time.is_valid() {
        assert_eq!(
            acquired_cm_time.value, cm_time.value,
            "Expected value {}, got {}",
            cm_time.value, acquired_cm_time.value
        );
        assert_eq!(
            acquired_cm_time.timescale, cm_time.timescale,
            "Expected timescale {}, got {}",
            cm_time.timescale, acquired_cm_time.timescale
        );
    }
}

#[test]
#[cfg(all(feature = "macos_13_0", feature = "macos_14_2"))]
fn test_advanced_setters() {
    use screencapturekit::stream::configuration::SCPresenterOverlayAlertSetting;

    // These advanced properties require macOS 13.0-14.2+
    // The test verifies that setters don't error, but getters may not
    // return the set values on older macOS versions
    let mut config = SCStreamConfiguration::default();
    config.set_ignores_shadows_single_window(true);
    config.set_should_be_opaque(true);
    config.set_includes_child_windows(true);
    config.set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Always);

    // Verify setters worked without errors
    // Note: getters may return default values on older macOS versions
    let _ = config.ignores_shadows_single_window();
    let _ = config.should_be_opaque();
    let _ = config.includes_child_windows();
    let _ = config.presenter_overlay_privacy_alert_setting();
}

#[test]
fn test_shows_cursor() {
    let mut config = SCStreamConfiguration::default();
    config.set_shows_cursor(true);
    assert!(config.shows_cursor());
}

#[test]
fn test_mutable_configuration() {
    // Test the mutable configuration pattern
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_pixel_format(PixelFormat::BGRA);
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    config.set_channel_count(2);

    assert_eq!(config.width(), 1920);
    assert_eq!(config.height(), 1080);
}

// MARK: - New Features Tests (macOS 14.0+)

#[test]
#[cfg(feature = "macos_14_0")]
fn test_captures_shadows_only() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_shadows_only(true);
    // On macOS 14.0+, this should return true; on older versions, false
    let _ = config.captures_shadows_only();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_captures_shadows_only_builder() {
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_captures_shadows_only(true);

    // Verify builder pattern works
    assert_eq!(config.width(), 1920);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_shows_mouse_clicks() {
    let mut config = SCStreamConfiguration::default();
    config.set_shows_mouse_clicks(true);
    // On macOS 15.0+, this should return true
    let result = config.shows_mouse_clicks();
    // Note: May return false on older macOS versions
    let _ = result;
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_shows_mouse_clicks_builder() {
    let config = SCStreamConfiguration::new()
        .with_shows_cursor(true)
        .with_shows_mouse_clicks(true);

    // Verify builder pattern works
    assert!(config.shows_cursor());
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignores_shadows_display() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignores_shadows_display(true);
    // On macOS 14.0+, this should return true
    let _ = config.ignores_shadows_display();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignore_global_clip_display() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignore_global_clip_display(true);
    let _ = config.ignore_global_clip_display();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignore_global_clip_single_window() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignore_global_clip_single_window(true);
    let _ = config.ignore_global_clip_single_window();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignore_global_clip_builder_pattern() {
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_ignores_shadows_display(true)
        .with_ignore_global_clip_display(true)
        .with_ignore_global_clip_single_window(true);

    assert_eq!(config.width(), 1920);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_preset_configuration() {
    use screencapturekit::stream::configuration::SCStreamConfigurationPreset;

    // Test creating configurations from presets
    let config = SCStreamConfiguration::from_preset(
        SCStreamConfigurationPreset::CaptureHDRStreamLocalDisplay,
    );
    // Just verify it doesn't crash
    let _ = config.width();
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_all_presets() {
    use screencapturekit::stream::configuration::SCStreamConfigurationPreset;

    let presets = [
        SCStreamConfigurationPreset::CaptureHDRStreamLocalDisplay,
        SCStreamConfigurationPreset::CaptureHDRStreamCanonicalDisplay,
        SCStreamConfigurationPreset::CaptureHDRScreenshotLocalDisplay,
        SCStreamConfigurationPreset::CaptureHDRScreenshotCanonicalDisplay,
    ];

    for preset in presets {
        let config = SCStreamConfiguration::from_preset(preset);
        // Just verify they don't crash
        let _ = config.width();
    }
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_hdr_recording_preset() {
    use screencapturekit::stream::configuration::{
        PixelFormat, SCCaptureDynamicRange, SCStreamConfigurationPreset,
    };
    use screencapturekit::FourCharCode;

    let config = SCStreamConfiguration::from_preset(
        SCStreamConfigurationPreset::CaptureHDRRecordingPreservedSDRHDR10,
    );

    assert_eq!(
        config.capture_dynamic_range(),
        SCCaptureDynamicRange::HDRCanonicalDisplay,
        "HDR10 recording preset must request canonical-display HDR capture"
    );
    assert_eq!(
        config.pixel_format(),
        PixelFormat::Unknown(FourCharCode::from_bytes(*b"x420")),
        "HDR10 recording preset must use the SDK's x420 recording pixel format"
    );
    assert_eq!(
        config.color_matrix(),
        Some("ITU_R_2020".to_string()),
        "HDR10 recording preset must use the SDK's ITU_R_2020 color matrix"
    );
    assert_eq!(
        config.color_space_name(),
        Some("kCGColorSpaceITUR_2100_PQ".to_string()),
        "HDR10 recording preset must use the SDK's PQ color space"
    );
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_preserves_aspect_ratio() {
    let mut config = SCStreamConfiguration::default();

    // Test setting preserves_aspect_ratio
    config.set_preserves_aspect_ratio(true);
    assert!(config.preserves_aspect_ratio());

    config.set_preserves_aspect_ratio(false);
    assert!(!config.preserves_aspect_ratio());
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_preserves_aspect_ratio_builder() {
    let config = SCStreamConfiguration::new().with_preserves_aspect_ratio(true);

    assert!(config.preserves_aspect_ratio());
}

// MARK: - Pixel Format Tests

#[test]
fn test_pixel_format_xf44() {
    // xf44: 2 plane "full" range YCbCr10 4:4:4 (10-bit)
    let format = PixelFormat::xf44;
    let display = format!("{format}");
    assert_eq!(display, "xf44");
}

#[test]
fn test_pixel_format_rgha() {
    // RGhA: 64-bit RGBA IEEE half-precision float (HDR)
    let format = PixelFormat::RGhA;
    let display = format!("{format}");
    assert_eq!(display, "RGhA");
}

#[test]
fn test_all_pixel_formats() {
    let formats = [
        (PixelFormat::BGRA, "BGRA"),
        (PixelFormat::l10r, "l10r"),
        (PixelFormat::YCbCr_420v, "420v"),
        (PixelFormat::YCbCr_420f, "420f"),
        (PixelFormat::xf44, "xf44"),
        (PixelFormat::RGhA, "RGhA"),
    ];

    for (format, expected_display) in formats {
        let display = format!("{format}");
        assert_eq!(display, expected_display, "Display mismatch for {format:?}");
    }
}

#[test]
fn test_pixel_format_roundtrip() {
    use screencapturekit::utils::four_char_code::FourCharCode;

    let formats = [
        PixelFormat::BGRA,
        PixelFormat::l10r,
        PixelFormat::YCbCr_420v,
        PixelFormat::YCbCr_420f,
        PixelFormat::xf44,
        PixelFormat::RGhA,
    ];

    for format in formats {
        let four_cc: FourCharCode = format.into();
        let back: PixelFormat = four_cc.into();
        assert_eq!(format, back, "Roundtrip failed for {format:?}");
    }
}

#[test]
fn test_pixel_format_hash_all() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(PixelFormat::BGRA);
    set.insert(PixelFormat::l10r);
    set.insert(PixelFormat::YCbCr_420v);
    set.insert(PixelFormat::YCbCr_420f);
    set.insert(PixelFormat::xf44);
    set.insert(PixelFormat::RGhA);

    assert_eq!(set.len(), 6);
}

#[test]
fn test_hdr_pixel_formats() {
    // These formats are typically used for HDR capture
    let hdr_formats = [
        PixelFormat::l10r, // 10-bit ARGB
        PixelFormat::xf44, // 10-bit YCbCr 4:4:4
        PixelFormat::RGhA, // 16-bit half-precision float
    ];

    for format in hdr_formats {
        let mut config = SCStreamConfiguration::default();
        config.set_pixel_format(format);
        // Just verify HDR formats can be set
    }
}
