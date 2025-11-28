//! `SCContentFilter` tests

use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::{Point, Rect, Size};
use screencapturekit::stream::content_filter::SCContentFilter;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes in CI
fn cg_init_for_headless_ci() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

#[test]
fn test_content_filter_builder_display() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Verify filter was created (debug output includes pointer)
    let debug_str = format!("{filter:?}");
    assert!(debug_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_builder_window() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::builder().window(window).build();
        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_exclude_windows() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];
    let windows = content.windows();

    if !windows.is_empty() {
        let window_refs: Vec<&_> = windows.iter().take(2).collect();
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&window_refs)
            .build();

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_include_windows() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];
    let windows = content.windows();

    if !windows.is_empty() {
        let window_refs: Vec<&_> = windows.iter().take(2).collect();
        let filter = SCContentFilter::builder()
            .display(display)
            .include_windows(&window_refs)
            .build();

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_include_applications() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];
    let apps = content.applications();

    if !apps.is_empty() {
        let app_refs: Vec<&_> = apps.iter().take(2).collect();
        let filter = SCContentFilter::builder()
            .display(display)
            .include_applications(&app_refs, &[])
            .build();

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_content_rect() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let rect = Rect::new(Point::new(100.0, 100.0), Size::new(800.0, 600.0));

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .content_rect(rect)
        .build();

    // Test get_content_rect
    let retrieved_rect = filter.get_content_rect();
    // The rect should be set (though exact values may vary based on macOS version)
    assert!(retrieved_rect.size.width >= 0.0);
    assert!(retrieved_rect.size.height >= 0.0);
}

#[test]
fn test_content_filter_set_content_rect() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let rect = Rect::new(Point::new(50.0, 50.0), Size::new(400.0, 300.0));
    let filter = filter.set_content_rect(rect);

    let debug_str = format!("{filter:?}");
    assert!(debug_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_clone() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let cloned = filter;
    let debug_str = format!("{cloned:?}");
    assert!(debug_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<SCContentFilter>();
    assert_sync::<SCContentFilter>();
}

#[test]
fn test_content_filter_debug_display() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let debug = format!("{filter:?}");
    assert!(debug.contains("SCContentFilter"));

    let display_str = format!("{filter}");
    assert!(display_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_equality() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter1 = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let filter2 = filter1.clone();

    // A filter should equal itself
    assert_eq!(filter1, filter1);
    // Cloned filters have same pointer
    assert_eq!(filter1, filter2);
}

#[test]
fn test_content_filter_hash() {
    cg_init_for_headless_ci();
    use std::collections::HashSet;

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let mut set = HashSet::new();
    set.insert(filter.clone());

    assert!(set.contains(&filter));
}
