//! Memory safety tests
//!
//! These tests verify proper memory management across the library.
//! Run with: `cargo test --test memory_tests --features "macos_14_0"`
//!
//! For comprehensive leak detection using macOS `leaks` command,
//! run the `15_memory_leak_check` example instead:
//! `cargo run --example 15_memory_leak_check`

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Test that `SCShareableContent` properly releases memory
#[test]
fn test_shareable_content_drop() {
    // Create and drop multiple times to check for leaks
    for _ in 0..10 {
        let content = SCShareableContent::get().expect("Failed to get content");
        let _displays = content.displays();
        let _windows = content.windows();
        let _apps = content.applications();
        // content dropped here
    }
}

/// Test that cloning and dropping works correctly
#[test]
fn test_clone_and_drop() {
    let content = SCShareableContent::get().expect("Failed to get content");

    // Clone displays multiple times
    let displays = content.displays();
    if let Some(display) = displays.first() {
        let clones: Vec<_> = (0..100).map(|_| display.clone()).collect();
        drop(clones);

        // Original should still be valid
        let _ = display.display_id();
    }

    // Clone windows multiple times
    let windows = content.windows();
    if let Some(window) = windows.first() {
        let clones: Vec<_> = (0..100).map(|_| window.clone()).collect();
        drop(clones);

        // Original should still be valid
        let _ = window.window_id();
    }
}

/// Test `SCContentFilter` memory management
#[test]
fn test_content_filter_memory() {
    let content = SCShareableContent::get().expect("Failed to get content");
    let displays = content.displays();

    if let Some(display) = displays.first() {
        // Create many filters
        let filters: Vec<_> = (0..50)
            .map(|_| {
                SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build()
            })
            .collect();

        // Clone filters
        let cloned: Vec<_> = filters.clone();

        drop(cloned);
        drop(filters);
    }
}

/// Test `SCStreamConfiguration` memory management
#[test]
fn test_stream_configuration_memory() {
    // Create many configurations
    let configs: Vec<_> = (0..100)
        .map(|i: i32| {
            SCStreamConfiguration::new()
                .with_width(1920)
                .with_height(1080)
                .with_fps(30 + (i.unsigned_abs() % 30))
                .with_shows_cursor(true)
                .with_captures_audio(true)
        })
        .collect();

    // Clone configurations
    let cloned: Vec<_> = configs.clone();

    drop(cloned);
    drop(configs);
}

/// Test that stream creation and destruction doesn't leak
#[test]
fn test_stream_lifecycle() {
    let content = SCShareableContent::get().expect("Failed to get content");
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(640)
            .with_height(480);

        // Create and drop streams multiple times
        for _ in 0..5 {
            let stream = SCStream::new(&filter, &config);
            drop(stream);
        }
    }
}

/// Test handler registration and cleanup
#[test]
fn test_handler_registration_cleanup() {
    struct TestHandler {
        count: Arc<AtomicUsize>,
    }

    impl SCStreamOutputTrait for TestHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    let content = SCShareableContent::get().expect("Failed to get content");
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(640)
            .with_height(480);

        // Register and remove handlers multiple times
        for _ in 0..10 {
            let mut stream = SCStream::new(&filter, &config);
            let count = Arc::new(AtomicUsize::new(0));

            let handler = TestHandler {
                count: count.clone(),
            };
            let id = stream.add_output_handler(handler, SCStreamOutputType::Screen);

            // Remove handler
            if let Some(handler_id) = id {
                stream.remove_output_handler(handler_id, SCStreamOutputType::Screen);
            }

            drop(stream);
        }
    }
}

/// Test closure handler memory management
#[test]
fn test_closure_handler_memory() {
    let content = SCShareableContent::get().expect("Failed to get content");
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(640)
            .with_height(480);

        for _ in 0..10 {
            let mut stream = SCStream::new(&filter, &config);
            let count = Arc::new(AtomicUsize::new(0));
            let count_clone = count.clone();

            stream.add_output_handler(
                move |_sample: CMSampleBuffer, _of_type: SCStreamOutputType| {
                    count_clone.fetch_add(1, Ordering::Relaxed);
                },
                SCStreamOutputType::Screen,
            );

            drop(stream);
            // count should be droppable after stream is dropped
            drop(count);
        }
    }
}

/// Test `CGRect` and geometry types don't leak
#[test]
fn test_geometry_types() {
    use screencapturekit::cg::{CGPoint, CGRect, CGSize};

    // These are Copy types, but let's verify they work correctly
    for _ in 0..1000 {
        let point = CGPoint { x: 100.0, y: 200.0 };
        let size = CGSize {
            width: 1920.0,
            height: 1080.0,
        };
        let rect = CGRect::new(point.x, point.y, size.width, size.height);

        let _ = rect.x;
        let _ = rect.y;
        let _ = rect.width;
        let _ = rect.height;
    }
}

