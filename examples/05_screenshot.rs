//! Screenshot Capture (macOS 14.0+)
//!
//! Demonstrates taking screenshots using `SCScreenshotManager`.
//! This example shows:
//! - Using `SCScreenshotManager` (macOS 14.0+)
//! - Capturing a screenshot
//! - Getting content info (scale factor, dimensions)
//! - Capturing a specific screen region (macOS 15.2+)
//! - Advanced HDR screenshot capture (macOS 26.0+)
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

    // 3. Get content info (macOS 14.0+ feature)
    if let Some(info) =
        screencapturekit::shareable_content::SCShareableContentInfo::for_filter(&filter)
    {
        println!("\n📊 Content Info:");
        println!("   Style: {:?}", info.style());
        println!("   Scale: {:.1}x (Retina)", info.point_pixel_scale());
        let (pw, ph) = info.pixel_size();
        println!("   Pixel dimensions: {}x{}", pw, ph);
    }

    // 4. Configure screenshot
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    // 5. Capture full screenshot
    println!("\n📷 Capturing full screenshot...");
    let image = SCScreenshotManager::capture_image(&filter, &config)?;

    let width = image.width();
    let height = image.height();
    println!("   Captured: {width}x{height}");

    // 6. Save as PNG using png crate
    let filename = "screenshot.png";
    save_image_as_png(&image, filename)?;
    println!("   ✅ Saved to {filename}");

    // 7. Capture specific region (macOS 15.2+)
    #[cfg(feature = "macos_15_2")]
    {
        use screencapturekit::cg::CGRect;

        println!("\n📷 Capturing screen region (macOS 15.2+)...");
        let rect = CGRect::new(100.0, 100.0, 640.0, 480.0);
        match SCScreenshotManager::capture_image_in_rect(rect) {
            Ok(region_image) => {
                let filename = "screenshot_region.png";
                save_image_as_png(&region_image, filename)?;
                println!(
                    "   Captured region: {}x{}",
                    region_image.width(),
                    region_image.height()
                );
                println!("   ✅ Saved to {filename}");
            }
            Err(e) => {
                println!("   ⚠️  Region capture failed: {e}");
            }
        }
    }

    // 8. Advanced HDR screenshot (macOS 26.0+)
    #[cfg(feature = "macos_26_0")]
    {
        use screencapturekit::screenshot_manager::{
            SCScreenshotConfiguration, SCScreenshotDynamicRange,
        };

        println!("\n🌈 Advanced HDR Screenshot (macOS 26.0+)...");

        // Create advanced configuration with HDR support
        let screenshot_config = SCScreenshotConfiguration::new()
            .with_width(1920)
            .with_height(1080)
            .with_shows_cursor(true)
            .with_dynamic_range(SCScreenshotDynamicRange::BothSDRAndHDR)
            .with_ignore_shadows(false)
            .with_include_child_windows(true);

        match SCScreenshotManager::capture_screenshot(&filter, &screenshot_config) {
            Ok(output) => {
                // Get SDR image
                if let Some(sdr) = output.sdr_image() {
                    let filename = "screenshot_sdr.png";
                    save_image_as_png(&sdr, filename)?;
                    println!("   SDR: {}x{} → {filename}", sdr.width(), sdr.height());
                }

                // Get HDR image (if available on HDR display)
                if let Some(hdr) = output.hdr_image() {
                    let filename = "screenshot_hdr.png";
                    save_image_as_png(&hdr, filename)?;
                    println!("   HDR: {}x{} → {filename}", hdr.width(), hdr.height());
                } else {
                    println!("   HDR: Not available (requires HDR display)");
                }

                println!("   ✅ Advanced screenshot complete");
            }
            Err(e) => {
                println!("   ⚠️  Advanced screenshot failed: {e}");
            }
        }

        // Save directly to file
        println!("\n💾 Screenshot with file output (macOS 26.0+)...");
        let file_config = SCScreenshotConfiguration::new()
            .with_width(1920)
            .with_height(1080)
            .with_file_path("screenshot_direct.png");

        match SCScreenshotManager::capture_screenshot(&filter, &file_config) {
            Ok(output) => {
                if let Some(url) = output.file_url() {
                    println!("   ✅ Saved directly to: {url}");
                } else {
                    println!("   ✅ Screenshot captured (file save may be async)");
                }
            }
            Err(e) => {
                println!("   ⚠️  File screenshot failed: {e}");
            }
        }
    }

    println!("\n✅ Screenshot example completed!");
    Ok(())
}

#[cfg(feature = "macos_14_0")]
fn save_image_as_png(
    image: &screencapturekit::screenshot_manager::CGImage,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let rgba_data = image.rgba_data()?;
    let file = std::fs::File::create(filename)?;
    let buf_writer = std::io::BufWriter::new(file);
    #[allow(clippy::cast_possible_truncation)]
    let mut encoder = png::Encoder::new(buf_writer, image.width() as u32, image.height() as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba_data)?;
    Ok(())
}
