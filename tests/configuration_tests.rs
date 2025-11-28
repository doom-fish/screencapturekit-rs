//! Stream configuration tests
//!
//! Tests for `SCStreamConfiguration` and related types

use screencapturekit::stream::configuration::{PixelFormat, SCStreamConfiguration};

#[test]
fn test_default_configuration() {
    let _config = SCStreamConfiguration::default();
    // Just verify it doesn't crash
}

#[test]
fn test_set_dimensions() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);

    assert_eq!(config.get_width(), 1920);
    assert_eq!(config.get_height(), 1080);
}

#[test]
fn test_set_dimensions_multiple() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_width(1280);
    config.set_height(720);

    assert_eq!(config.get_width(), 1280);
    assert_eq!(config.get_height(), 720);
}

#[test]
fn test_set_pixel_format() {
    let formats = [
        PixelFormat::BGRA,
        PixelFormat::YCbCr_420v,
        PixelFormat::YCbCr_420f,
        PixelFormat::l10r,
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

        assert_eq!(
            config.get_width(),
            width,
            "Width mismatch for {width}x{height}"
        );
        assert_eq!(
            config.get_height(),
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

#[test]
fn test_configuration_set_width() {
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    assert_eq!(config.get_width(), 1920);
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

    assert_eq!(config1.get_width(), config2.get_width());
    assert_eq!(config1.get_height(), config2.get_height());
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
    let _ = config.get_stream_name();
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_dynamic_range() {
    use screencapturekit::stream::configuration::SCCaptureDynamicRange;

    let mut config = SCStreamConfiguration::default();
    config.set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay);

    // May return SDR on macOS < 15.0
    let _ = config.get_capture_dynamic_range();
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

    assert_eq!(config.get_queue_depth(), queue_depth);

    let acquired_cm_time = config.get_minimum_frame_interval();
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
    config.set_ignore_fraction_of_screen(0.1);
    config.set_ignores_shadows_single_window(true);
    config.set_should_be_opaque(true);
    config.set_includes_child_windows(true);
    config.set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Always);

    // Verify setters worked without errors
    // Note: getters may return default values on older macOS versions
    let _ = config.get_ignore_fraction_of_screen();
    let _ = config.get_ignores_shadows_single_window();
    let _ = config.get_should_be_opaque();
    let _ = config.get_includes_child_windows();
    let _ = config.get_presenter_overlay_privacy_alert_setting();
}

#[test]
fn test_shows_cursor() {
    let mut config = SCStreamConfiguration::default();
    config.set_shows_cursor(true);
    assert!(config.get_shows_cursor());
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

    assert_eq!(config.get_width(), 1920);
    assert_eq!(config.get_height(), 1080);
}

// MARK: - New Features Tests (macOS 14.0+)

#[test]
fn test_captures_shadows_only() {
    let mut config = SCStreamConfiguration::default();
    config.set_captures_shadows_only(true);
    // On macOS 14.0+, this should return true; on older versions, false
    let _ = config.get_captures_shadows_only();
}

#[test]
fn test_captures_shadows_only_builder() {
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_captures_shadows_only(true);
    
    // Verify builder pattern works
    assert_eq!(config.get_width(), 1920);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_shows_mouse_clicks() {
    let mut config = SCStreamConfiguration::default();
    config.set_shows_mouse_clicks(true);
    // On macOS 15.0+, this should return true
    let result = config.get_shows_mouse_clicks();
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
    assert!(config.get_shows_cursor());
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignores_shadows_display() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignores_shadows_display(true);
    // On macOS 14.0+, this should return true
    let _ = config.get_ignores_shadows_display();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignore_global_clip_display() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignore_global_clip_display(true);
    let _ = config.get_ignore_global_clip_display();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_ignore_global_clip_single_window() {
    let mut config = SCStreamConfiguration::default();
    config.set_ignore_global_clip_single_window(true);
    let _ = config.get_ignore_global_clip_single_window();
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
    
    assert_eq!(config.get_width(), 1920);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_preset_configuration() {
    use screencapturekit::stream::configuration::SCStreamConfigurationPreset;
    
    // Test creating configurations from presets
    let config = SCStreamConfiguration::from_preset(SCStreamConfigurationPreset::CaptureHDRStreamLocalDisplay);
    // Just verify it doesn't crash
    let _ = config.get_width();
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
        let _ = config.get_width();
    }
}
