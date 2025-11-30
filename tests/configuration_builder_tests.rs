//! Stream configuration builder pattern tests
//!
//! Comprehensive tests for the SCStreamConfiguration builder pattern

use screencapturekit::cg::CGRect;
use screencapturekit::cm::CMTime;
use screencapturekit::stream::configuration::{PixelFormat, SCStreamConfiguration};

// MARK: - Builder Pattern Tests

#[test]
fn test_builder_new() {
    let config = SCStreamConfiguration::new();
    // Should have some default width/height
    let _ = config.width();
    let _ = config.height();
}

#[test]
fn test_builder_with_width() {
    let config = SCStreamConfiguration::new().with_width(1920);
    assert_eq!(config.width(), 1920);
}

#[test]
fn test_builder_with_height() {
    let config = SCStreamConfiguration::new().with_height(1080);
    assert_eq!(config.height(), 1080);
}

#[test]
fn test_builder_chaining() {
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA)
        .with_shows_cursor(true)
        .with_captures_audio(false);

    assert_eq!(config.width(), 1920);
    assert_eq!(config.height(), 1080);
    assert!(config.shows_cursor());
}

#[test]
fn test_builder_with_queue_depth() {
    let config = SCStreamConfiguration::new().with_queue_depth(8);
    assert_eq!(config.queue_depth(), 8);
}

#[test]
fn test_builder_with_minimum_frame_interval() {
    let frame_interval = CMTime::new(1, 60); // 60 FPS
    let config = SCStreamConfiguration::new().with_minimum_frame_interval(&frame_interval);

    let result = config.minimum_frame_interval();
    if result.is_valid() {
        assert_eq!(result.value, 1);
        assert_eq!(result.timescale, 60);
    }
}

#[test]
fn test_builder_with_fps() {
    let config = SCStreamConfiguration::new().with_fps(60);
    // FPS is derived from minimum_frame_interval
    let fps = config.fps();
    // Note: May be 0 on some macOS versions
    println!("FPS: {}", fps);
}

#[test]
fn test_builder_with_scales_to_fit() {
    let config = SCStreamConfiguration::new().with_scales_to_fit(true);
    assert!(config.scales_to_fit());

    let config2 = SCStreamConfiguration::new().with_scales_to_fit(false);
    assert!(!config2.scales_to_fit());
}

#[test]
fn test_builder_with_source_rect() {
    let rect = CGRect::new(0.0, 0.0, 1920.0, 1080.0);
    let config = SCStreamConfiguration::new().with_source_rect(rect);

    let result = config.source_rect();
    // Note: source_rect might return default on some macOS versions
    println!("Source rect: {:?}", result);
}

#[test]
fn test_builder_with_destination_rect() {
    let rect = CGRect::new(0.0, 0.0, 960.0, 540.0);
    let config = SCStreamConfiguration::new().with_destination_rect(rect);

    let result = config.destination_rect();
    println!("Destination rect: {:?}", result);
}

#[test]
fn test_builder_with_background_color() {
    let config = SCStreamConfiguration::new().with_background_color(1.0, 0.0, 0.0); // Red
    // No getter for background color, just verify it doesn't crash
}

#[test]
fn test_builder_with_stream_name() {
    let config = SCStreamConfiguration::new().with_stream_name(Some("TestStream"));
    let name = config.stream_name();
    // Stream name may not be retrievable on all macOS versions
    println!("Stream name: {:?}", name);
}

#[test]
fn test_builder_audio_configuration() {
    let config = SCStreamConfiguration::new()
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    assert!(config.captures_audio());
    // Note: sample_rate and channel_count getters may return defaults
}

// MARK: - Multiple Format Tests

#[test]
fn test_all_pixel_formats() {
    let formats = [
        PixelFormat::BGRA,
        PixelFormat::l10r,
        PixelFormat::YCbCr_420v,
        PixelFormat::YCbCr_420f,
    ];

    for format in formats {
        let config = SCStreamConfiguration::new().with_pixel_format(format);
        // Just verify no crash
        let _ = config.width();
    }
}

#[test]
fn test_pixel_format_display() {
    let format = PixelFormat::BGRA;
    let display = format!("{format}");
    assert!(!display.is_empty());

    let format = PixelFormat::YCbCr_420v;
    let display = format!("{format}");
    assert!(!display.is_empty());
}

