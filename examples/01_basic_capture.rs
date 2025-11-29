//! Basic Screen Capture
//!
//! Demonstrates the simplest way to capture screen content.
//! This example shows:
//! - Getting shareable content (displays)
//! - Creating a content filter
//! - Configuring stream settings (including new macOS 14.0+/15.0+ options)
//! - Starting and stopping capture
//! - Using both struct handlers and closure handlers

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Method 1: Struct-based handler
struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _type: SCStreamOutputType) {
        let n = self.count.fetch_add(1, Ordering::Relaxed);
        if n % 30 == 0 {
            println!("📹 Frame {n}");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎥 Basic Screen Capture\n");

    // 1. Get available displays
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!("Display: {}x{}", display.width(), display.height());

    // 2. Create content filter (what to capture)
    let filter = SCContentFilter::builder()
        .display(&display)
        .exclude_windows(&[])
        .build();

    // Show filter info (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    {
        let scale = filter.point_pixel_scale();
        println!("Filter scale: {:.1}x", scale);
    }

    // 3. Configure stream (how to capture)
    let mut config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA)
        .with_shows_cursor(true);

    // macOS 14.0+ configuration options
    #[cfg(feature = "macos_14_0")]
    {
        config = config
            .with_ignores_shadows_display(false)
            .with_ignore_global_clip_display(false);
    }

    // macOS 15.0+ configuration options
    #[cfg(feature = "macos_15_0")]
    {
        // Show mouse click indicators (circle around cursor when clicking)
        config.set_shows_mouse_clicks(true);
        println!("Mouse click indicators: enabled");
    }

    // 4. Create stream
    let mut stream = SCStream::new(&filter, &config);

    // Method 1: Struct-based handler
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler {
        count: count.clone(),
    };
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    // Method 2: Closure-based handler (alternative approach)
    // Uncomment to use instead of struct handler:
    // let count = Arc::new(AtomicUsize::new(0));
    // let count_clone = count.clone();
    // stream.add_output_handler(
    //     move |_sample: CMSampleBuffer, _type: SCStreamOutputType| {
    //         let n = count_clone.fetch_add(1, Ordering::Relaxed);
    //         if n % 30 == 0 {
    //             println!("📹 Frame {n}");
    //         }
    //     },
    //     SCStreamOutputType::Screen
    // );

    println!("Starting capture...\n");
    stream.start_capture()?;

    // Capture for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));

    stream.stop_capture()?;

    println!("\n✅ Captured {} frames", count.load(Ordering::Relaxed));
    Ok(())
}
