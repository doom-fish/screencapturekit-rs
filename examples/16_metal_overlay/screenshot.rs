//! Screenshot capture logic

use screencapturekit::prelude::*;
use screencapturekit::screenshot_manager::SCScreenshotManager;
use screencapturekit::stream::content_filter::SCContentFilter;

/// Take a screenshot using the best available API
/// - macOS 26.0+: Uses `SCScreenshotConfiguration` with native file saving
/// - macOS 14.0+: Uses `SCStreamConfiguration` and `CGImage::save_png()`
pub fn take_screenshot(
    filter: &SCContentFilter,
    capture_size: (u32, u32),
    stream_config: &SCStreamConfiguration,
) {
    println!("📸 Taking screenshot...");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("/tmp/screenshot_{timestamp}.png");

    #[cfg(feature = "macos_26_0")]
    {
        use screencapturekit::screenshot_manager::SCScreenshotConfiguration;

        // Use the new macOS 26.0 API with native file saving
        let config = SCScreenshotConfiguration::new()
            .with_width(capture_size.0 as usize)
            .with_height(capture_size.1 as usize)
            .with_shows_cursor(stream_config.shows_cursor())
            .with_file_path(&path);

        match SCScreenshotManager::capture_screenshot(filter, &config) {
            Ok(output) => {
                if let Some(url) = output.file_url() {
                    println!("✅ Screenshot saved to {url}");
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                } else if let Some(image) = output.sdr_image() {
                    println!(
                        "✅ Screenshot captured: {}x{}",
                        image.width(),
                        image.height()
                    );
                    match image.save_png(&path) {
                        Ok(()) => {
                            println!("📁 Saved to {path}");
                            let _ = std::process::Command::new("open").arg(&path).spawn();
                        }
                        Err(e) => eprintln!("❌ Failed to save: {e:?}"),
                    }
                }
            }
            Err(e) => eprintln!("❌ Screenshot failed: {e:?}"),
        }
    }

    #[cfg(not(feature = "macos_26_0"))]
    {
        // Use macOS 14.0+ API
        let screenshot_config = SCStreamConfiguration::new()
            .with_width(capture_size.0)
            .with_height(capture_size.1)
            .with_shows_cursor(stream_config.shows_cursor());

        match SCScreenshotManager::capture_image(filter, &screenshot_config) {
            Ok(image) => {
                println!(
                    "✅ Screenshot captured: {}x{}",
                    image.width(),
                    image.height()
                );
                match image.save_png(&path) {
                    Ok(()) => {
                        println!("📁 Saved to {}", path);
                        let _ = std::process::Command::new("open").arg(&path).spawn();
                    }
                    Err(e) => eprintln!("❌ Failed to save: {:?}", e),
                }
            }
            Err(e) => eprintln!("❌ Screenshot failed: {:?}", e),
        }
    }
}
