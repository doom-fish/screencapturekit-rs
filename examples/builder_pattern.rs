//! Example demonstrating the new builder pattern for SCContentFilter
//!
//! This example shows the modern, clean way to create content filters.

use screencapturekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Content Filter Builder Pattern Examples\n");

    // Get shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let windows = content.windows();

    if displays.is_empty() {
        eprintln!("No displays available");
        return Ok(());
    }

    let display = &displays[0];
    println!("✅ Found display: {}x{}", display.width(), display.height());

    // Example 1: Capture entire display
    println!("\n1. Capture entire display:");
    let _filter = SCContentFilter::build()
        .display(display)
        .exclude_windows(&[])
        .build();
    println!("   Created filter for full display capture");

    // Example 2: Capture display excluding specific windows
    if !windows.is_empty() {
        println!("\n2. Capture display excluding windows:");
        let window_refs: Vec<&SCWindow> = windows.iter().take(2).collect();
        let _filter = SCContentFilter::build()
            .display(display)
            .exclude_windows(&window_refs)
            .build();
        println!("   Created filter excluding {} windows", window_refs.len());
    }

    // Example 3: Capture specific window
    if let Some(window) = windows.first() {
        println!("\n3. Capture specific window:");
        let _filter = SCContentFilter::build()
            .window(window)
            .build();
        if let Some(title) = window.title() {
            println!("   Created filter for window: {}", title);
        } else {
            println!("   Created filter for window (no title)");
        }
    }

    // Example 4: With content rect (macOS 14.2+)
    #[cfg(feature = "macos_14_2")]
    {
        use screencapturekit::stream::configuration::{Point, Rect, Size};

        println!("\n4. Capture with content rect:");
        let _filter = SCContentFilter::build()
            .display(display)
            .exclude_windows(&[])
            .content_rect(Rect::new(Point::new(0.0, 0.0), Size::new(1920.0, 1080.0)))
            .build();
        println!("   Created filter with custom content rectangle");
    }

    // Example 5: Usage with stream
    println!("\n5. Complete stream setup:");
    let filter = SCContentFilter::build()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?;

    let _stream = SCStream::new(&filter, &config);
    println!("   Created stream with builder pattern filter");

    println!("\n✨ All examples completed successfully!");
    println!("\n💡 Benefits of the builder pattern:");
    println!("   • Cleaner, more readable code");
    println!("   • No need for deprecated warnings");
    println!("   • Type-safe construction");
    println!("   • Chainable configuration");

    Ok(())
}
