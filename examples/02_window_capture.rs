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
        let app_name = window
            .owning_application()
            .map(|app| app.application_name())
            .unwrap_or_default();

        println!(
            "  {}. {} - {} ({}x{})",
            i + 1,
            app_name,
            window.title().unwrap_or_default(),
            window.frame().width,
            window.frame().height
        );
    }

    // 3. Find a window (example: Safari)
    let window = windows
        .iter()
        .find(|w| {
            w.owning_application()
                .is_some_and(|app| app.application_name().contains("Safari"))
        })
        .ok_or("Safari window not found. Try another app.")?;

    println!("\nCapturing: {}\n", window.title().unwrap_or_default());

    // 4. Create window filter
    let filter = SCContentFilter::builder().window(window).build();

    // 5. Configure stream
    let config = SCStreamConfiguration::builder()
        .width(1920)
        .height(1080)
        .build();

    // 6. Start capture
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler {
        count: count.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;

    println!("✅ Captured {} frames", count.load(Ordering::Relaxed));
    Ok(())
}
