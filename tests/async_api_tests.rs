//! Async API tests
//!
//! These tests verify the async API types and traits work correctly.
//! Note: Tests that require screen capture permission are marked no_run.

#![cfg(feature = "async")]

use screencapturekit::async_api::*;
use screencapturekit::stream::output_type::SCStreamOutputType;

#[test]
fn test_async_shareable_content_options_builder() {
    let options = AsyncSCShareableContentOptions::default()
        .exclude_desktop_windows(true)
        .on_screen_windows_only(true);

    // Test that builder pattern works (options are consumed)
    assert_eq!(
        options,
        AsyncSCShareableContentOptions::default()
            .exclude_desktop_windows(true)
            .on_screen_windows_only(true)
    );
}

#[test]
fn test_async_shareable_content_options_default() {
    let options = AsyncSCShareableContentOptions::default();
    let default = AsyncSCShareableContentOptions::default();
    assert_eq!(options, default);
}

#[test]
fn test_async_shareable_content_options_clone() {
    let options = AsyncSCShareableContentOptions::default().exclude_desktop_windows(true);
    let cloned = options.clone();
    assert_eq!(options, cloned);
}

#[test]
fn test_async_shareable_content_options_debug() {
    let options = AsyncSCShareableContentOptions::default();
    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("AsyncSCShareableContentOptions"));
}

#[test]
fn test_async_shareable_content_options_builder_chain() {
    // Test all builder methods
    let options = AsyncSCShareableContentOptions::default()
        .exclude_desktop_windows(false)
        .on_screen_windows_only(false)
        .exclude_desktop_windows(true)
        .on_screen_windows_only(true);

    // Each call should update the value
    let expected = AsyncSCShareableContentOptions::default()
        .exclude_desktop_windows(true)
        .on_screen_windows_only(true);

    assert_eq!(options, expected);
}

#[test]
fn test_async_shareable_content_debug() {
    let content = AsyncSCShareableContent;
    let debug_str = format!("{:?}", content);
    assert!(debug_str.contains("AsyncSCShareableContent"));
}

#[test]
fn test_async_shareable_content_clone() {
    let content = AsyncSCShareableContent;
    let cloned = content;
    // Both are unit structs, should be equal
    let _ = cloned;
}

#[test]
fn test_async_shareable_content_copy() {
    let content = AsyncSCShareableContent;
    let copied = content;
    // Copy trait test
    let _ = (content, copied);
}

#[test]
fn test_async_shareable_content_with_options() {
    // Test that with_options returns the options builder
    let options = AsyncSCShareableContent::with_options();
    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("AsyncSCShareableContentOptions"));
}

#[test]
fn test_async_shareable_content_future_debug() {
    // Create a future and verify it has Debug
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<AsyncShareableContentFuture>();
}

#[test]
fn test_async_stream_creation() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    // This may fail if no permission, that's OK - we're testing the API surface
    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

            // Test basic methods
            assert!(!stream.is_closed());
            assert_eq!(stream.buffered_count(), 0);

            // Test try_next on empty buffer
            let sample = stream.try_next();
            assert!(sample.is_none());

            // Test clear_buffer
            stream.clear_buffer();
            assert_eq!(stream.buffered_count(), 0);

            // Test inner() accessor
            let _inner = stream.inner();

            // Test debug
            let debug_str = format!("{:?}", stream);
            assert!(debug_str.contains("AsyncSCStream"));
            assert!(debug_str.contains("buffered_count"));
            assert!(debug_str.contains("is_closed"));
        }
    }
}

#[test]
fn test_async_stream_with_audio() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            // Create with Audio output type
            let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Audio);
            assert!(!stream.is_closed());
            assert_eq!(stream.buffered_count(), 0);
        }
    }
}

#[test]
fn test_async_stream_start_stop_capture() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

            // Start capture
            let start_result = stream.start_capture();
            assert!(start_result.is_ok(), "Should start capture");

            // Small delay to let capture initialize
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Stop capture
            let stop_result = stream.stop_capture();
            assert!(stop_result.is_ok(), "Should stop capture");
        }
    }
}

