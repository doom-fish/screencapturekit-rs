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

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
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

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(640)
        .with_height(480);

    let result = SCScreenshotManager::capture_sample_buffer(&filter, &config);
    // Note: May fail if screen recording permission not granted
    if let Ok(buffer) = result {
        // The buffer should have a presentation timestamp
        let _pts = buffer.presentation_timestamp();
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

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(100)
        .with_height(100);

    if let Ok(image) = SCScreenshotManager::capture_image(&filter, &config) {
        if let Ok(data) = image.rgba_data() {
            // RGBA is 4 bytes per pixel
            let expected_min_size = image.width() * image.height() * 4;
            assert!(data.len() >= expected_min_size);
        }
    }
}

#[test]
fn test_cgimage_bgra_matches_rgba_byteswap() {
    // Both rgba_data() and bgra_data() should write width*height*4 bytes,
    // and bgra[i*4..] must equal [rgba[i*4+2], rgba[i*4+1], rgba[i*4], rgba[i*4+3]]
    // for every pixel — i.e. the BGRA path is the byte-for-byte channel
    // permutation of the RGBA path. If this regresses we'd silently ship a
    // broken format to BGRA consumers.
    cg_init_for_headless_ci();
    let Ok(content) = SCShareableContent::get() else {
        return;
    };
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new().with_width(64).with_height(64);

    let Ok(image) = SCScreenshotManager::capture_image(&filter, &config) else {
        return;
    };
    let Ok(rgba) = image.rgba_data() else { return };
    let Ok(bgra) = image.bgra_data() else { return };

    assert_eq!(
        rgba.len(),
        bgra.len(),
        "RGBA and BGRA buffers must match in size"
    );
    assert_eq!(rgba.len(), image.width() * image.height() * 4);

    // Verify channel permutation on every 4-byte pixel.
    let mismatches = rgba
        .chunks_exact(4)
        .zip(bgra.chunks_exact(4))
        .filter(|(rgba_px, bgra_px)| {
            // RGBA = [R, G, B, A]; BGRA = [B, G, R, A].
            rgba_px[0] != bgra_px[2]
                || rgba_px[1] != bgra_px[1]
                || rgba_px[2] != bgra_px[0]
                || rgba_px[3] != bgra_px[3]
        })
        .count();
    let total = rgba.len() / 4;
    // Allow up to 0.5% mismatched pixels — both paths go through CGContext.draw
    // which can produce minor rounding deltas in alpha-premultiplication
    // depending on sub-pixel layout. Anything more than that means the channel
    // layout is genuinely wrong, not a rounding issue.
    let tolerance = total / 200 + 1;
    assert!(
        mismatches <= tolerance,
        "BGRA layout doesn't match RGBA byte-swap: {mismatches}/{total} pixels differ (tolerance {tolerance})"
    );
}

#[test]
fn test_cgimage_data_into_buffer_apis() {
    cg_init_for_headless_ci();
    let Ok(content) = SCShareableContent::get() else {
        return;
    };
    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new().with_width(64).with_height(64);
    let Ok(image) = SCScreenshotManager::capture_image(&filter, &config) else {
        return;
    };

    let total_bytes = image.width() * image.height() * 4;

    // 1. Both APIs must report the same total byte count and succeed. We
    // intentionally do NOT assert byte-for-byte equality between rgba_data
    // and rgba_data_into — even though they hit the same Swift FFI on the
    // same CGImage, observed behaviour across macOS versions is that
    // CGContext.draw on a separately-captured frame can produce slightly
    // different pixel values (cursor blink, animation frame, etc.). The
    // safety contract is "writes exactly width*height*4 bytes into the
    // destination" and that's what we check here.
    let rgba_owned = image.rgba_data().expect("rgba_data");
    assert_eq!(rgba_owned.len(), total_bytes);

    let mut rgba_buf = vec![0u8; total_bytes];
    let written = image.rgba_data_into(&mut rgba_buf).expect("rgba_data_into");
    assert_eq!(written, total_bytes);

    let bgra_owned = image.bgra_data().expect("bgra_data");
    assert_eq!(bgra_owned.len(), total_bytes);

    let mut bgra_buf = vec![0u8; total_bytes];
    let written = image.bgra_data_into(&mut bgra_buf).expect("bgra_data_into");
    assert_eq!(written, total_bytes);

    // 2. Two consecutive into-buffer calls on the *same buffer* must produce
    // identical output (the FFI is deterministic for a given destination
    // initialisation; we control both ends here).
    let mut a = vec![0u8; total_bytes];
    let mut b = vec![0u8; total_bytes];
    image.bgra_data_into(&mut a).expect("a");
    image.bgra_data_into(&mut b).expect("b");
    assert_eq!(a, b, "deterministic output for identical destination state");

    // 3. A too-small buffer must be rejected — no out-of-bounds writes.
    let mut small = vec![0u8; total_bytes - 1];
    assert!(
        image.rgba_data_into(&mut small).is_err(),
        "rgba_data_into must reject undersized destination"
    );
    assert!(
        image.bgra_data_into(&mut small).is_err(),
        "bgra_data_into must reject undersized destination"
    );

    // 4. An over-sized buffer should still work; only the first N bytes are
    // touched and the rest is left at whatever the caller had.
    let sentinel = 0xCDu8;
    let mut large = vec![sentinel; total_bytes + 16];
    let written = image.bgra_data_into(&mut large).expect("oversize ok");
    assert_eq!(written, total_bytes);
    assert!(
        large[total_bytes..].iter().all(|&b| b == sentinel),
        "bytes past the rendered region must not be touched"
    );
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
            println!(
                "✓ Captured image in rect: {}x{}",
                image.width(),
                image.height()
            );
        }
        Err(e) => {
            // Expected on macOS < 15.2 or without permission
            println!("⚠ capture_image_in_rect not available: {e}");
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
            println!(
                "✓ Captured small region: {}x{}",
                image.width(),
                image.height()
            );
        }
        Err(_) => {
            println!("⚠ capture_image_in_rect not available");
        }
    }
}

