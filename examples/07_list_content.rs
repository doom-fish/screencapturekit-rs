//! List Available Content
//!
//! Demonstrates listing all available shareable content.
//! This example shows:
//! - Getting displays
//! - Getting windows
//! - Getting applications
//! - Filtering content
//! - Getting content info from filters (macOS 14.0+)

use screencapturekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Available Shareable Content\n");

    // Get all shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let windows = content.windows();
    let applications = content.applications();

    // List displays
    println!("🖥️  Displays ({}):", displays.len());
    for display in &displays {
        println!("  - ID: {}", display.display_id());
        println!("    Size: {}x{}", display.width(), display.height());
        println!("    Frame: {:?}", display.frame());

        // Show content info for display (macOS 14.0+)
        #[cfg(feature = "macos_14_0")]
        {
            let filter = SCContentFilter::with()
                .with_display(display)
                .with_excluding_windows(&[])
                .build();

            if let Some(info) =
                screencapturekit::shareable_content::SCShareableContentInfo::for_filter(&filter)
            {
                let (pw, ph) = info.pixel_size();
                println!(
                    "    Pixel size: {}x{} ({:.1}x scale)",
                    pw,
                    ph,
                    info.point_pixel_scale()
                );
            }
        }
    }

    // List windows (first 10)
    println!("\n🪟 Windows (showing first 10 of {}):", windows.len());
    for window in windows.iter().take(10) {
        let app_name = window
            .owning_application()
            .map(|app| app.application_name())
            .unwrap_or_default();

        println!("  - {} - {}", app_name, window.title().unwrap_or_default());
        println!("    Window ID: {}", window.window_id());
        println!(
            "    Size: {}x{}",
            window.frame().width,
            window.frame().height
        );
        println!("    Layer: {}", window.window_layer());
        println!("    On screen: {}", window.is_on_screen());
    }

    // List applications
    println!("\n📱 Applications ({}):", applications.len());
    for app in applications {
        println!("  - {}", app.application_name());
        println!("    Bundle ID: {}", app.bundle_identifier());
        println!("    PID: {}", app.process_id());
    }

    // Filter windows by app
    println!("\n🔍 Filter Example - Safari Windows:");
    let safari_windows: Vec<_> = windows
        .iter()
        .filter(|w| {
            w.owning_application()
                .is_some_and(|app| app.application_name().contains("Safari"))
        })
        .collect();

    println!("Found {} Safari windows", safari_windows.len());
    for window in safari_windows {
        println!("  - {}", window.title().unwrap_or_default());
    }

    // Custom filtering options
    println!("\n⚙️  Custom Filtering:");
    let filtered = SCShareableContent::create()
        .with_on_screen_windows_only(true)
        .with_exclude_desktop_windows(true)
        .get()?;

    println!("On-screen windows only: {}", filtered.windows().len());

    // Show filter style information (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    {
        use screencapturekit::stream::content_filter::SCShareableContentStyle;

        println!("\n📊 Content Filter Styles (macOS 14.0+):");
        if let Some(display) = displays.first() {
            let display_filter = SCContentFilter::with()
                .with_display(display)
                .with_excluding_windows(&[])
                .build();
            println!("  Display filter style: {:?}", display_filter.style());
        }
        if let Some(window) = windows.first() {
            let window_filter = SCContentFilter::with().with_window(window).build();
            println!("  Window filter style: {:?}", window_filter.style());
        }
        println!("\n  Style values:");
        println!("    None = {:?}", SCShareableContentStyle::None);
        println!("    Window = {:?}", SCShareableContentStyle::Window);
        println!("    Display = {:?}", SCShareableContentStyle::Display);
        println!(
            "    Application = {:?}",
            SCShareableContentStyle::Application
        );
    }

    Ok(())
}
