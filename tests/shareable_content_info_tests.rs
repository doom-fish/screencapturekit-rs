//! SCShareableContentInfo tests (macOS 14.0+)

#![cfg(feature = "macos_14_0")]

use screencapturekit::shareable_content::{SCShareableContent, SCShareableContentInfo};
use screencapturekit::stream::content_filter::SCContentFilter;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes in CI
fn cg_init_for_headless_ci() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

#[test]
fn test_shareable_content_info_for_display_filter() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
        // Test style
        let style = info.style();
        println!("Content style: {:?}", style);

        // Test point_pixel_scale
        let scale = info.point_pixel_scale();
        assert!(scale > 0.0, "Scale should be positive");
        println!("Point pixel scale: {}", scale);

        // Test content_rect
        let rect = info.content_rect();
        println!("Content rect: {:?}", rect);

        // Test pixel_size
        let (width, height) = info.pixel_size();
        println!("Pixel size: {}x{}", width, height);
    } else {
        println!("⚠ SCShareableContentInfo not available on this macOS version");
    }
}

#[test]
fn test_shareable_content_info_for_window_filter() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::builder().window(window).build();

        if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
            let style = info.style();
            println!("Window filter style: {:?}", style);

            let scale = info.point_pixel_scale();
            assert!(scale > 0.0);

            let (width, height) = info.pixel_size();
            println!("Window pixel size: {}x{}", width, height);
        }
    }
}

#[test]
fn test_shareable_content_info_clone() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
        let cloned = info.clone();

        // Both should have the same values
        assert_eq!(info.point_pixel_scale(), cloned.point_pixel_scale());
        assert_eq!(info.pixel_size(), cloned.pixel_size());
    }
}

#[test]
fn test_shareable_content_info_debug() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("SCShareableContentInfo"));
        assert!(debug_str.contains("style"));
        assert!(debug_str.contains("point_pixel_scale"));
    }
}

#[test]
fn test_shareable_content_info_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<SCShareableContentInfo>();
    assert_sync::<SCShareableContentInfo>();
}

#[test]
fn test_shareable_content_info_pixel_size_calculation() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    if let Some(info) = SCShareableContentInfo::for_filter(&filter) {
        let rect = info.content_rect();
        let scale = info.point_pixel_scale();
        let (width, height) = info.pixel_size();

        // Verify the pixel size calculation
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let expected_width = (rect.width * f64::from(scale)) as u32;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let expected_height = (rect.height * f64::from(scale)) as u32;

        assert_eq!(width, expected_width, "Width calculation mismatch");
        assert_eq!(height, expected_height, "Height calculation mismatch");
    }
}
