//! Stream configuration tests
//!
//! Tests for SCStreamConfiguration and related types

use screencapturekit::stream::configuration::{PixelFormat, SCStreamConfiguration};

#[test]
fn test_default_configuration() {
    let _config = SCStreamConfiguration::build();
    // Just verify it doesn't crash
}

#[test]
fn test_set_dimensions() {
    let result = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080));
    
    assert!(result.is_ok());
}

#[test]
fn test_set_dimensions_chaining() {
    let result = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080))
        .and_then(|c| c.set_width(1280))
        .and_then(|c| c.set_height(720));
    
    assert!(result.is_ok());
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
        let result = SCStreamConfiguration::build().set_pixel_format(format);
        assert!(result.is_ok(), "Failed to set pixel format {:?}", format);
    }
}

#[test]
fn test_audio_configuration() {
    let result = SCStreamConfiguration::build()
        .set_captures_audio(true);
    
    assert!(result.is_ok());
}

#[test]
fn test_audio_sample_rate() {
    let result = SCStreamConfiguration::build()
        .set_captures_audio(true)
        .and_then(|c| c.set_sample_rate(48000));
    
    assert!(result.is_ok());
}

#[test]
fn test_audio_channel_count() {
    let result = SCStreamConfiguration::build()
        .set_captures_audio(true)
        .and_then(|c| c.set_channel_count(2));
    
    assert!(result.is_ok());
}

#[test]
fn test_complete_audio_configuration() {
    let result = SCStreamConfiguration::build()
        .set_captures_audio(true)
        .and_then(|c| c.set_sample_rate(48000))
        .and_then(|c| c.set_channel_count(2));
    
    assert!(result.is_ok());
}

#[test]
fn test_complete_video_audio_configuration() {
    let result = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080))
        .and_then(|c| c.set_pixel_format(PixelFormat::BGRA))
        .and_then(|c| c.set_captures_audio(true))
        .and_then(|c| c.set_sample_rate(48000))
        .and_then(|c| c.set_channel_count(2));
    
    assert!(result.is_ok());
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
        let result = SCStreamConfiguration::build()
            .set_width(width)
            .and_then(|c| c.set_height(height));
        
        assert!(result.is_ok(), "Failed to set resolution {}x{}", width, height);
    }
}

#[test]
fn test_common_sample_rates() {
    let sample_rates = [44100, 48000, 96000];
    
    for rate in sample_rates {
        let result = SCStreamConfiguration::build()
            .set_captures_audio(true)
            .and_then(|c| c.set_sample_rate(rate));
        
        assert!(result.is_ok(), "Failed to set sample rate {}", rate);
    }
}

#[test]
fn test_channel_counts() {
    let channels = [1, 2];
    
    for count in channels {
        let result = SCStreamConfiguration::build()
            .set_captures_audio(true)
            .and_then(|c| c.set_channel_count(count));
        
        assert!(result.is_ok(), "Failed to set channel count {}", count);
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
fn test_configuration_error_handling() {
    // Test that configuration methods return Result types
    let result = SCStreamConfiguration::build().set_width(1920);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_configurations() {
    // Test creating multiple configurations
    let _config1 = SCStreamConfiguration::build();
    let _config2 = SCStreamConfiguration::build();
    let _config3 = SCStreamConfiguration::build();
    
    // Should not crash or leak memory
}

#[test]
fn test_configuration_modification_order() {
    // Test that order of modifications doesn't matter
    let result1 = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080));
    
    let result2 = SCStreamConfiguration::build()
        .set_height(1080)
        .and_then(|c| c.set_width(1920));
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_audio_without_video() {
    let result = SCStreamConfiguration::build()
        .set_captures_audio(true)
        .and_then(|c| c.set_sample_rate(48000));
    
    assert!(result.is_ok());
}

#[test]
fn test_video_without_audio() {
    let result = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080))
        .and_then(|c| c.set_captures_audio(false));
    
    assert!(result.is_ok());
}
