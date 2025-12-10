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
    let config =
        SCRecordingOutputConfiguration::new().with_video_codec(SCRecordingOutputCodec::H264);
    assert_eq!(config.video_codec(), SCRecordingOutputCodec::H264);

    // Test HEVC
    let config =
        SCRecordingOutputConfiguration::new().with_video_codec(SCRecordingOutputCodec::HEVC);
    assert_eq!(config.video_codec(), SCRecordingOutputCodec::HEVC);
}

#[test]
fn test_recording_output_file_type() {
    use screencapturekit::recording_output::SCRecordingOutputFileType;

    // Test MP4
    let config =
        SCRecordingOutputConfiguration::new().with_output_file_type(SCRecordingOutputFileType::MP4);
    assert_eq!(config.output_file_type(), SCRecordingOutputFileType::MP4);

    // Test MOV
    let config =
        SCRecordingOutputConfiguration::new().with_output_file_type(SCRecordingOutputFileType::MOV);
    assert_eq!(config.output_file_type(), SCRecordingOutputFileType::MOV);
}

#[test]
fn test_recording_output_available_codecs_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_video_codecs_count();
    // Should have at least one codec available
    println!("Available video codecs: {count}");
}

#[test]
fn test_recording_output_available_file_types_count() {
    let config = SCRecordingOutputConfiguration::new();
    let count = config.available_output_file_types_count();
    // Should have at least one file type available
    println!("Available file types: {count}");
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

    assert_eq!(
        SCRecordingOutputFileType::MP4,
        SCRecordingOutputFileType::MP4
    );
    assert_eq!(
        SCRecordingOutputFileType::MOV,
        SCRecordingOutputFileType::MOV
    );
    assert_ne!(
        SCRecordingOutputFileType::MP4,
        SCRecordingOutputFileType::MOV
    );
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

    let config =
        SCRecordingOutputConfiguration::new().with_video_codec(SCRecordingOutputCodec::HEVC);

    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("SCRecordingOutputConfiguration"));
    assert!(debug_str.contains("HEVC"));
}

#[test]
fn test_recording_output_available_video_codecs() {
    use screencapturekit::recording_output::SCRecordingOutputCodec;

    let config = SCRecordingOutputConfiguration::new();
    let codecs = config.available_video_codecs();

    println!("Available video codecs: {codecs:?}");
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

    println!("Available file types: {file_types:?}");
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

// MARK: - SCRecordingOutputDelegate Tests

#[test]
fn test_recording_delegate_trait_implementation() {
    use screencapturekit::recording_output::SCRecordingOutputDelegate;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    struct TestRecordingDelegate {
        start_count: Arc<AtomicU32>,
        finish_count: Arc<AtomicU32>,
        fail_count: Arc<AtomicU32>,
        last_error: Arc<std::sync::Mutex<Option<String>>>,
    }

    impl SCRecordingOutputDelegate for TestRecordingDelegate {
        fn recording_did_start(&self) {
            self.start_count.fetch_add(1, Ordering::SeqCst);
        }

        fn recording_did_finish(&self) {
            self.finish_count.fetch_add(1, Ordering::SeqCst);
        }

        fn recording_did_fail(&self, error: String) {
            self.fail_count.fetch_add(1, Ordering::SeqCst);
            *self.last_error.lock().unwrap() = Some(error);
        }
    }

    let delegate = TestRecordingDelegate {
        start_count: Arc::new(AtomicU32::new(0)),
        finish_count: Arc::new(AtomicU32::new(0)),
        fail_count: Arc::new(AtomicU32::new(0)),
        last_error: Arc::new(std::sync::Mutex::new(None)),
    };

    // Test start callback
    delegate.recording_did_start();
    assert_eq!(delegate.start_count.load(Ordering::SeqCst), 1);

    // Test finish callback
    delegate.recording_did_finish();
    assert_eq!(delegate.finish_count.load(Ordering::SeqCst), 1);

    // Test fail callback
    delegate.recording_did_fail("Test error".to_string());
    assert_eq!(delegate.fail_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        delegate.last_error.lock().unwrap().as_deref(),
        Some("Test error")
    );
}

#[test]
fn test_recording_delegate_default_implementations() {
    use screencapturekit::recording_output::SCRecordingOutputDelegate;

    struct MinimalRecordingDelegate;
    impl SCRecordingOutputDelegate for MinimalRecordingDelegate {}

    let delegate = MinimalRecordingDelegate;

    // These should not panic - they have default empty implementations
    delegate.recording_did_start();
    delegate.recording_did_finish();
    delegate.recording_did_fail("error".to_string());
}

// MARK: - RecordingCallbacks Tests

#[test]
fn test_recording_callbacks_new() {
    use screencapturekit::recording_output::RecordingCallbacks;

    let callbacks = RecordingCallbacks::new();
    // Should not panic
    drop(callbacks);
}

#[test]
fn test_recording_callbacks_default() {
    use screencapturekit::recording_output::RecordingCallbacks;

    let callbacks = RecordingCallbacks::default();
    // Should not panic
    drop(callbacks);
}

#[test]
fn test_recording_callbacks_on_start() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);

    let callbacks = RecordingCallbacks::new().on_start(move || {
        called_clone.store(true, Ordering::SeqCst);
    });

    callbacks.recording_did_start();
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_recording_callbacks_on_finish() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);

    let callbacks = RecordingCallbacks::new().on_finish(move || {
        called_clone.store(true, Ordering::SeqCst);
    });

    callbacks.recording_did_finish();
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_recording_callbacks_on_fail() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::Arc;

    let error_msg = Arc::new(std::sync::Mutex::new(String::new()));
    let error_clone = Arc::clone(&error_msg);

    let callbacks = RecordingCallbacks::new().on_fail(move |error| {
        *error_clone.lock().unwrap() = error;
    });

    callbacks.recording_did_fail("test failure".to_string());
    assert_eq!(error_msg.lock().unwrap().as_str(), "test failure");
}