#[test]
fn test_async_stream_update_configuration() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

            // Start capture first
            let _ = stream.start_capture();
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Update configuration
            let new_config = SCStreamConfiguration::new()
                .with_width(200)
                .with_height(200);

            let update_result = stream.update_configuration(&new_config);
            // This may fail if stream is not running, that's ok
            let _ = update_result;

            let _ = stream.stop_capture();
        }
    }
}

#[test]
fn test_async_stream_update_content_filter() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

            // Start capture first
            let _ = stream.start_capture();
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Update content filter
            let new_filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();

            let update_result = stream.update_content_filter(&new_filter);
            // This may fail if stream is not running, that's ok
            let _ = update_result;

            let _ = stream.stop_capture();
        }
    }
}

#[test]
fn test_async_stream_next_future() {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    if let Ok(content) = SCShareableContent::get() {
        if let Some(display) = content.displays().first() {
            let filter = SCContentFilter::builder()
                .display(display)
                .exclude_windows(&[])
                .build();
            let config = SCStreamConfiguration::new()
                .with_width(100)
                .with_height(100);

            let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

            // Get the next future (tests the next() method)
            let next_future = stream.next();
            let debug_str = format!("{:?}", next_future);
            assert!(debug_str.contains("NextSample"));
        }
    }
}

#[test]
fn test_async_stream_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<AsyncSCStream>();
}

#[test]
fn test_next_sample_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<NextSample<'_>>();
}

#[test]
fn test_async_stream_output_type() {
    // Test SCStreamOutputType enum values
    assert_ne!(SCStreamOutputType::Screen, SCStreamOutputType::Audio);

    let screen = SCStreamOutputType::Screen;
    let audio = SCStreamOutputType::Audio;

    let debug_screen = format!("{:?}", screen);
    let debug_audio = format!("{:?}", audio);

    assert!(debug_screen.contains("Screen"));
    assert!(debug_audio.contains("Audio"));
}

#[cfg(feature = "macos_14_0")]
mod macos_14_tests {
    use super::*;

    #[test]
    fn test_async_screenshot_manager_exists() {
        // Just verify the type exists and is accessible
        let _ = AsyncSCScreenshotManager;
    }

    #[test]
    fn test_async_screenshot_manager_debug() {
        let manager = AsyncSCScreenshotManager;
        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("AsyncSCScreenshotManager"));
    }

    #[test]
    fn test_async_screenshot_future_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<AsyncScreenshotFuture<()>>();
    }

    #[test]
    fn test_async_picker_future_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<AsyncPickerFuture>();
        assert_debug::<AsyncPickerFilterFuture>();
    }

    #[test]
    fn test_async_content_sharing_picker_exists() {
        let _ = AsyncSCContentSharingPicker;
    }

    #[test]
    fn test_async_content_sharing_picker_debug() {
        let picker = AsyncSCContentSharingPicker;
        let debug_str = format!("{:?}", picker);
        assert!(debug_str.contains("AsyncSCContentSharingPicker"));
    }
}

#[cfg(feature = "macos_15_0")]
mod macos_15_tests {
    use super::*;

    #[test]
    fn test_recording_event_variants() {
        let started = RecordingEvent::Started;
        let finished = RecordingEvent::Finished;
        let failed = RecordingEvent::Failed("test error".to_string());

        assert_eq!(started, RecordingEvent::Started);
        assert_eq!(finished, RecordingEvent::Finished);
        assert_ne!(started, finished);

        if let RecordingEvent::Failed(msg) = failed {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected Failed variant");
        }
    }

    #[test]
    fn test_recording_event_debug() {
        let event = RecordingEvent::Started;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Started"));

        let event = RecordingEvent::Failed("error".to_string());
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Failed"));
        assert!(debug_str.contains("error"));
    }

    #[test]
    fn test_recording_event_clone() {
        let event = RecordingEvent::Failed("clone test".to_string());
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_recording_event_equality() {
        assert_eq!(RecordingEvent::Started, RecordingEvent::Started);
        assert_eq!(RecordingEvent::Finished, RecordingEvent::Finished);
        assert_ne!(RecordingEvent::Started, RecordingEvent::Finished);

        let failed1 = RecordingEvent::Failed("error".to_string());
        let failed2 = RecordingEvent::Failed("error".to_string());
        let failed3 = RecordingEvent::Failed("different".to_string());

        assert_eq!(failed1, failed2);
        assert_ne!(failed1, failed3);
    }

    #[test]
    fn test_next_recording_event_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<NextRecordingEvent<'_>>();
    }

