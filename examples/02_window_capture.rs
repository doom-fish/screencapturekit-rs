//! Window Capture
//!
//! Demonstrates capturing a specific window.
//! This example shows:
//! - Listing available windows
//! - Filtering windows by title
//! - Creating window-specific content filter

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _type: SCStreamOutputType) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🪟 Window Capture\n");

    // 1. Get available content
    let content = SCShareableContent::get()?;
    let windows = content.windows();
    
    // 2. List windows (optional - for demonstration)
    println!("Available windows:");
    for (i, window) in windows.iter().take(10).enumerate() {
        let app_name = window.owning_application()
            .as_ref()
            .map(|app| app.application_name())
            .unwrap_or_else(|| "Unknown".to_string());
        let title = window.title().as_deref().unwrap_or("Untitled").to_string();
        
        println!("  {}. {} - {} ({}x{})",
            i + 1,
            app_name,
            title,
            window.frame().width,
            window.frame().height
        );
    }
    
    // 3. Find a window (example: Safari)
    let window = windows
        .iter()
        .find(|w| {
            w.owning_application()
                .as_ref()
                .map(|app| app.application_name().contains("Safari"))
                .unwrap_or(false)
        })
        .ok_or("Safari window not found. Try another app.")?;
    
    let window_title = window.title().as_deref().unwrap_or("Untitled").to_string();
    println!("\nCapturing: {}\n", window_title);

    // 4. Create window filter
    let filter = SCContentFilter::build()
        .window(window)
        .build();

    // 5. Configure stream
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?;

    // 6. Start capture
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler { count: count.clone() };
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;
    
    println!("✅ Captured {} frames", count.load(Ordering::Relaxed));
    Ok(())
}
