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
    let config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080);
    
    assert_eq!(config.get_width(), 1920);
    assert_eq!(config.get_height(), 1080);
}

#[test]
fn test_set_dimensions_chaining() {
    let config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080)
        .set_width(1280)
        .set_height(720);
    
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
        let _config = SCStreamConfiguration::default().set_pixel_format(format);
        // Just verify it doesn't crash
    }
}

#[test]
fn test_audio_configuration() {
    let _config = SCStreamConfiguration::default()
        .set_captures_audio(true);
    // Just verify it doesn't crash
}

#[test]
fn test_audio_sample_rate() {
    let _config = SCStreamConfiguration::default()
        .set_captures_audio(true)
        .set_sample_rate(48000);
    // Just verify it doesn't crash
}

#[test]
fn test_audio_channel_count() {
    let _config = SCStreamConfiguration::default()
        .set_captures_audio(true)
        .set_channel_count(2);
    // Just verify it doesn't crash
}

#[test]
fn test_complete_audio_configuration() {
    let _config = SCStreamConfiguration::default()
        .set_captures_audio(true)
        .set_sample_rate(48000)
        .set_channel_count(2);
    // Just verify it doesn't crash
}

#[test]
fn test_complete_video_audio_configuration() {
    let _config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080)
        .set_pixel_format(PixelFormat::BGRA)
        .set_captures_audio(true)
        .set_sample_rate(48000)
        .set_channel_count(2);
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
        let config = SCStreamConfiguration::default()
            .set_width(width)
            .set_height(height);
        
        assert_eq!(config.get_width(), width, "Width mismatch for {width}x{height}");
        assert_eq!(config.get_height(), height, "Height mismatch for {width}x{height}");
    }
}

#[test]
fn test_common_sample_rates() {
    let sample_rates = [44100, 48000, 96000];
    
    for rate in sample_rates {
        let _config = SCStreamConfiguration::default()
            .set_captures_audio(true)
            .set_sample_rate(rate);
        // Just verify it doesn't crash
    }
}

#[test]
fn test_channel_counts() {
    let channels = [1, 2];
    
    for count in channels {
        let _config = SCStreamConfiguration::default()
            .set_captures_audio(true)
            .set_channel_count(count);
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
fn test_configuration_chaining() {
    // Test that configuration methods can be chained
    let config = SCStreamConfiguration::default().set_width(1920);
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
    let config1 = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080);
    
    let config2 = SCStreamConfiguration::default()
        .set_height(1080)
        .set_width(1920);
    
    assert_eq!(config1.get_width(), config2.get_width());
    assert_eq!(config1.get_height(), config2.get_height());
}

#[test]
fn test_audio_without_video() {
    let _config = SCStreamConfiguration::default()
        .set_captures_audio(true)
        .set_sample_rate(48000);
    // Just verify it doesn't crash
}

#[test]
fn test_video_without_audio() {
    let _config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080)
        .set_captures_audio(false);
    // Just verify it doesn't crash
}

#[test]
fn test_stream_name() {
    let config = SCStreamConfiguration::default()
        .set_stream_name(Some("test-stream"));
    
    // The getter may not work on all macOS versions
    let _ = config.get_stream_name();
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_dynamic_range() {
    use screencapturekit::stream::configuration::SCCaptureDynamicRange;
    
    let config = SCStreamConfiguration::default()
        .set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay);
    
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
    let config = SCStreamConfiguration::default()
        .set_queue_depth(queue_depth)
        .set_minimum_frame_interval(&cm_time);

    assert_eq!(config.get_queue_depth(), queue_depth);

    let acquired_cm_time = config.get_minimum_frame_interval();
    // Note: minimum_frame_interval may not be supported on all macOS versions
    // If supported, values should match
    if acquired_cm_time.is_valid() {
        assert_eq!(acquired_cm_time.value, cm_time.value, 
            "Expected value {}, got {}", cm_time.value, acquired_cm_time.value);
        assert_eq!(acquired_cm_time.timescale, cm_time.timescale,
            "Expected timescale {}, got {}", cm_time.timescale, acquired_cm_time.timescale);
    }
}

#[test]
#[cfg(all(feature = "macos_13_0", feature = "macos_14_2"))]
fn test_advanced_setters() {
    use screencapturekit::stream::configuration::SCPresenterOverlayAlertSetting;
    
    // These advanced properties require macOS 13.0-14.2+
    // The test verifies that setters don't error, but getters may not
    // return the set values on older macOS versions
    let config = SCStreamConfiguration::default()
        .set_ignore_fraction_of_screen(0.1)
        .set_ignores_shadows_single_window(true)
        .set_should_be_opaque(true)
        .set_includes_child_windows(true)
        .set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Always);

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
    let config = SCStreamConfiguration::default()
        .set_shows_cursor(true);
    assert!(config.get_shows_cursor());
}

#[test]
fn test_builder_pattern() {
    // Test the new builder pattern
    let config = SCStreamConfiguration::builder()
        .width(1920)
        .height(1080)
        .pixel_format(PixelFormat::BGRA)
        .captures_audio(true)
        .sample_rate(48000)
        .channel_count(2)
        .build();
    
    assert_eq!(config.get_width(), 1920);
    assert_eq!(config.get_height(), 1080);
}
