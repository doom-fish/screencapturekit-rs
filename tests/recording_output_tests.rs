//! Recording output tests
//!
//! Tests for `SCRecordingOutput` and `SCRecordingOutputConfiguration` (macOS 15.0+).

#![cfg(feature = "macos_15_0")]

use screencapturekit::recording_output::{SCRecordingOutput, SCRecordingOutputConfiguration};

#[test]
fn test_recording_output_configuration_new() {
    let config = SCRecordingOutputConfiguration::new();
    println!("✓ Recording output configuration created");
    drop(config);
}

#[test]
fn test_recording_output_configuration_clone() {
    let config1 = SCRecordingOutputConfiguration::new();
    let config2 = config1.clone();

    drop(config1);
    drop(config2);

    println!("✓ Recording output configuration clone works");
}

#[test]
fn test_recording_output_configuration_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SCRecordingOutputConfiguration>();
    assert_sync::<SCRecordingOutputConfiguration>();

    println!("✓ SCRecordingOutputConfiguration is Send + Sync");
}

#[test]
fn test_recording_output_new() {
    let config = SCRecordingOutputConfiguration::new();

    let result = SCRecordingOutput::new(&config);

    match result {
        Some(output) => {
            println!("✓ Recording output created successfully");
            drop(output);
        }
        None => {
            println!(
                "⚠ Recording output creation failed (expected in test env - requires macOS 15.0+)"
            );
        }
    }
}

#[test]
fn test_recording_output_clone() {
    let config = SCRecordingOutputConfiguration::new();

    if let Some(output1) = SCRecordingOutput::new(&config) {
        let output2 = output1.clone();

        drop(output1);
        drop(output2);

        println!("✓ Recording output clone works");
    } else {
        println!("⚠ Skipping clone test - recording output unavailable");
    }
}

#[test]
fn test_recording_output_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SCRecordingOutput>();
    assert_sync::<SCRecordingOutput>();

    println!("✓ SCRecordingOutput is Send + Sync");
}

#[test]
fn test_recording_output_multiple_instances() {
    let config = SCRecordingOutputConfiguration::new();

    let output1 = SCRecordingOutput::new(&config);
    let output2 = SCRecordingOutput::new(&config);

    if output1.is_some() {
        println!("✓ Multiple recording outputs can be created");
    } else {
        println!("⚠ Recording output creation requires macOS 15.0+ or permissions");
    }

    assert!(
        output1.is_some() == output2.is_some(),
        "Both outputs should have same creation status"
    );
}

#[test]
fn test_recording_output_api_availability() {
    // Just test that the types exist and are accessible
    let _config_type = std::any::type_name::<SCRecordingOutputConfiguration>();
    let _output_type = std::any::type_name::<SCRecordingOutput>();

    println!("✓ Recording output API is available on macOS 15.0+");
}

#[test]
fn test_recording_configuration() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;
    use std::path::PathBuf;

    let path = PathBuf::from("/tmp/test_recording.mp4");
    let config = SCRecordingOutputConfiguration::new()
        .with_output_url(&path)
        .with_video_codec(SCRecordingOutputCodec::H264);
    // Just verify it doesn't crash
    assert!(!config.as_ptr().is_null());
}

// MARK: - New Recording Output Features

#[test]
fn test_recording_output_video_codec_get_set() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    // Test H264
    let config = SCRecordingOutputConfiguration::new()
        .with_video_codec(SCRecordingOutputCodec::H264);
    assert_eq!(config.video_codec(), SCRecordingOutputCodec::H264);
    
    // Test HEVC
    let config = SCRecordingOutputConfiguration::new()
        .with_video_codec(SCRecordingOutputCodec::HEVC);
    assert_eq!(config.video_codec(), SCRecordingOutputCodec::HEVC);
}

#[test]
fn test_recording_output_file_type() {
    use screencapturekit::recording_output::SCRecordingOutputFileType;

    // Test MP4
    let config = SCRecordingOutputConfiguration::new()
        .with_output_file_type(SCRecordingOutputFileType::MP4);
    assert_eq!(config.output_file_type(), SCRecordingOutputFileType::MP4);
    
    // Test MOV
    let config = SCRecordingOutputConfiguration::new()
        .with_output_file_type(SCRecordingOutputFileType::MOV);
    assert_eq!(config.output_file_type(), SCRecordingOutputFileType::MOV);
}

