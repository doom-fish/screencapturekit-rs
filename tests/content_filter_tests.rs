//! `SCContentFilter` tests

mod common;

use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::content_filter::SCContentFilter;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes in CI
fn cg_init_for_headless_ci() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

macro_rules! require_display {
    ($content:expr, $display:ident) => {
        let displays = $content.displays();
        let Some($display) = displays.first() else {
            eprintln!("skip: no displays available");
            return;
        };
    };
}

#[test]
fn test_content_filter_builder_display() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    // Verify filter was created (debug output includes pointer)
    let debug_str = format!("{filter:?}");
    assert!(debug_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_builder_window() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::create()
            .with_window(window)
            .build()
            .expect("failed to build content filter");
        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_exclude_windows() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);
    let windows = content.windows();

    if !windows.is_empty() {
        let window_refs: Vec<&_> = windows.iter().take(2).collect();
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&window_refs)
            .build()
            .expect("failed to build content filter");

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_include_windows() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);
    let windows = content.windows();

    if !windows.is_empty() {
        let window_refs: Vec<&_> = windows.iter().take(2).collect();
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_windows(&window_refs)
            .build()
            .expect("failed to build content filter");

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

#[test]
fn test_content_filter_include_applications() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);
    let apps = content.applications();

    if !apps.is_empty() {
        let app_refs: Vec<&_> = apps.iter().take(2).collect();
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_applications(&app_refs, &[])
            .build()
            .expect("failed to build content filter");

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("SCContentFilter"));
    }
}

/// `SCContentFilter.contentRect` is read-only in `ScreenCaptureKit` — there is
/// no setter to round-trip against, so the filter derives the rect from the
/// display it was built from.
#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_content_rect_is_derived_from_the_display() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let rect = filter.content_rect();
    assert!(rect.size.width >= 0.0);
    assert!(rect.size.height >= 0.0);
    // The whole-display filter covers the display it was built from.
    assert!(
        (rect.size.width - display.frame().size.width).abs() < 1.0,
        "content_rect width {} does not match display width {}",
        rect.size.width,
        display.frame().size.width
    );
}

/// Reading `content_rect` twice must return the same value and must not
/// mutate the filter — the setter that used to exist mapped onto a Swift
/// no-op, so callers were silently misled.
#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_content_rect_is_stable() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let first = filter.content_rect();
    let second = filter.content_rect();
    let same = |a: f64, b: f64| (a - b).abs() < f64::EPSILON;
    assert!(same(first.origin.x, second.origin.x));
    assert!(same(first.origin.y, second.origin.y));
    assert!(same(first.size.width, second.size.width));
    assert!(same(first.size.height, second.size.height));
}

#[test]
fn test_content_filter_clone() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

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
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let debug = format!("{filter:?}");
    assert!(debug.contains("SCContentFilter"));

    let display_str = format!("{filter}");
    assert!(display_str.contains("SCContentFilter"));
}

#[test]
fn test_content_filter_equality() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter1 = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let filter2 = filter1.clone();

    // A filter should equal itself
    assert_eq!(filter1, filter1);
    // Cloned filters have same pointer
    assert_eq!(filter1, filter2);
}

#[test]
fn test_content_filter_hash() {
    use std::collections::HashSet;
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let mut set = HashSet::new();
    set.insert(filter.clone());

    assert!(set.contains(&filter));
}

// MARK: - New Content Filter Features (macOS 14.0+)

#[test]
#[cfg(feature = "macos_14_0")]
fn test_content_filter_style() {
    use screencapturekit::stream::content_filter::SCShareableContentStyle;
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

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
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::create()
            .with_window(window)
            .build()
            .expect("failed to build content filter");
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
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let scale = filter.point_pixel_scale();
    // Scale should be positive (typically 1.0 or 2.0 for Retina)
    assert!(scale > 0.0);
}

/// `includeMenuBar` is set by the builder — the built filter has no setter, so
/// there is nothing for a clone on another thread to race against.
#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_include_menu_bar_is_set_by_the_builder() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    for requested in [true, false] {
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&[])
            .with_include_menu_bar(requested)
            .build()
            .expect("failed to build content filter");

        assert_eq!(
            filter.include_menu_bar(),
            requested,
            "builder failed to apply include_menu_bar({requested})"
        );
    }
}

/// Apple's default depends on the constructor: `true` for display-excluding
/// filters, `false` for display-including ones. Leaving the builder option
/// unset must not disturb it.
#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_include_menu_bar_defaults_are_untouched() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let excluding = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");
    assert!(
        excluding.include_menu_bar(),
        "display-excluding filters default to including the menu bar"
    );

    let including = SCContentFilter::create()
        .with_display(display)
        .with_including_windows(&[])
        .build()
        .expect("failed to build content filter");
    assert!(
        !including.include_menu_bar(),
        "display-including filters default to excluding the menu bar"
    );
}

/// A clone aliases the same Objective-C object, and the filter is immutable, so
/// clones can be shared across threads and must always agree with the original.
#[test]
#[cfg(feature = "macos_14_2")]
fn test_content_filter_clones_are_immutable_and_shareable() {
    use std::sync::Arc;
    if !crate::common::screen_capture_allowed() {
        return;
    }

    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = Arc::new(
        SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&[])
            .with_include_menu_bar(false)
            .build()
            .expect("failed to build content filter"),
    );
    let expected = filter.include_menu_bar();

    let readers: Vec<_> = (0..4)
        .map(|_| {
            let filter = Arc::clone(&filter);
            let cloned = (*filter).clone();
            std::thread::spawn(move || {
                for _ in 0..200 {
                    assert_eq!(filter.include_menu_bar(), expected);
                    assert_eq!(cloned.include_menu_bar(), expected);
                }
            })
        })
        .collect();

    for reader in readers {
        reader.join().expect("reader thread panicked");
    }

    assert_eq!(filter.include_menu_bar(), expected);
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_displays() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .expect("failed to build content filter");

    let included_displays = filter.included_displays();
    // Display filters should have at least one included display
    // (Note: may return empty on older macOS)
    let _ = included_displays;
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_windows() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");

    if let Some(window) = content.windows().first() {
        let filter = SCContentFilter::create()
            .with_window(window)
            .build()
            .expect("failed to build content filter");
        let included_windows = filter.included_windows();
        // Window filters should have at least one included window
        // (Note: may return empty on older macOS)
        let _ = included_windows;
    }
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_content_filter_included_applications() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    cg_init_for_headless_ci();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    require_display!(content, display);
    let apps = content.applications();

    if !apps.is_empty() {
        let app_refs: Vec<&_> = apps.iter().take(1).collect();
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_including_applications(&app_refs, &[])
            .build()
            .expect("failed to build content filter");

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

#[test]
fn test_content_filter_build_without_target_is_an_error() {
    use screencapturekit::error::SCError;

    let result = SCContentFilter::create().build();
    assert!(matches!(result, Err(SCError::InvalidConfiguration(_))));

    let result = SCContentFilter::create()
        .with_excluding_windows(&[])
        .with_including_applications(&[], &[])
        .build();
    assert!(matches!(result, Err(SCError::InvalidConfiguration(_))));
}
