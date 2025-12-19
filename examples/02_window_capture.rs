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

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes
fn init_cg() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _type: SCStreamOutputType) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_cg();
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

    // 3. Find a window - prefer Safari, but fallback to any visible window
    let window = windows
        .iter()
        .find(|w| {
            w.owning_application()
                .is_some_and(|app| app.application_name().contains("Safari"))
        })
        .or_else(|| {
            // Fallback: find any on-screen window with a title
            windows.iter().find(|w| {
                w.is_on_screen()
                    && w.frame().width > 100.0
                    && w.frame().height > 100.0
                    && w.title().is_some_and(|t| !t.is_empty())
            })
        })
        .or_else(|| {
            // Last resort: any on-screen window
            windows.iter().find(|w| w.is_on_screen())
        })
        .ok_or("No suitable window found")?;

    let app_name = window
        .owning_application()
        .map(|app| app.application_name())
        .unwrap_or_default();
    println!(
        "\nCapturing: {} - {}\n",
        app_name,
        window.title().unwrap_or_default()
    );

    // 4. Create window filter
    let filter = SCContentFilter::create().with_window(window).build();

    // 5. Configure stream
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

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
