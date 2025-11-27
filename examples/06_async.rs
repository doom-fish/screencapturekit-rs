//! Async API
//!
//! Demonstrates async/await API (requires "async" feature).
//! This example shows:
//! - Async content retrieval
//! - Async stream with frame iteration
//! - Works with any async runtime (Tokio shown here)

#[cfg(not(feature = "async"))]
fn main() {
    println!("⚠️  This example requires the 'async' feature");
    println!("    Run with: cargo run --example 06_async --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
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

    // 2. Capture frames asynchronously
    if let Some(display) = displays.first() {
        println!("\nStarting async capture...");
        
        let filter = SCContentFilter::build()
            .display(display)
            .exclude_windows(&[])
            .build();
        
        let config = SCStreamConfiguration::build()
            .set_width(1920)?
            .set_height(1080)?;
        
        // Create async stream with 30-frame buffer
        let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
        stream.start_capture().await?;
        
        // Capture 10 frames
        let mut count = 0;
        while count < 10 {
            if let Some(_frame) = stream.next().await {
                count += 1;
                println!("  Frame {}", count);
            }
        }
        
        stream.stop_capture().await?;
        println!("✅ Captured {} frames", count);
    }

    Ok(())
}