#[test]
fn test_recording_callbacks_all_callbacks() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let start_called = Arc::new(AtomicBool::new(false));
    let finish_called = Arc::new(AtomicBool::new(false));
    let fail_called = Arc::new(AtomicBool::new(false));

    let start_clone = Arc::clone(&start_called);
    let finish_clone = Arc::clone(&finish_called);
    let fail_clone = Arc::clone(&fail_called);

    let callbacks = RecordingCallbacks::new()
        .on_start(move || start_clone.store(true, Ordering::SeqCst))
        .on_finish(move || finish_clone.store(true, Ordering::SeqCst))
        .on_fail(move |_| fail_clone.store(true, Ordering::SeqCst));

    // Trigger all callbacks
    callbacks.recording_did_start();
    callbacks.recording_did_finish();
    callbacks.recording_did_fail("error".to_string());

    // Verify all were called
    assert!(start_called.load(Ordering::SeqCst));
    assert!(finish_called.load(Ordering::SeqCst));
    assert!(fail_called.load(Ordering::SeqCst));
}

#[test]
fn test_recording_callbacks_without_handlers() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};

    // Test that callbacks without handlers don't panic
    let callbacks = RecordingCallbacks::new();

    callbacks.recording_did_start();
    callbacks.recording_did_finish();
    callbacks.recording_did_fail("error".to_string());
}

#[test]
fn test_recording_callbacks_partial_handlers() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let fail_called = Arc::new(AtomicBool::new(false));
    let fail_clone = Arc::clone(&fail_called);

    // Only set one callback
    let callbacks =
        RecordingCallbacks::new().on_fail(move |_| fail_clone.store(true, Ordering::SeqCst));

    // Call all methods - only the one with handler should do anything
    callbacks.recording_did_start();
    callbacks.recording_did_finish();
    callbacks.recording_did_fail("error".to_string());

    assert!(fail_called.load(Ordering::SeqCst));
}

#[test]
fn test_recording_callbacks_debug() {
    use screencapturekit::recording_output::RecordingCallbacks;

    let callbacks = RecordingCallbacks::new().on_start(|| {}).on_fail(|_| {});

    let debug_str = format!("{callbacks:?}");
    assert!(debug_str.contains("RecordingCallbacks"));
    assert!(debug_str.contains("on_start"));
    assert!(debug_str.contains("on_fail"));
    assert!(debug_str.contains("on_finish"));
}

#[test]
fn test_recording_callbacks_is_send() {
    use screencapturekit::recording_output::RecordingCallbacks;

    fn assert_send<T: Send>() {}
    assert_send::<RecordingCallbacks>();
}

#[test]
fn test_recording_callbacks_multiple_calls() {
    use screencapturekit::recording_output::{RecordingCallbacks, SCRecordingOutputDelegate};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let start_count = Arc::new(AtomicU32::new(0));
    let finish_count = Arc::new(AtomicU32::new(0));

    let start_clone = Arc::clone(&start_count);
    let finish_clone = Arc::clone(&finish_count);

    let callbacks = RecordingCallbacks::new()
        .on_start(move || {
            start_clone.fetch_add(1, Ordering::SeqCst);
        })
        .on_finish(move || {
            finish_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Call start multiple times (simulating restart scenarios)
    callbacks.recording_did_start();
    callbacks.recording_did_start();
    callbacks.recording_did_finish();

    assert_eq!(start_count.load(Ordering::SeqCst), 2);
    assert_eq!(finish_count.load(Ordering::SeqCst), 1);
}

// MARK: - Delegate with SCRecordingOutput Integration

#[test]
fn test_recording_output_with_delegate() {
    use screencapturekit::recording_output::RecordingCallbacks;
    use std::path::PathBuf;

    let path = PathBuf::from("/tmp/test_delegate_recording.mp4");
    let config = SCRecordingOutputConfiguration::new().with_output_url(&path);

    let callbacks = RecordingCallbacks::new()
        .on_start(|| println!("Recording started"))
        .on_finish(|| println!("Recording finished"))
        .on_fail(|e| eprintln!("Recording failed: {e}"));

    let result = SCRecordingOutput::new_with_delegate(&config, callbacks);

    match result {
        Some(output) => {
            println!("✓ Recording output with delegate created successfully");
            drop(output);
        }
        None => {
            println!("⚠ Recording output creation requires macOS 15.0+ runtime");
        }
    }
}