/// Test `CMTime` doesn't leak
#[test]
fn test_cmtime_memory() {
    use screencapturekit::cm::CMTime;

    for _ in 0..1000 {
        let time = CMTime::new(1, 30);
        let _ = time.as_seconds();
        let _ = time.is_valid();

        let time2 = CMTime::new(0, 1);
        let _ = time2.is_zero();
    }
}

/// Test `DispatchQueue` memory management
#[test]
fn test_dispatch_queue_memory() {
    use screencapturekit::dispatch_queue::{DispatchQoS, DispatchQueue};

    for i in 0..20 {
        let queue = DispatchQueue::new(&format!("com.test.queue.{i}"), DispatchQoS::Default);
        let cloned = queue.clone();
        drop(cloned);
        drop(queue);
    }
}

/// Test that multiple streams with handlers don't leak
#[test]
fn test_multiple_streams_memory() {
    let content = SCShareableContent::get().expect("Failed to get content");
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(320)
            .with_height(240);

        // Create multiple streams simultaneously
        let streams: Vec<_> = (0..5)
            .map(|_| {
                let mut stream = SCStream::new(&filter, &config);
                let count = Arc::new(AtomicUsize::new(0));
                let count_clone = count.clone();

                stream.add_output_handler(
                    move |_: CMSampleBuffer, _: SCStreamOutputType| {
                        count_clone.fetch_add(1, Ordering::Relaxed);
                    },
                    SCStreamOutputType::Screen,
                );

                (stream, count)
            })
            .collect();

        // Drop all streams
        drop(streams);
    }
}

/// Test window filter creation doesn't leak
#[test]
fn test_window_filter_memory() {
    let content = SCShareableContent::get().expect("Failed to get content");
    let windows = content.windows();

    if let Some(window) = windows.first() {
        // Create many window filters
        let filters: Vec<_> = (0..50)
            .map(|_| SCContentFilter::builder().window(window).build())
            .collect();

        drop(filters);
    }
}

/// Test that audio configuration doesn't leak
#[test]
fn test_audio_config_memory() {
    for _ in 0..100 {
        let config = SCStreamConfiguration::new()
            .with_captures_audio(true)
            .with_sample_rate(48000)
            .with_channel_count(2)
            .with_captures_microphone(true)
            .with_excludes_current_process_audio(true);

        let _ = config.captures_audio();
        let _ = config.sample_rate();
        let _ = config.channel_count();

        drop(config);
    }
}

#[cfg(feature = "macos_14_0")]
mod macos_14_tests {
    use super::*;
    use screencapturekit::shareable_content::SCShareableContentInfo;

    /// Test `SCShareableContentInfo` memory management
    #[test]
    fn test_content_info_memory() {
        let content = SCShareableContent::get().expect("Failed to get content");
        let displays = content.displays();

        if let Some(display) = displays.first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();

            // Create and drop content info multiple times
            for _ in 0..20 {
                if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
                    let _ = info.style();
                    let _ = info.point_pixel_scale();
                    let _ = info.pixel_size();
                    // info dropped here
                }
            }
        }
    }

    /// Test `SCContentSharingPickerConfiguration` memory
    #[test]
    fn test_picker_config_memory() {
        use screencapturekit::content_sharing_picker::{
            SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
        };

        for _ in 0..50 {
            let mut config = SCContentSharingPickerConfiguration::new();
            config.set_allowed_picker_modes(&[
                SCContentSharingPickerMode::SingleWindow,
                SCContentSharingPickerMode::SingleDisplay,
            ]);

            let cloned = config.clone();
            drop(cloned);
            drop(config);
        }
    }
}

#[cfg(feature = "macos_15_0")]
mod macos_15_tests {
    /// Test `SCRecordingOutputConfiguration` memory
    #[test]
    fn test_recording_config_memory() {
        use screencapturekit::recording_output::{
            SCRecordingOutputCodec, SCRecordingOutputConfiguration, SCRecordingOutputFileType,
        };

        for _ in 0..50 {
            let config = SCRecordingOutputConfiguration::new()
                .with_video_codec(SCRecordingOutputCodec::H264)
                .with_output_file_type(SCRecordingOutputFileType::MP4);

            let _ = config.video_codec();
            let _ = config.output_file_type();

            let cloned = config.clone();
            drop(cloned);
            drop(config);
        }
    }
}