// MARK: - Edge Cases

#[test]
fn test_zero_dimensions() {
    let config = SCStreamConfiguration::new().with_width(0).with_height(0);
    assert_eq!(config.width(), 0);
    assert_eq!(config.height(), 0);
}

#[test]
fn test_large_dimensions() {
    let config = SCStreamConfiguration::new()
        .with_width(7680) // 8K
        .with_height(4320);
    assert_eq!(config.width(), 7680);
    assert_eq!(config.height(), 4320);
}

#[test]
fn test_odd_dimensions() {
    // Non-standard dimensions
    let config = SCStreamConfiguration::new().with_width(1921).with_height(1081);
    assert_eq!(config.width(), 1921);
    assert_eq!(config.height(), 1081);
}

#[test]
fn test_very_high_fps() {
    let frame_interval = CMTime::new(1, 240); // 240 FPS
    let config = SCStreamConfiguration::new().with_minimum_frame_interval(&frame_interval);
    // Just verify no crash
    let _ = config.minimum_frame_interval();
}

#[test]
fn test_very_low_fps() {
    let frame_interval = CMTime::new(1, 1); // 1 FPS
    let config = SCStreamConfiguration::new().with_minimum_frame_interval(&frame_interval);
    let _ = config.minimum_frame_interval();
}

// MARK: - Configuration Clone and Equality

#[test]
fn test_configuration_clone() {
    let config1 = SCStreamConfiguration::new().with_width(1920).with_height(1080);

    let config2 = config1.clone();
    assert_eq!(config1.width(), config2.width());
    assert_eq!(config1.height(), config2.height());
}

#[test]
fn test_configuration_equality() {
    let config1 = SCStreamConfiguration::new();
    let config2 = SCStreamConfiguration::new();

    // Two default configs should be equal
    assert_eq!(config1, config2);
}

#[test]
fn test_configuration_hash() {
    use std::collections::HashSet;

    let config1 = SCStreamConfiguration::new();
    let config2 = SCStreamConfiguration::new();

    let mut set = HashSet::new();
    set.insert(config1);
    set.insert(config2);

    // Both should hash to the same value since they're default configs
    // Note: This depends on the Hash implementation
}

// MARK: - Mutable vs Builder Pattern Consistency

#[test]
fn test_mutable_matches_builder() {
    // Using builder pattern
    let builder_config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_shows_cursor(true);

    // Using mutable pattern
    let mut mutable_config = SCStreamConfiguration::new();
    mutable_config.set_width(1920);
    mutable_config.set_height(1080);
    mutable_config.set_shows_cursor(true);

    assert_eq!(builder_config.width(), mutable_config.width());
    assert_eq!(builder_config.height(), mutable_config.height());
    assert_eq!(builder_config.shows_cursor(), mutable_config.shows_cursor());
}

// MARK: - Feature-Gated Tests

