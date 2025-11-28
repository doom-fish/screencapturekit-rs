//! Screenshot Capture (macOS 14.0+)
//!
//! Demonstrates taking a single screenshot.
//! This example shows:
//! - Using `SCScreenshotManager` (macOS 14.0+)
//! - Capturing a screenshot
//! - Saving as PNG

#[cfg(feature = "macos_14_0")]
use screencapturekit::prelude::*;
#[cfg(feature = "macos_14_0")]
use screencapturekit::screenshot_manager::SCScreenshotManager;

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
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!("Display: {}x{}", display.width(), display.height());

    // 2. Create filter
    let filter = SCContentFilter::builder()
        .display(&display)
        .exclude_windows(&[])
        .build();

    // 3. Configure screenshot
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    // 4. Capture screenshot
    println!("Capturing...");
    let image = SCScreenshotManager::capture_image(&filter, &config)?;

    let width = image.width();
    let height = image.height();
    println!("Captured: {width}x{height}");

    // 5. Save as PNG using png crate
    let filename = "screenshot.png";
    let rgba_data = image.get_rgba_data()?;

    let file = std::fs::File::create(filename)?;
    let buf_writer = std::io::BufWriter::new(file);
    #[allow(clippy::cast_possible_truncation)]
    let mut encoder = png::Encoder::new(buf_writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba_data)?;

    println!("✅ Saved to {filename}");
    Ok(())
}
