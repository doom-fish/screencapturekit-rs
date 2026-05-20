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
use screencapturekit::shareable_content::{
    ApplicationSnapshot, ContentSnapshot, DisplaySnapshot, WindowSnapshot,
};

type ExampleResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> ExampleResult {
    println!("📋 Available Shareable Content\n");

    let content = SCShareableContent::get()?;
    let ContentSnapshot {
        displays,
        applications,
        windows,
    } = content
        .snapshot()
        .ok_or("Could not collect shareable content snapshot")?;

    print_displays(&content, &displays);
    print_windows(&applications, &windows);
    print_applications(&applications);
    print_safari_windows(&applications, &windows);
    print_custom_filtering()?;
    #[cfg(feature = "macos_14_0")]
    print_filter_styles(&content);

    Ok(())
}

fn print_displays(content: &SCShareableContent, displays: &[DisplaySnapshot]) {
    println!("🖥️  Displays ({}):", displays.len());
    for display in displays {
        println!("  - ID: {}", display.display_id);
        println!("    Size: {}x{}", display.width, display.height);
        println!("    Frame: {:?}", display.frame);
        print_display_scale_info(content, display);
    }
}

#[cfg(feature = "macos_14_0")]
fn print_display_scale_info(content: &SCShareableContent, display: &DisplaySnapshot) {
    if let Some(live) = content
        .displays()
        .into_iter()
        .find(|live| live.display_id() == display.display_id)
    {
        let filter = SCContentFilter::create()
            .with_display(&live)
            .with_excluding_windows(&[])
            .build();

        if let Some(info) =
            screencapturekit::shareable_content::SCShareableContentInfo::for_filter(&filter)
        {
            let (width, height) = info.pixel_size();
            println!(
                "    Pixel size: {}x{} ({:.1}x scale)",
                width,
                height,
                info.point_pixel_scale()
            );
        }
    }
}

#[cfg(not(feature = "macos_14_0"))]
const fn print_display_scale_info(_content: &SCShareableContent, _display: &DisplaySnapshot) {}

fn print_windows(applications: &[ApplicationSnapshot], windows: &[WindowSnapshot]) {
    println!("\n🪟 Windows (showing first 10 of {}):", windows.len());
    for window in windows.iter().take(10) {
        let app_name = window
            .owning_app_index
            .and_then(|i| applications.get(i))
            .map_or("", |app| app.application_name.as_str());

        println!(
            "  - {} - {}",
            app_name,
            window.title.as_deref().unwrap_or("")
        );
        println!("    Window ID: {}", window.window_id);
        println!(
            "    Size: {}x{}",
            window.frame.size.width, window.frame.size.height
        );
        println!("    Layer: {}", window.window_layer);
        println!("    On screen: {}", window.is_on_screen);
    }
}

fn print_applications(applications: &[ApplicationSnapshot]) {
    println!("\n📱 Applications ({}):", applications.len());
    for app in applications {
        println!("  - {}", app.application_name);
        println!("    Bundle ID: {}", app.bundle_identifier);
        println!("    PID: {}", app.process_id);
    }
}

fn print_safari_windows(applications: &[ApplicationSnapshot], windows: &[WindowSnapshot]) {
    println!("\n🔍 Filter Example - Safari Windows:");
    let safari_app_indices: Vec<usize> = applications
        .iter()
        .enumerate()
        .filter(|(_, app)| app.application_name.contains("Safari"))
        .map(|(i, _)| i)
        .collect();

    let safari_windows: Vec<_> = windows
        .iter()
        .filter(|window| {
            window
                .owning_app_index
                .is_some_and(|i| safari_app_indices.contains(&i))
        })
        .collect();

    println!("Found {} Safari windows", safari_windows.len());
    for window in safari_windows {
        println!("  - {}", window.title.as_deref().unwrap_or(""));
    }
}

fn print_custom_filtering() -> ExampleResult {
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
    Ok(())
}

#[cfg(feature = "macos_14_0")]
fn print_filter_styles(content: &SCShareableContent) {
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