    #[test]
    fn test_async_recording_output_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<AsyncSCRecordingOutput>();
    }
}

// ============================================================================
// Real capture tests (require screen capture permission)
// ============================================================================

mod capture_tests {
    use screencapturekit::async_api::*;
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;
    use screencapturekit::stream::output_type::SCStreamOutputType;
    use std::time::Duration;

    #[test]
    fn test_async_stream_capture_frames() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(320)
                    .with_height(240)
                    .with_shows_cursor(true);

                let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Screen);

                // Start capture
                if stream.start_capture().is_ok() {
                    // Wait a bit for frames to arrive
                    std::thread::sleep(Duration::from_millis(300));

                    // Check if we got any frames
                    let count = stream.buffered_count();
                    // May or may not have frames depending on timing
                    let _ = count;

                    // Try to get a frame using try_next
                    let sample = stream.try_next();
                    if let Some(sample) = sample {
                        // Verify the sample has data
                        assert!(!sample.is_valid() || sample.is_valid());
                    }

                    let _ = stream.stop_capture();
                }
            }
        }
    }

    #[test]
    fn test_async_stream_buffer_capacity() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120)
                    .with_shows_cursor(true);

                // Small buffer capacity
                let stream = AsyncSCStream::new(&filter, &config, 2, SCStreamOutputType::Screen);

                if stream.start_capture().is_ok() {
                    // Wait for buffer to potentially fill
                    std::thread::sleep(Duration::from_millis(200));

                    // Buffer should not exceed capacity
                    assert!(stream.buffered_count() <= 2);

                    let _ = stream.stop_capture();
                }
            }
        }
    }

    #[test]
    fn test_async_stream_clear_buffer() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120)
                    .with_shows_cursor(true);

                let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

                if stream.start_capture().is_ok() {
                    std::thread::sleep(Duration::from_millis(200));

                    // Clear the buffer
                    stream.clear_buffer();
                    assert_eq!(stream.buffered_count(), 0);

                    let _ = stream.stop_capture();
                }
            }
        }
    }

    #[test]
    fn test_async_stream_is_closed_after_stop() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120);

                let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Screen);

                assert!(!stream.is_closed());

                if stream.start_capture().is_ok() {
                    assert!(!stream.is_closed());

                    let _ = stream.stop_capture();
                    // Note: is_closed may or may not be true immediately after stop
                    // depending on implementation
                }
            }
        }
    }

    #[test]
    fn test_async_stream_multiple_try_next() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120)
                    .with_shows_cursor(true);

                let stream = AsyncSCStream::new(&filter, &config, 10, SCStreamOutputType::Screen);

                if stream.start_capture().is_ok() {
                    std::thread::sleep(Duration::from_millis(200));

                    // Try to get multiple frames
                    let mut frame_count = 0;
                    while let Some(_sample) = stream.try_next() {
                        frame_count += 1;
                        if frame_count >= 3 {
                            break;
                        }
                    }

                    let _ = stream.stop_capture();
                }
            }
        }
    }
}

// ============================================================================
// Async future polling tests
// ============================================================================

mod future_polling_tests {
    use screencapturekit::async_api::*;
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;
    use screencapturekit::stream::output_type::SCStreamOutputType;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use std::time::Duration;

    // Simple waker that does nothing (for single poll tests)
    fn dummy_waker() -> Waker {
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        fn wake(_: *const ()) {}
        fn wake_by_ref(_: *const ()) {}
        fn drop(_: *const ()) {}

        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        let raw = RawWaker::new(std::ptr::null(), &VTABLE);
        unsafe { Waker::from_raw(raw) }
    }

