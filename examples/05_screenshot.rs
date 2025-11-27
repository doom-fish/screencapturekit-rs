//! Screenshot Capture (macOS 14.0+)
//!
//! Demonstrates taking a single screenshot.
//! This example shows:
//! - Using SCScreenshotManager (macOS 14.0+)
//! - Capturing a screenshot
//! - Saving as PNG

#[cfg(feature = "macos_14_0")]
use screencapturekit::screenshot_manager::SCScreenshotManager;
#[cfg(feature = "macos_14_0")]
use screencapturekit::prelude::*;

#[cfg(not(feature = "macos_14_0"))]
fn main() {
    println!("⚠️  This example requires macOS 14.0+");
    println!("    Run with: cargo run --example 05_screenshot --features macos_14_0");
}

#[cfg(feature = "macos_14_0")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📸 Screenshot Capture\n");

    // 1. Get display
    let content = SCShareableContent::get()?;
    let display = content.displays().into_iter().next()
        .ok_or("No displays found")?;
    
    println!("Display: {}x{}", display.width(), display.height());

    // 2. Create filter
    let filter = SCContentFilter::build()
        .display(&display)
        .exclude_windows(&[])
        .build();

    // 3. Configure screenshot
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?;

    // 4. Capture screenshot
    println!("Capturing...");
    let image = SCScreenshotManager::capture_image(&filter, &config)?;
    
    println!("Captured: {}x{}", image.width(), image.height());

    // 5. Save as PNG
    let filename = "screenshot.png";
    image.save_to_png(filename)?;
    
    println!("✅ Saved to {}", filename);
    Ok(())
}
