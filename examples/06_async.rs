//! Async API
//!
//! Demonstrates async/await API (requires "async" feature).
//! This example shows:
//! - Async content retrieval
//! - Async screenshot capture
//! - Works with any async runtime (Tokio shown here)

#[cfg(not(feature = "async"))]
fn main() {
    println!("⚠️  This example requires the 'async' feature");
    println!("    Run with: cargo run --example 06_async --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCScreenshotManager};
    use screencapturekit::prelude::*;

    println!("⚡ Async API Demo\n");

    // 1. Get content asynchronously
    println!("Fetching content...");
    let content = AsyncSCShareableContent::get().await?;
    
    let displays = content.displays();
    println!("Found {} displays", displays.len());
    
    for display in &displays {
        println!("  - Display {}: {}x{}", 
            display.display_id(),
            display.width(), 
            display.height()
        );
    }

    // 2. Capture screenshot asynchronously
    if let Some(display) = displays.first() {
        println!("\nCapturing screenshot...");
        
        let filter = SCContentFilter::build()
            .display(display)
            .exclude_windows(&[])
            .build();
        
        let config = SCStreamConfiguration::build()
            .set_width(1920)?
            .set_height(1080)?;
        
        let image = AsyncSCScreenshotManager::capture_image(&filter, &config).await?;
        
        println!("Captured: {}x{}", image.width(), image.height());
        
        // Save screenshot
        image.save_to_png("async_screenshot.png")?;
        println!("✅ Saved to async_screenshot.png");
    }

    Ok(())
}