#[test]
fn test_recording_output_available_codecs_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_video_codecs_count();
    // Should have at least one codec available
    println!("Available video codecs: {}", count);
}

#[test]
fn test_recording_output_available_file_types_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_output_file_types_count();
    // Should have at least one file type available
    println!("Available file types: {}", count);
}

#[test]
fn test_recording_output_recorded_duration() {
    let config = SCRecordingOutputConfiguration::new();

    if let Some(output) = SCRecordingOutput::new(&config) {
        let duration = output.recorded_duration();
        // Not recording, so duration should be 0
        assert_eq!(duration.value, 0);
        println!("✓ Recorded duration accessible");
    } else {
        println!("⚠ Skipping duration test - recording output unavailable");
    }
}

#[test]
fn test_recording_output_recorded_file_size() {
    let config = SCRecordingOutputConfiguration::new();

    if let Some(output) = SCRecordingOutput::new(&config) {
        let size = output.recorded_file_size();
        // Not recording, so size should be 0
        assert_eq!(size, 0);
        println!("✓ Recorded file size accessible");
    } else {
        println!("⚠ Skipping file size test - recording output unavailable");
    }
}

#[test]
fn test_recording_output_codec_equality() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    assert_eq!(SCRecordingOutputCodec::H264, SCRecordingOutputCodec::H264);
    assert_eq!(SCRecordingOutputCodec::HEVC, SCRecordingOutputCodec::HEVC);
    assert_ne!(SCRecordingOutputCodec::H264, SCRecordingOutputCodec::HEVC);
}

#[test]
fn test_recording_output_file_type_equality() {
    use screencapturekit::recording_output::SCRecordingOutputFileType;

    assert_eq!(SCRecordingOutputFileType::MP4, SCRecordingOutputFileType::MP4);
    assert_eq!(SCRecordingOutputFileType::MOV, SCRecordingOutputFileType::MOV);
    assert_ne!(SCRecordingOutputFileType::MP4, SCRecordingOutputFileType::MOV);
}

#[test]
fn test_recording_output_codec_hash() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;
    use std::collections::HashSet;

    let mut codecs = HashSet::new();
    codecs.insert(SCRecordingOutputCodec::H264);
    codecs.insert(SCRecordingOutputCodec::HEVC);
    codecs.insert(SCRecordingOutputCodec::H264); // Duplicate

    assert_eq!(codecs.len(), 2);
}

#[test]
fn test_recording_output_file_type_hash() {
    use screencapturekit::recording_output::SCRecordingOutputFileType;
    use std::collections::HashSet;

    let mut types = HashSet::new();
    types.insert(SCRecordingOutputFileType::MP4);
    types.insert(SCRecordingOutputFileType::MOV);
    types.insert(SCRecordingOutputFileType::MP4); // Duplicate

    assert_eq!(types.len(), 2);
}

#[test]
fn test_recording_output_configuration_debug() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    let config = SCRecordingOutputConfiguration::new()
        .with_video_codec(SCRecordingOutputCodec::HEVC);

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SCRecordingOutputConfiguration"));
    assert!(debug_str.contains("HEVC"));
}

#[test]
fn test_recording_output_available_video_codecs() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    let config = SCRecordingOutputConfiguration::new();
    let codecs = config.available_video_codecs();

    println!("Available video codecs: {:?}", codecs);
    // Should contain at least H264
    if !codecs.is_empty() {
        assert!(
            codecs.contains(&SCRecordingOutputCodec::H264)
                || codecs.contains(&SCRecordingOutputCodec::HEVC)
        );
    }
}

#[test]
fn test_recording_output_available_file_types() {
    use screencapturekit::recording_output::SCRecordingOutputFileType;

    let config = SCRecordingOutputConfiguration::new();
    let file_types = config.available_output_file_types();

    println!("Available file types: {:?}", file_types);
    // Should contain at least MP4 or MOV
    if !file_types.is_empty() {
        assert!(
            file_types.contains(&SCRecordingOutputFileType::MP4)
                || file_types.contains(&SCRecordingOutputFileType::MOV)
        );
    }
}

#[test]
fn test_recording_output_codec_array_matches_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_video_codecs_count();
    let codecs = config.available_video_codecs();

    // The array length should match the count
    assert_eq!(codecs.len(), count);
}

#[test]
fn test_recording_output_file_type_array_matches_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_output_file_types_count();
    let file_types = config.available_output_file_types();

    // The array length should match the count
    assert_eq!(file_types.len(), count);
}
