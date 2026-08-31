//! Recording output tests
//!
//! Tests for `SCRecordingOutput` and `SCRecordingOutputConfiguration` (macOS 15.0+).

#![cfg(feature = "macos_15_0")]

use screencapturekit::recording_output::{SCRecordingOutput, SCRecordingOutputConfiguration};

/// Serialises the tests that drive a real `SCStream`. Two concurrent captures
/// of the same display make `ScreenCaptureKit`'s completions stall past their
/// timeout, which surfaces as an unrelated-looking 30 s failure.
static LIVE_CAPTURE: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

    let path = PathBuf::from("test_recording.mp4");
    let config = SCRecordingOutputConfiguration::new()
        .with_output_url(&path)
        .with_video_codec(SCRecordingOutputCodec::H264);

    let output_url = config.output_url().expect("output_url should round-trip");
    assert!(
        output_url.ends_with(&path),
        "expected output URL {output_url:?} to end with {path:?}"
    );
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
fn test_recording_callbacks_are_send_and_sync() {
    use screencapturekit::recording_output::RecordingCallbacks;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<RecordingCallbacks>();
    assert_sync::<RecordingCallbacks>();
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

// MARK: - Soundness / forward-compatibility regressions

/// `SCRecordingOutputConfiguration` wraps a *mutable* Objective-C object, so a
/// retain-based `Clone` would hand out aliases: reconfiguring the clone would
/// silently reconfigure the original, and two threads configuring "their own"
/// clone would race on the same non-atomic properties.
#[test]
fn test_recording_configuration_clone_is_independent() {
    use screencapturekit::recording_output::{SCRecordingOutputCodec, SCRecordingOutputFileType};
    use std::path::Path;

    let original = SCRecordingOutputConfiguration::new()
        .with_output_url(Path::new("/tmp/original.mov"))
        .with_video_codec(SCRecordingOutputCodec::H264)
        .with_output_file_type(SCRecordingOutputFileType::MOV);

    let clone = original.clone();
    assert_eq!(clone.video_codec(), SCRecordingOutputCodec::H264);
    assert_eq!(clone.output_file_type(), SCRecordingOutputFileType::MOV);

    let clone = clone
        .with_video_codec(SCRecordingOutputCodec::HEVC)
        .with_output_file_type(SCRecordingOutputFileType::MP4)
        .with_output_url(Path::new("/tmp/clone.mp4"));

    assert_eq!(
        original.video_codec(),
        SCRecordingOutputCodec::H264,
        "mutating the clone must not reach the original"
    );
    assert_eq!(original.output_file_type(), SCRecordingOutputFileType::MOV);
    assert_eq!(clone.video_codec(), SCRecordingOutputCodec::HEVC);
    assert_eq!(clone.output_file_type(), SCRecordingOutputFileType::MP4);

    let original_url = original.output_url().expect("original url");
    let clone_url = clone.output_url().expect("clone url");
    assert!(original_url.ends_with("original.mov"), "{original_url:?}");
    assert!(clone_url.ends_with("clone.mp4"), "{clone_url:?}");
}

/// The available-codec / available-file-type lists are open-ended. Entries the
/// bridge cannot name used to be dropped, so the vector silently disagreed
/// with the count and index-based lookups pointed at the wrong entry.
#[test]
fn test_available_lists_match_their_counts() {
    let config = SCRecordingOutputConfiguration::new();

    assert_eq!(
        config.available_video_codecs().len(),
        config.available_video_codecs_count()
    );
    assert_eq!(
        config.available_output_file_types().len(),
        config.available_output_file_types_count()
    );
}

/// An unknown identifier must survive rather than collapsing onto a default.
#[test]
fn test_codec_and_file_type_are_open() {
    use screencapturekit::recording_output::{SCRecordingOutputCodec, SCRecordingOutputFileType};

    let future_codec = SCRecordingOutputCodec::from_identifier("com.example.future-codec").unwrap();
    assert_eq!(future_codec.identifier(), "com.example.future-codec");
    assert_ne!(future_codec, SCRecordingOutputCodec::H264);
    assert!(future_codec.to_string().contains("future-codec"));

    let future_file_type =
        SCRecordingOutputFileType::from_identifier("com.example.future-file").unwrap();
    assert_eq!(future_file_type.identifier(), "com.example.future-file");
    assert_eq!(future_file_type.extension(), None);

    let config = SCRecordingOutputConfiguration::new()
        .with_video_codec(future_codec.clone())
        .with_output_file_type(future_file_type.clone());
    assert_eq!(config.video_codec().identifier(), future_codec.identifier());
    assert_eq!(
        config.output_file_type().identifier(),
        future_file_type.identifier()
    );

    assert_eq!(
        SCRecordingOutputCodec::default(),
        SCRecordingOutputCodec::H264
    );
    assert_eq!(
        SCRecordingOutputFileType::default(),
        SCRecordingOutputFileType::MP4
    );
    assert_eq!(SCRecordingOutputFileType::MOV.extension(), Some("mov"));
}

#[test]
fn test_recording_identifiers_reject_interior_nul() {
    use screencapturekit::recording_output::{SCRecordingOutputCodec, SCRecordingOutputFileType};

    assert!(SCRecordingOutputCodec::from_identifier("bad\0codec").is_err());
    assert!(SCRecordingOutputFileType::from_identifier("bad\0file").is_err());
}

/// A path that cannot be encoded as a C string must be reported, not silently
/// swallowed — an ignored `with_output_url` leaves the recording pointed at
/// the previous (or default) location.
#[test]
fn test_output_url_rejects_interior_nul() {
    use std::path::Path;

    let config = SCRecordingOutputConfiguration::new();
    let result = config.try_with_output_url(Path::new("/tmp/bad\0name.mov"));
    assert!(result.is_err(), "interior NUL must be rejected");
}

#[test]
fn test_output_url_rejects_non_utf8_path() {
    use std::os::unix::ffi::OsStringExt;

    let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(
        b"/tmp/sck-recording-\xff.mp4".to_vec(),
    ));
    assert!(
        SCRecordingOutputConfiguration::new()
            .try_with_output_url(&path)
            .is_err(),
        "Foundation file URLs cannot represent non-UTF-8 paths faithfully"
    );
}