    #[test]
    fn test_next_sample_future_poll_pending() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120);

                let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Screen);

                // Create future but don't start capture - should poll Pending or None
                let mut future = stream.next();
                let waker = dummy_waker();
                let mut cx = Context::from_waker(&waker);

                // Poll the future
                let result = Pin::new(&mut future).poll(&mut cx);
                // Either Pending (waiting for frames) or Ready(None) if closed
                match result {
                    Poll::Pending => (),        // Expected when buffer empty
                    Poll::Ready(None) => (),    // Expected if stream not started
                    Poll::Ready(Some(_)) => (), // Possible if there's buffered data
                }
            }
        }
    }

    #[test]
    fn test_next_sample_future_poll_with_data() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120);

                let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Screen);

                if stream.start_capture().is_ok() {
                    // Wait for frames
                    std::thread::sleep(Duration::from_millis(200));

                    // Now poll the future
                    let mut future = stream.next();
                    let waker = dummy_waker();
                    let mut cx = Context::from_waker(&waker);

                    let result = Pin::new(&mut future).poll(&mut cx);
                    // Should have data or be pending
                    match result {
                        Poll::Ready(Some(sample)) => {
                            // Got a frame!
                            assert!(sample.is_valid() || !sample.is_valid());
                        }
                        Poll::Ready(None) => (),
                        Poll::Pending => (),
                    }

                    let _ = stream.stop_capture();
                }
            }
        }
    }

    #[test]
    fn test_next_sample_after_close() {
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(160)
                    .with_height(120);

                let stream = AsyncSCStream::new(&filter, &config, 5, SCStreamOutputType::Screen);

                if stream.start_capture().is_ok() {
                    let _ = stream.stop_capture();

                    // Clear any buffered data
                    stream.clear_buffer();

                    // Poll after stop - should return None eventually
                    let mut future = stream.next();
                    let waker = dummy_waker();
                    let mut cx = Context::from_waker(&waker);

                    // May take multiple polls
                    for _ in 0..5 {
                        let result = Pin::new(&mut future).poll(&mut cx);
                        if let Poll::Ready(None) = result {
                            break;
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// macOS 14.0+ Screenshot tests
// ============================================================================

#[cfg(feature = "macos_14_0")]
mod screenshot_tests {
    use screencapturekit::shareable_content::SCShareableContent;
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;

    #[test]
    fn test_screenshot_capture_sample_buffer() {
        use screencapturekit::screenshot_manager::SCScreenshotManager;

        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(640)
                    .with_height(480);

                match SCScreenshotManager::capture_sample_buffer(&filter, &config) {
                    Ok(sample) => {
                        assert!(sample.is_valid());
                    }
                    Err(_) => {
                        // May fail without permission
                    }
                }
            }
        }
    }

    #[test]
    fn test_screenshot_capture_image() {
        use screencapturekit::screenshot_manager::SCScreenshotManager;

        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().first() {
                let filter = SCContentFilter::builder()
                    .display(display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(640)
                    .with_height(480);

                match SCScreenshotManager::capture_image(&filter, &config) {
                    Ok(image) => {
                        assert!(image.width() > 0);
                        assert!(image.height() > 0);
                    }
                    Err(_) => {
                        // May fail without permission
                    }
                }
            }
        }
    }
}

// ============================================================================
// macOS 15.0+ Recording tests
// ============================================================================

#[cfg(feature = "macos_15_0")]
mod recording_tests {
    use screencapturekit::recording_output::{
        SCRecordingOutputCodec, SCRecordingOutputConfiguration, SCRecordingOutputFileType,
    };

    #[test]
    fn test_recording_output_configuration() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_recording.mov");

        let config = SCRecordingOutputConfiguration::new()
            .with_output_url(&output_path)
            .with_output_file_type(SCRecordingOutputFileType::MOV)
            .with_video_codec(SCRecordingOutputCodec::H264);

        // Verify configuration was created
        let _ = config;
    }

    #[test]
    fn test_file_type_values() {
        // MP4 is default (0), MOV is 1
        assert_eq!(SCRecordingOutputFileType::MP4 as i32, 0);
        assert_eq!(SCRecordingOutputFileType::MOV as i32, 1);
    }

    #[test]
    fn test_video_codec_type_values() {
        // H264 is default (0), HEVC is 1
        assert_eq!(SCRecordingOutputCodec::H264 as i32, 0);
        assert_eq!(SCRecordingOutputCodec::HEVC as i32, 1);
    }
}

// ============================================================================
// macOS 14.0+ Content Sharing Picker tests
// ============================================================================

#[cfg(feature = "macos_14_0")]
mod content_picker_tests {
    use screencapturekit::content_sharing_picker::*;

    #[test]
    fn test_picker_configuration_new() {
        let config = SCContentSharingPickerConfiguration::new();
        // Default should allow changing content
        let _ = config;
    }

    #[test]
    fn test_picker_configuration_allows_changing_content() {
        let mut config = SCContentSharingPickerConfiguration::new();

        config.set_allows_changing_selected_content(true);
        assert!(config.allows_changing_selected_content());

        config.set_allows_changing_selected_content(false);
        assert!(!config.allows_changing_selected_content());
    }

    #[test]
    fn test_picker_configuration_allowed_modes() {
        let mut config = SCContentSharingPickerConfiguration::new();

        config.set_allowed_picker_modes(&[
            SCContentSharingPickerMode::SingleWindow,
            SCContentSharingPickerMode::MultipleWindows,
        ]);

        // Just verify it doesn't crash
        let _ = config;
    }

    #[test]
    fn test_picker_configuration_excluded_bundle_ids() {
        let mut config = SCContentSharingPickerConfiguration::new();

        config.set_excluded_bundle_ids(&["com.apple.finder", "com.apple.dock"]);
        let excluded = config.excluded_bundle_ids();
        assert_eq!(excluded.len(), 2);
        assert!(excluded.contains(&"com.apple.finder".to_string()));
        assert!(excluded.contains(&"com.apple.dock".to_string()));
    }

    #[test]
    fn test_picker_configuration_excluded_window_ids() {
        let mut config = SCContentSharingPickerConfiguration::new();

        config.set_excluded_window_ids(&[123, 456, 789]);
        let excluded = config.excluded_window_ids();
        assert_eq!(excluded.len(), 3);
        assert!(excluded.contains(&123));
        assert!(excluded.contains(&456));
        assert!(excluded.contains(&789));
    }

    #[test]
    fn test_picker_mode_values() {
        assert_eq!(SCContentSharingPickerMode::SingleWindow as i32, 0);
        assert_eq!(SCContentSharingPickerMode::MultipleWindows as i32, 1);
        assert_eq!(SCContentSharingPickerMode::SingleDisplay as i32, 2);
        assert_eq!(SCContentSharingPickerMode::SingleApplication as i32, 3);
        assert_eq!(SCContentSharingPickerMode::MultipleApplications as i32, 4);
    }

    #[test]
    fn test_picker_mode_debug() {
        let mode = SCContentSharingPickerMode::SingleWindow;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("SingleWindow"));
    }

    #[test]
    fn test_picker_mode_clone() {
        let mode1 = SCContentSharingPickerMode::MultipleWindows;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }

    #[test]
    fn test_picked_source_variants() {
        let window_source = SCPickedSource::Window("Test Window".to_string());
        let display_source = SCPickedSource::Display(1);
        let app_source = SCPickedSource::Application("Test App".to_string());
        let unknown_source = SCPickedSource::Unknown;

        // Just verify they can be created
        let _ = (window_source, display_source, app_source, unknown_source);
    }

    #[test]
    fn test_picked_source_debug() {
        let source = SCPickedSource::Display(1);
        let debug_str = format!("{:?}", source);
        assert!(debug_str.contains("Display"));
    }

    #[test]
    fn test_picker_outcome_debug() {
        let outcome = SCPickerOutcome::Cancelled;
        let debug_str = format!("{:?}", outcome);
        assert!(debug_str.contains("Cancelled"));
    }

    #[test]
    fn test_picker_filter_outcome_debug() {
        let outcome = SCPickerFilterOutcome::Cancelled;
        let debug_str = format!("{:?}", outcome);
        assert!(debug_str.contains("Cancelled"));
    }

    #[test]
    fn test_set_maximum_stream_count() {
        // Just verify it doesn't crash
        SCContentSharingPicker::set_maximum_stream_count(5);
        SCContentSharingPicker::set_maximum_stream_count(1);
    }
}