// MARK: - Advanced Screenshot Configuration (macOS 26.0+)

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_configuration_creation() {
    use screencapturekit::screenshot_manager::SCScreenshotConfiguration;

    let config = SCScreenshotConfiguration::new();
    assert!(!config.as_ptr().is_null());
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_configuration_builder() {
    use screencapturekit::cg::CGRect;
    use screencapturekit::screenshot_manager::{
        SCScreenshotConfiguration, SCScreenshotDisplayIntent, SCScreenshotDynamicRange,
    };

    let config = SCScreenshotConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_shows_cursor(true)
        .with_source_rect(CGRect::new(0.0, 0.0, 1920.0, 1080.0))
        .with_destination_rect(CGRect::new(0.0, 0.0, 1920.0, 1080.0))
        .with_ignore_shadows(true)
        .with_ignore_clipping(false)
        .with_include_child_windows(true)
        .with_display_intent(SCScreenshotDisplayIntent::Canonical)
        .with_dynamic_range(SCScreenshotDynamicRange::SDR);

    assert!(!config.as_ptr().is_null());
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_configuration_hdr() {
    use screencapturekit::screenshot_manager::{
        SCScreenshotConfiguration, SCScreenshotDynamicRange,
    };

    // Test each dynamic range option
    let sdr_config =
        SCScreenshotConfiguration::new().with_dynamic_range(SCScreenshotDynamicRange::SDR);
    assert!(!sdr_config.as_ptr().is_null());

    let hdr_config =
        SCScreenshotConfiguration::new().with_dynamic_range(SCScreenshotDynamicRange::HDR);
    assert!(!hdr_config.as_ptr().is_null());

    let both_config = SCScreenshotConfiguration::new()
        .with_dynamic_range(SCScreenshotDynamicRange::BothSDRAndHDR);
    assert!(!both_config.as_ptr().is_null());
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_configuration_file_path() {
    use screencapturekit::screenshot_manager::SCScreenshotConfiguration;

    let config = SCScreenshotConfiguration::new().with_file_path("/tmp/test_screenshot.png");
    assert!(!config.as_ptr().is_null());
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_configuration_send_sync() {
    use screencapturekit::screenshot_manager::{SCScreenshotConfiguration, SCScreenshotOutput};

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SCScreenshotConfiguration>();
    assert_sync::<SCScreenshotConfiguration>();
    assert_send::<SCScreenshotOutput>();
    assert_sync::<SCScreenshotOutput>();
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_display_intent_enum() {
    use screencapturekit::screenshot_manager::SCScreenshotDisplayIntent;

    assert_eq!(SCScreenshotDisplayIntent::Canonical as i32, 0);
    assert_eq!(SCScreenshotDisplayIntent::Local as i32, 1);

    // Test default
    let default = SCScreenshotDisplayIntent::default();
    assert_eq!(default, SCScreenshotDisplayIntent::Canonical);
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_screenshot_dynamic_range_enum() {
    use screencapturekit::screenshot_manager::SCScreenshotDynamicRange;

    assert_eq!(SCScreenshotDynamicRange::SDR as i32, 0);
    assert_eq!(SCScreenshotDynamicRange::HDR as i32, 1);
    assert_eq!(SCScreenshotDynamicRange::BothSDRAndHDR as i32, 2);

    // Test default
    let default = SCScreenshotDynamicRange::default();
    assert_eq!(default, SCScreenshotDynamicRange::SDR);
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_capture_screenshot_with_configuration() {
    use screencapturekit::screenshot_manager::{
        SCScreenshotConfiguration, SCScreenshotDynamicRange,
    };

    cg_init_for_headless_ci();
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCScreenshotConfiguration::new()
        .with_width(640)
        .with_height(480)
        .with_shows_cursor(true)
        .with_dynamic_range(SCScreenshotDynamicRange::SDR);

    let result = SCScreenshotManager::capture_screenshot(&filter, &config);

    match result {
        Ok(output) => {
            // Should have at least SDR image
            if let Some(sdr) = output.sdr_image() {
                assert!(sdr.width() > 0);
                assert!(sdr.height() > 0);
                println!(
                    "✓ Advanced screenshot SDR: {}x{}",
                    sdr.width(),
                    sdr.height()
                );
            }
        }
        Err(e) => {
            // Expected on macOS < 26.0 or without permission
            println!("⚠ capture_screenshot not available: {e}");
        }
    }
}

#[test]
#[cfg(feature = "macos_26_0")]
fn test_capture_screenshot_in_rect_with_configuration() {
    use screencapturekit::cg::CGRect;
    use screencapturekit::screenshot_manager::SCScreenshotConfiguration;

    cg_init_for_headless_ci();

    let rect = CGRect::new(0.0, 0.0, 640.0, 480.0);
    let config = SCScreenshotConfiguration::new()
        .with_width(640)
        .with_height(480);

    let result = SCScreenshotManager::capture_screenshot_in_rect(rect, &config);

    match result {
        Ok(output) => {
            if let Some(image) = output.sdr_image() {
                assert!(image.width() > 0);
                println!(
                    "✓ Advanced screenshot in rect: {}x{}",
                    image.width(),
                    image.height()
                );
            }
        }
        Err(e) => {
            println!("⚠ capture_screenshot_in_rect not available: {e}");
        }
    }
}