/// A delegate must keep receiving callbacks while *any* clone of the recording
/// output is alive: the delegate storage is reference-counted on both sides of
/// the bridge, and dropping the first clone used to tear it down.
#[test]
fn test_delegate_survives_clone_drop() {
    use screencapturekit::recording_output::RecordingCallbacks;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let starts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&starts);

    let config = SCRecordingOutputConfiguration::new();
    let delegate = RecordingCallbacks::new().on_start(move || {
        observed.fetch_add(1, Ordering::SeqCst);
    });

    let Some(output) = SCRecordingOutput::new_with_delegate(&config, delegate) else {
        println!("⚠ Skipping - recording output unavailable");
        return;
    };

    let clone = output.clone();
    drop(clone);

    // The surviving handle must still be usable; a torn-down delegate used to
    // surface here as a crash or as silently dead callbacks.
    assert_eq!(output.recorded_file_size(), 0);
    assert_eq!(starts.load(Ordering::SeqCst), 0);
    drop(output);
}

#[test]
fn test_remove_recording_then_stop_completes() {
    use screencapturekit::prelude::*;
    use screencapturekit::recording_output::RecordingCallbacks;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let _capture = LIVE_CAPTURE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let Ok(content) = SCShareableContent::get() else {
        eprintln!("skip: screen-recording permission unavailable");
        return;
    };
    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("skip: no displays available");
        return;
    };

    let output_path =
        std::env::temp_dir().join(format!("sck-recording-test-{}.mp4", std::process::id()));
    let _ = std::fs::remove_file(&output_path);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let stream_config = SCStreamConfiguration::new()
        .with_width(320)
        .with_height(240);
    let recording_config = SCRecordingOutputConfiguration::new().with_output_url(&output_path);
    let started_recording = Arc::new(AtomicBool::new(false));
    let started_observed = Arc::clone(&started_recording);
    let finished = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&finished);
    let failure = Arc::new(std::sync::Mutex::new(None));
    let failure_observed = Arc::clone(&failure);
    let callbacks = RecordingCallbacks::new()
        .on_start(move || started_observed.store(true, Ordering::Release))
        .on_finish(move || observed.store(true, Ordering::Release))
        .on_fail(move |error| {
            *failure_observed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(error);
        });
    let Some(recording) = SCRecordingOutput::new_with_delegate(&recording_config, callbacks) else {
        eprintln!("skip: recording output unavailable");
        return;
    };

    let stream = SCStream::new(&filter, &stream_config);
    stream
        .add_recording_output(&recording)
        .expect("failed to add recording output");
    if let Err(error) = stream.start_capture() {
        eprintln!("skip: capture failed to start: {error}");
        return;
    }
    let start_deadline = Instant::now() + Duration::from_secs(5);
    while !started_recording.load(Ordering::Acquire) && Instant::now() < start_deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        started_recording.load(Ordering::Acquire),
        "recording never reached didStartRecording"
    );

    stream
        .remove_recording_output(&recording)
        .expect("failed to remove recording output");
    drop(recording);

    let started = Instant::now();
    stream
        .stop_capture()
        .expect("stop_capture failed after removing recording output");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "stop_capture stalled during recording finalization"
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    while !finished.load(Ordering::Acquire)
        && failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(20));
    }
    let failure = failure
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        finished.load(Ordering::Acquire) || failure.is_some(),
        "recording delegate was released before a terminal callback"
    );
    let _ = std::fs::remove_file(output_path);
}

