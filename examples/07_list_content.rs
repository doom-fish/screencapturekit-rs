//! List Available Content
//!
//! Demonstrates listing all available shareable content using the
//! batched [`SCShareableContent::snapshot`] API — populates every
//! display, window, and running application with their attributes
//! in a single FFI round-trip per category (~15× faster than the
//! per-element accessors when reading multiple attributes per item).
//!
//! For a rare lookup of a single attribute the older
//! `.windows()` / `.displays()` / `.applications()` accessors still work and
//! issue per-element FFI calls; this example uses the batched form because
//! it's reading every attribute on every item.

use screencapturekit::prelude::*;
use screencapturekit::shareable_content::ContentSnapshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Available Shareable Content\n");

    // Get the live SCShareableContent handle (one async ScreenCaptureKit
    // round-trip — this is unavoidable, the OS owns the data).
    let content = SCShareableContent::get()?;

    // Pull every display + window + app + their attrs in a single batched FFI
    // round-trip. Returns plain data structs that don't hold any Swift
    // references, so we can iterate freely without paying per-attribute FFI.
    let ContentSnapshot {
        displays,
        applications,
        windows,
    } = content
        .snapshot()
        .ok_or("Could not collect shareable content snapshot")?;

    // List displays
    println!("🖥️  Displays ({}):", displays.len());
    for d in &displays {
        println!("  - ID: {}", d.display_id);
        println!("    Size: {}x{}", d.width, d.height);
        println!("    Frame: {:?}", d.frame);

        // Pixel-size + scale info needs a content filter, which we still
        // build via the regular SCDisplay path. This is fine because we'll
        // do it at most once per display, not per attribute.
        #[cfg(feature = "macos_14_0")]
        {
            // Re-fetch the live SCDisplay handle for this id so we can build
            // a filter — the snapshot only carries plain data, not pointers.
            if let Some(live) = content
                .displays()
                .into_iter()
                .find(|live| live.display_id() == d.display_id)
            {
                let filter = SCContentFilter::create()
                    .with_display(&live)
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
    }

    // List windows (first 10) — owning_app_index is an index into
    // `applications` from the same snapshot batch, so we resolve names
    // without any extra FFI.
    println!("\n🪟 Windows (showing first 10 of {}):", windows.len());
    for w in windows.iter().take(10) {
        let app_name = w
            .owning_app_index
            .and_then(|i| applications.get(i))
            .map(|a| a.application_name.as_str())
            .unwrap_or("");

        println!("  - {} - {}", app_name, w.title.as_deref().unwrap_or(""));
        println!("    Window ID: {}", w.window_id);
        println!("    Size: {}x{}", w.frame.width, w.frame.height);
        println!("    Layer: {}", w.window_layer);
        println!("    On screen: {}", w.is_on_screen);
    }

    // List applications
    println!("\n📱 Applications ({}):", applications.len());
    for app in &applications {
        println!("  - {}", app.application_name);
        println!("    Bundle ID: {}", app.bundle_identifier);
        println!("    PID: {}", app.process_id);
    }

    // Filter windows by app — purely in-memory now, no FFI per filter check.
    println!("\n🔍 Filter Example - Safari Windows:");
    let safari_app_indices: Vec<usize> = applications
        .iter()
        .enumerate()
        .filter(|(_, a)| a.application_name.contains("Safari"))
        .map(|(i, _)| i)
        .collect();

    let safari_windows: Vec<_> = windows
        .iter()
        .filter(|w| {
            w.owning_app_index
                .is_some_and(|i| safari_app_indices.contains(&i))
        })
        .collect();

    println!("Found {} Safari windows", safari_windows.len());
    for w in safari_windows {
        println!("  - {}", w.title.as_deref().unwrap_or(""));
    }

    // Custom filtering options — uses the live API because the options
    // change what the OS itself returns.
    println!("\n⚙️  Custom Filtering:");
    let filtered = SCShareableContent::create()
        .with_on_screen_windows_only(true)
        .with_exclude_desktop_windows(true)
        .get()?;

    let filtered_snapshot = filtered.snapshot().ok_or("snapshot")?;
    println!(
        "On-screen windows only: {}",
        filtered_snapshot.windows.len()
    );

    // Show filter style information (macOS 14.0+) — these checks operate on
    // the live SCWindow/SCDisplay handles, not on snapshot rows.
    #[cfg(feature = "macos_14_0")]
    {
        use screencapturekit::stream::content_filter::SCShareableContentStyle;

        println!("\n📊 Content Filter Styles (macOS 14.0+):");
        let live_displays = content.displays();
        let live_windows = content.windows();
        if let Some(display) = live_displays.first() {
            let display_filter = SCContentFilter::create()
                .with_display(display)
                .with_excluding_windows(&[])
                .build();
            println!("  Display filter style: {:?}", display_filter.style());
        }
        if let Some(window) = live_windows.first() {
            let window_filter = SCContentFilter::create().with_window(window).build();
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