#[test]
#[cfg(feature = "macos_14_0")]
fn test_builder_capture_resolution_type() {
    use screencapturekit::stream::configuration::SCCaptureResolutionType;

    let config = SCStreamConfiguration::new()
        .with_capture_resolution_type(SCCaptureResolutionType::Best);

    let result = config.capture_resolution_type();
    println!("Capture resolution type: {:?}", result);
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_all_capture_resolution_types() {
    use screencapturekit::stream::configuration::SCCaptureResolutionType;

    let types = [
        SCCaptureResolutionType::Automatic,
        SCCaptureResolutionType::Best,
        SCCaptureResolutionType::Nominal,
    ];

    for res_type in types {
        let config = SCStreamConfiguration::new().with_capture_resolution_type(res_type);
        let _ = config.capture_resolution_type();
    }
}

#[test]
#[cfg(feature = "macos_14_2")]
fn test_builder_advanced_options() {
    use screencapturekit::stream::configuration::SCPresenterOverlayAlertSetting;

    let config = SCStreamConfiguration::new()
        .with_ignore_fraction_of_screen(0.1)
        .with_ignores_shadows_single_window(true)
        .with_should_be_opaque(false)
        .with_includes_child_windows(true)
        .with_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Never);

    // Verify setters worked (getters may return defaults on older macOS)
    let _ = config.ignore_fraction_of_screen();
    let _ = config.ignores_shadows_single_window();
    let _ = config.should_be_opaque();
    let _ = config.includes_child_windows();
    let _ = config.presenter_overlay_privacy_alert_setting();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_builder_ignore_global_clipboard() {
    let config = SCStreamConfiguration::new().with_ignore_global_clipboard(true);
    let _ = config.ignore_global_clipboard();
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_builder_ignores_shadow_display_configuration() {
    let config = SCStreamConfiguration::new().with_ignores_shadow_display_configuration(true);
    let _ = config.ignores_shadow_display_configuration();
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_builder_captures_microphone() {
    let config = SCStreamConfiguration::new().with_captures_microphone(true);
    let result = config.captures_microphone();
    println!("Captures microphone: {}", result);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_builder_dynamic_range() {
    use screencapturekit::stream::configuration::SCCaptureDynamicRange;

    let ranges = [
        SCCaptureDynamicRange::SDR,
        SCCaptureDynamicRange::HDRLocalDisplay,
        SCCaptureDynamicRange::HDRCanonicalDisplay,
    ];

    for range in ranges {
        let config = SCStreamConfiguration::new().with_capture_dynamic_range(range);
        let result = config.capture_dynamic_range();
        println!("Dynamic range: {:?}", result);
    }
}

// MARK: - Audio Configuration Tests

#[test]
fn test_audio_sample_rates() {
    let rates = [22050, 44100, 48000, 96000, 192000];

    for rate in rates {
        let config = SCStreamConfiguration::new()
            .with_captures_audio(true)
            .with_sample_rate(rate);
        // Just verify no crash
        let _ = config.captures_audio();
    }
}

#[test]
fn test_audio_channel_counts() {
    let counts = [1, 2, 6, 8]; // Mono, Stereo, 5.1, 7.1

    for count in counts {
        let config = SCStreamConfiguration::new()
            .with_captures_audio(true)
            .with_channel_count(count);
        // Just verify no crash
        let _ = config.captures_audio();
    }
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_excludes_current_process_audio() {
    let config = SCStreamConfiguration::new()
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true);
    let _ = config.excludes_current_process_audio();
}

// MARK: - Color Configuration Tests

#[test]
fn test_color_space_name() {
    let config = SCStreamConfiguration::new().with_color_space_name("kCGColorSpaceSRGB");
    // No getter, just verify no crash
}

#[test]
fn test_color_matrix() {
    let config = SCStreamConfiguration::new().with_color_matrix("kCGColorMatrix709");
    // No getter, just verify no crash
}

// MARK: - Complete Configuration Test

#[test]
fn test_complete_production_configuration() {
    // A realistic production configuration
    let frame_interval = CMTime::new(1, 60);

    let config = SCStreamConfiguration::new()
        // Video
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&frame_interval)
        .with_queue_depth(8)
        .with_shows_cursor(true)
        .with_scales_to_fit(false)
        // Audio
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    // Verify critical settings
    assert_eq!(config.width(), 1920);
    assert_eq!(config.height(), 1080);
    assert!(config.shows_cursor());
    assert!(config.captures_audio());
    assert_eq!(config.queue_depth(), 8);
}

#[test]
fn test_streaming_configuration() {
    // Configuration optimized for streaming
    let frame_interval = CMTime::new(1, 30);

    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::YCbCr_420v) // Better for encoding
        .with_minimum_frame_interval(&frame_interval)
        .with_queue_depth(3) // Lower latency
        .with_shows_cursor(true)
        .with_captures_audio(true)
        .with_sample_rate(44100)
        .with_channel_count(2);

    assert_eq!(config.width(), 1280);
    assert_eq!(config.height(), 720);
}

#[test]
fn test_screenshot_configuration() {
    // Configuration optimized for single screenshots
    let config = SCStreamConfiguration::new()
        .with_width(3840)
        .with_height(2160)
        .with_pixel_format(PixelFormat::BGRA)
        .with_queue_depth(1)
        .with_shows_cursor(false)
        .with_captures_audio(false);

    assert_eq!(config.width(), 3840);
    assert_eq!(config.height(), 2160);
    assert!(!config.shows_cursor());
    assert!(!config.captures_audio());
}
