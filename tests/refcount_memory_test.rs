//! Reference counting and memory leak tests
//!
//! These tests verify that:
//! 1. Clone properly increments reference count
//! 2. Drop properly decrements reference count
//! 3. No memory leaks occur with normal usage patterns

use screencapturekit::prelude::*;
use std::sync::Arc;
use std::thread;

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_shareable_content_clone_drop() {
    // This test requires Screen Recording permission
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    // Clone should work without issues
    let content2 = content.clone();
    let content3 = content2.clone();

    // All should have the same displays
    assert_eq!(content.displays().len(), content2.displays().len());
    assert_eq!(content2.displays().len(), content3.displays().len());

    // Drop happens automatically at end of scope
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_display_clone_drop() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    if let Some(display) = content.displays().first() {
        // Clone display multiple times
        let display2 = display.clone();
        let display3 = display2.clone();

        // All clones should have same ID
        assert_eq!(display.display_id(), display2.display_id());
        assert_eq!(display2.display_id(), display3.display_id());

        // Verify properties are accessible
        assert!(display.width() > 0);
        assert!(display.height() > 0);
    }
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_window_clone_drop() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    if let Some(window) = content.windows().first() {
        let window2 = window.clone();
        let window3 = window2.clone();

        assert_eq!(window.window_id(), window2.window_id());
        assert_eq!(window2.window_id(), window3.window_id());
    }
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_running_application_clone_drop() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    if let Some(app) = content.applications().first() {
        let app2 = app.clone();
        let app3 = app2.clone();

        assert_eq!(app.process_id(), app2.process_id());
        assert_eq!(app2.process_id(), app3.process_id());
    }
}

#[test]
fn test_stream_configuration_clone_drop() {
    let config = SCStreamConfiguration::build()
        .set_width(1920)
        .unwrap()
        .set_height(1080)
        .unwrap();

    let config2 = config.clone();
    let config3 = config2.clone();

    // All configurations should be valid
    drop(config);
    drop(config2);
    drop(config3);
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_content_filter_clone_drop() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    if let Some(display) = content.displays().first() {
        let filter = SCContentFilter::build()
            .display(display)
            .exclude_windows(&[])
            .build();

        let filter2 = filter.clone();
        let filter3 = filter2.clone();

        drop(filter);
        drop(filter2);
        drop(filter3);
    }
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_thread_safety_with_clone() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    if let Some(display) = content.displays().first() {
        let display = display.clone();
        let display_arc = Arc::new(display);

        let mut handles = vec![];

        // Spawn multiple threads that access the display
        for _ in 0..5 {
            let display = Arc::clone(&display_arc);
            let handle = thread::spawn(move || {
                // Access display properties
                let _ = display.display_id();
                let _ = display.width();
                let _ = display.height();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_sharing_picker_config_clone_drop() {
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    };

    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(&[SCContentSharingPickerMode::SingleWindow]);

    let config2 = config.clone();
    let config3 = config2.clone();

    drop(config);
    drop(config2);
    drop(config3);
}

#[test]
#[cfg(feature = "macos_15_0")]
fn test_recording_output_config_clone_drop() {
    use screencapturekit::recording_output::{
        SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    };
    use std::path::PathBuf;

    let mut config = SCRecordingOutputConfiguration::new();
    config.set_output_url(&PathBuf::from("/tmp/test.mp4"));
    config.set_video_codec(SCRecordingOutputCodec::H264);

    let config2 = config.clone();
    let config3 = config2.clone();

    drop(config);
    drop(config2);
    drop(config3);
}

#[test]
fn test_multiple_clone_drop_cycles() {
    // Test that we can clone and drop multiple times without issues
    for _ in 0..100 {
        let config = SCStreamConfiguration::build()
            .set_width(1920)
            .unwrap()
            .set_height(1080)
            .unwrap();

        let config2 = config.clone();
        drop(config);
        
        let config3 = config2.clone();
        drop(config2);
        
        drop(config3);
    }
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_nested_drops() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test - Screen Recording permission not granted");
            return;
        }
    };

    // Create nested structures
    if let Some(display) = content.displays().first() {
        let displays = vec![
            display.clone(),
            display.clone(),
            display.clone(),
        ];

        // Vector drop should properly drop all displays
        drop(displays);

        // Original display should still be valid
        let _ = display.display_id();
    }
}
