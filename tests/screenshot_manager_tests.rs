//! Screenshot manager tests (macOS 14.0+)

#![cfg(feature = "macos_14_0")]

use screencapturekit::screenshot_manager::{CGImage, SCScreenshotManager};
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;

#[test]
fn test_screenshot_manager_type() {
    // Just verify the type exists and can be referenced
    let _ = SCScreenshotManager;
}

#[test]
fn test_capture_image() {
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(640)
        .with_height(480);

    let result = SCScreenshotManager::capture_image(&filter, &config);

    if let Ok(image) = result {
        assert!(image.width() > 0);
        assert!(image.height() > 0);
    }
    // Note: May fail if screen recording permission not granted
}

#[test]
fn test_capture_sample_buffer() {
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(640)
        .with_height(480);

    let result = SCScreenshotManager::capture_sample_buffer(&filter, &config);
    // Note: May fail if screen recording permission not granted
    if let Ok(buffer) = result {
        // The buffer should have a presentation timestamp
        let _pts = buffer.get_presentation_timestamp();
    }
}

#[test]
fn test_cgimage_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CGImage>();
    assert_sync::<CGImage>();
}

#[test]
fn test_cgimage_rgba_data() {
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(100)
        .with_height(100);

    if let Ok(image) = SCScreenshotManager::capture_image(&filter, &config) {
        if let Ok(data) = image.get_rgba_data() {
            // RGBA is 4 bytes per pixel
            let expected_min_size = image.width() * image.height() * 4;
            assert!(data.len() >= expected_min_size);
        }
    }
}