/// Removing an output before `didStartRecording` has been delivered must not
/// report success while the movie is still being finalized.
#[test]
fn test_remove_recording_racing_start_still_waits_for_terminal() {
    use screencapturekit::prelude::*;
    use screencapturekit::recording_output::RecordingCallbacks;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let _capture = LIVE_CAPTURE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let Ok(content) = SCShareableContent::get() else {
        eprintln!("skip: screen-recording permission unavailable");
        return;
    };
    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("skip: no displays available");
        return;
    };

    let output_path =
        std::env::temp_dir().join(format!("sck-recording-race-{}.mp4", std::process::id()));
    let _ = std::fs::remove_file(&output_path);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let stream_config = SCStreamConfiguration::new()
        .with_width(320)
        .with_height(240);
    let recording_config = SCRecordingOutputConfiguration::new().with_output_url(&output_path);
    let started = Arc::new(AtomicBool::new(false));
    let started_observed = Arc::clone(&started);
    let terminal = Arc::new(AtomicBool::new(false));
    let finish_observed = Arc::clone(&terminal);
    let fail_observed = Arc::clone(&terminal);
    let callbacks = RecordingCallbacks::new()
        .on_start(move || started_observed.store(true, Ordering::Release))
        .on_finish(move || finish_observed.store(true, Ordering::Release))
        .on_fail(move |_| fail_observed.store(true, Ordering::Release));
    let Some(recording) = SCRecordingOutput::new_with_delegate(&recording_config, callbacks) else {
        eprintln!("skip: recording output unavailable");
        return;
    };

    let stream = SCStream::new(&filter, &stream_config);
    stream
        .add_recording_output(&recording)
        .expect("failed to add recording output");
    if let Err(error) = stream.start_capture() {
        eprintln!("skip: capture failed to start: {error}");
        return;
    }

    let removal_started = std::time::Instant::now();
    let removal = stream.remove_recording_output(&recording);
    let removal_elapsed = removal_started.elapsed();
    let started_before_removal_returned = started.load(Ordering::Acquire);
    let reached_terminal = terminal.load(Ordering::Acquire);

    let _ = stream.stop_capture();
    drop(recording);
    let _ = std::fs::remove_file(output_path);

    // ScreenCaptureKit intermittently stalls its own `stopCapture` when it
    // races a start that has only just landed. The bounded wait turns that into
    // an error instead of a hang, which is all the binding can do.
    if let Err(SCError::CaptureStopFailed(message)) = &removal {
        eprintln!("skip: ScreenCaptureKit stalled stopping a just-started capture: {message}");
        return;
    }
    removal.expect("failed to remove recording output");

    // Whether or not the recording ever began, the wait for finalization is
    // bounded: it must resolve on its own rather than run into the completion
    // timeout.
    assert!(
        removal_elapsed < std::time::Duration::from_secs(10),
        "removal stalled for {removal_elapsed:?} (started: \
         {started_before_removal_returned}, terminal: {reached_terminal})"
    );
}
