//! Screenshot manager tests (macOS 14.0+)

#![cfg(feature = "macos_14_0")]

use screencapturekit::screenshot_manager::{CGImage, SCScreenshotManager};
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes in CI
fn cg_init_for_headless_ci() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

#[test]
fn test_screenshot_manager_type() {
    // Just verify the type exists and can be referenced
    let _ = SCScreenshotManager;
}

#[test]
fn test_capture_image() {
    cg_init_for_headless_ci();
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
    cg_init_for_headless_ci();
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
    cg_init_for_headless_ci();
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

// MARK: - New Screenshot Features (macOS 15.2+)

#[test]
#[cfg(feature = "macos_15_2")]
fn test_capture_image_in_rect() {
    use screencapturekit::cg::CGRect;
    cg_init_for_headless_ci();

    // Capture a specific region of the screen
    let rect = CGRect::new(0.0, 0.0, 640.0, 480.0);
    let result = SCScreenshotManager::capture_image_in_rect(rect);

    match result {
        Ok(image) => {
            assert!(image.width() > 0);
            assert!(image.height() > 0);
            println!("✓ Captured image in rect: {}x{}", image.width(), image.height());
        }
        Err(e) => {
            // Expected on macOS < 15.2 or without permission
            println!("⚠ capture_image_in_rect not available: {}", e);
        }
    }
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_capture_image_in_rect_small_region() {
    use screencapturekit::cg::CGRect;
    cg_init_for_headless_ci();

    // Capture a small 100x100 region
    let rect = CGRect::new(100.0, 100.0, 100.0, 100.0);
    let result = SCScreenshotManager::capture_image_in_rect(rect);

    match result {
        Ok(image) => {
            println!("✓ Captured small region: {}x{}", image.width(), image.height());
        }
        Err(_) => {
            println!("⚠ capture_image_in_rect not available");
        }
    }
}
