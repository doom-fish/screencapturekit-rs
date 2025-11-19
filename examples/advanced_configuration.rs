//! Example demonstrating the newly added advanced configuration options
//! 
//! This example shows how to use:
//! - Advanced SCStreamConfiguration properties (macOS 14.0+)
//! - SCWindow.is_active() (macOS 14.0+)
//! - SCContentFilter.contentRect (macOS 14.2+)
//!
//! Requires: --features macos_14_2

use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::{
    SCStreamConfiguration, 
    Point,
    Rect,
    Size,
};
#[cfg(all(feature = "macos_13_0", feature = "macos_14_2"))]
use screencapturekit::stream::configuration::SCPresenterOverlayAlertSetting;
use screencapturekit::stream::content_filter::SCContentFilter;

#[cfg(all(feature = "macos_13_0", feature = "macos_14_2"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎥 ScreenCaptureKit Advanced Configuration Example\n");

    // Get shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let windows = content.windows();

    println!("Found {} displays and {} windows", displays.len(), windows.len());

    // Example 1: Using advanced SCStreamConfiguration options
    println!("\n📋 Example 1: Advanced Stream Configuration");
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?
        .set_captures_audio(true)?
        // ✨ NEW: Advanced properties (macOS 14.0+)
        .set_should_be_opaque(true)?
        .set_ignore_fraction_of_screen(0.1)?
        .set_ignores_shadows_single_window(true)?
        .set_includes_child_windows(true)?
        .set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Once)?;

    println!("✅ Created configuration with advanced options:");
    println!("   - Width: {}", config.get_width());
    println!("   - Height: {}", config.get_height());
    println!("   - Should be opaque: {}", config.get_should_be_opaque());
    println!("   - Ignore fraction: {:.2}", config.get_ignore_fraction_of_screen());
    println!("   - Ignores shadows: {}", config.get_ignores_shadows_single_window());
    println!("   - Includes child windows: {}", config.get_includes_child_windows());
    println!("   - Privacy alert: {:?}", config.get_presenter_overlay_privacy_alert_setting());

    // Example 2: Using SCWindow.is_active()
    println!("\n🪟 Example 2: Finding Active Window (macOS 14.0+)");
    if let Some(active_window) = windows.iter().find(|w| w.is_active()) {
        println!("✅ Found active window:");
        println!("   - Title: {:?}", active_window.title());
        println!("   - Window ID: {}", active_window.window_id());
        println!("   - On screen: {}", active_window.is_on_screen());
        println!("   - Active: {}", active_window.is_active());
    } else {
        println!("ℹ️  No active window found (may require macOS 14.0+)");
    }

    // Example 3: Using SCContentFilter.contentRect
    println!("\n📐 Example 3: Content Rectangle Filtering (macOS 14.2+)");
    if let Some(display) = displays.first() {
        #[allow(deprecated)]
        let _filter = SCContentFilter::new()
            .with_display_excluding_windows(display, &[])
            .set_content_rect(Rect::new(
                Point::new(0.0, 0.0),
                Size::new(800.0, 600.0),
            ));

        let rect = _filter.get_content_rect();
        println!("✅ Created filter with content rect:");
        println!("   - Origin: ({:.1}, {:.1})", rect.origin.x, rect.origin.y);
        println!("   - Size: {:.1}x{:.1}", rect.size.width, rect.size.height);
    }

    // Example 4: Complete stream setup with all new features
    println!("\n🚀 Example 4: Complete Setup");
    if let Some(display) = displays.first() {
        // Create filter with content rect
        #[allow(deprecated)]
        let _filter = SCContentFilter::new()
            .with_display_excluding_windows(display, &[])
            .set_content_rect(Rect::new(
                Point::new(100.0, 100.0),
                Size::new(1920.0, 1080.0),
            ));

        // Create configuration with all advanced options
        let _config = SCStreamConfiguration::build()
            .set_width(1920)?
            .set_height(1080)?
            .set_captures_audio(true)?
            .set_should_be_opaque(true)?
            .set_ignore_fraction_of_screen(0.05)?
            .set_ignores_shadows_single_window(true)?
            .set_includes_child_windows(false)?
            .set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Never)?;

        println!("✅ Created complete stream configuration:");
        println!("   - Filter with content rect: 100x100+1920x1080");
        println!("   - All advanced configuration options enabled");
        println!("   - Ready for stream creation");

        // Note: Actual stream creation and capture would go here
        // let mut stream = SCStream::new(&_filter, &_config);
        // stream.start_capture()?;
    }

    println!("\n✨ All advanced features demonstrated!");
    println!("\nNote: Some features require macOS 14.0+ or 14.2+");
    println!("On older versions, they gracefully degrade (return defaults or no-op).");

    Ok(())
}

#[cfg(not(all(feature = "macos_13_0", feature = "macos_14_2")))]
fn main() {
    eprintln!("This example requires the macos_14_2 feature flag.");
    eprintln!("Run with: cargo run --example advanced_configuration --features macos_14_2");
    std::process::exit(1);
}
