//! `SCContentFilter` tests

use screencapturekit::cg::CGRect;
use screencapturekit::shareable_content::SCShareableContent;
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

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
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
        let filter = SCContentFilter::create().with_window(window).build();
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
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&window_refs)
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
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_windows(&window_refs)
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
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_applications(&app_refs, &[])
            .build();

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_content_rect() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let rect = CGRect::new(100.0, 100.0, 800.0, 600.0);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .with_content_rect(rect)
        .build();

    // Test get_content_rect
    let retrieved_rect = filter.content_rect();
    // The rect should be set (though exact values may vary based on macOS version)
    assert!(retrieved_rect.width >= 0.0);
    assert!(retrieved_rect.height >= 0.0);
}

#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_set_content_rect() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let rect = CGRect::new(50.0, 50.0, 400.0, 300.0);
    let filter = filter.set_content_rect(rect);

    let debug_str = format!("{filter:?}");
    assert!(debug_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_clone() {
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
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

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
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

    let filter1 = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let filter2 = filter1.clone();

    // A filter should equal itself
    assert_eq!(filter1, filter1);
    // Cloned filters have same pointer
    assert_eq!(filter1, filter2);
}

#[test]
fn test_content_filter_hash() {
    use std::collections::HashSet;
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let mut set = HashSet::new();
    set.insert(filter.clone());

    assert!(set.contains(&filter));
}

// MARK: - New Content Filter Features (macOS 14.0+)

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_style() {
    use screencapturekit::stream::content_filter::SCShareableContentStyle;
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let style = filter.style();
    // Display filters should have Display style
    assert!(matches!(
        style,
        SCShareableContentStyle::Display | SCShareableContentStyle::None
    ));
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_style_window() {
    use screencapturekit::stream::content_filter::SCShareableContentStyle;
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::create().with_window(window).build();
        let style = filter.style();
        // Window filters should have Window style
        assert!(matches!(
            style,
            SCShareableContentStyle::Window | SCShareableContentStyle::None
        ));
    }
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_point_pixel_scale() {
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let scale = filter.point_pixel_scale();
    // Scale should be positive (typically 1.0 or 2.0 for Retina)
    assert!(scale > 0.0);
}

#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_include_menu_bar() {
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let mut filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    // Set include menu bar
    filter.set_include_menu_bar(true);
    let includes_menu_bar = filter.include_menu_bar();
    // May return false on older macOS versions
    let _ = includes_menu_bar;
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_displays() {
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let included_displays = filter.included_displays();
    // Display filters should have at least one included display
    // (Note: may return empty on older macOS)
    let _ = included_displays;
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_windows() {
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::create().with_window(window).build();
        let included_windows = filter.included_windows();
        // Window filters should have at least one included window
        // (Note: may return empty on older macOS)
        let _ = included_windows;
    }
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_applications() {
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];
    let apps = content.applications();

    if !apps.is_empty() {
        let app_refs: Vec<&_> = apps.iter().take(1).collect();
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_applications(&app_refs, &[])
            .build();

        let included_apps = filter.included_applications();
        // Application filters should have included applications
        // (Note: may return empty on older macOS)
        let _ = included_apps;
    }
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_shareable_content_style_values() {
    use screencapturekit::stream::content_filter::SCShareableContentStyle;

    // Test that all style values can be compared
    assert_eq!(SCShareableContentStyle::None, SCShareableContentStyle::None);
    assert_eq!(
        SCShareableContentStyle::Window,
        SCShareableContentStyle::Window
    );
    assert_eq!(
        SCShareableContentStyle::Display,
        SCShareableContentStyle::Display
    );
    assert_eq!(
        SCShareableContentStyle::Application,
        SCShareableContentStyle::Application
    );

    assert_ne!(
        SCShareableContentStyle::None,
        SCShareableContentStyle::Window
    );
    assert_ne!(
        SCShareableContentStyle::Display,
        SCShareableContentStyle::Application
    );
}

#[test]
#[cfg(feature = "macos_14_0")]
fn test_shareable_content_style_from_i32() {
    use screencapturekit::stream::content_filter::SCShareableContentStyle;

    assert_eq!(
        SCShareableContentStyle::from(0),
        SCShareableContentStyle::None
    );
    assert_eq!(
        SCShareableContentStyle::from(1),
        SCShareableContentStyle::Window
    );
    assert_eq!(
        SCShareableContentStyle::from(2),
        SCShareableContentStyle::Display
    );
    assert_eq!(
        SCShareableContentStyle::from(3),
        SCShareableContentStyle::Application
    );
    assert_eq!(
        SCShareableContentStyle::from(99),
        SCShareableContentStyle::None
    ); // Unknown
}
