//! Basic Screen Capture
//!
//! Demonstrates the simplest way to capture screen content.
//! This example shows:
//! - Getting shareable content (displays)
//! - Creating a content filter
//! - Configuring stream settings
//! - Starting and stopping capture

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _type: SCStreamOutputType) {
        let n = self.count.fetch_add(1, Ordering::Relaxed);
        if n % 30 == 0 {
            println!("📹 Frame {}", n);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎥 Basic Screen Capture\n");

    // 1. Get available displays
    let content = SCShareableContent::get()?;
    let display = content.displays().into_iter().next()
        .ok_or("No displays found")?;
    
    println!("Display: {}x{}", display.width(), display.height());

    // 2. Create content filter (what to capture)
    let filter = SCContentFilter::build()
        .display(&display)
        .exclude_windows(&[])
        .build();

    // 3. Configure stream (how to capture)
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?
        .set_pixel_format(PixelFormat::BGRA)?;

    // 4. Create and start stream
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler { count: count.clone() };
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    
    println!("Starting capture...\n");
    stream.start_capture()?;

    // Capture for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));

    stream.stop_capture()?;
    
    println!("\n✅ Captured {} frames", count.load(Ordering::Relaxed));
    Ok(())
}
