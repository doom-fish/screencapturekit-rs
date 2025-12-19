//! Dynamic Stream Updates
//!
//! Demonstrates updating a running stream's configuration and content filter.
//! This example shows:
//! - Updating stream configuration while capturing (resolution, frame rate)
//! - Switching capture source via content filter update
//! - Using the synchronization clock for A/V sync
//! - Stream delegate for error handling
//!
//! Run with: `cargo run --example 12_stream_updates --features macos_14_0`

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if matches!(of_type, SCStreamOutputType::Screen) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Dynamic Stream Updates\n");

    // 1. Get available content
    let content = SCShareableContent::get()?;
    let displays = content.displays();

    if displays.is_empty() {
        println!("⚠️  Need at least 1 display for this example");
        return Ok(());
    }

    let display = &displays[0];
    println!("📺 Using display: {}x{}", display.width(), display.height());

    // 2. Initial configuration - low resolution
    let filter = SCContentFilter::with()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(640)
        .with_height(480)
        .with_pixel_format(PixelFormat::BGRA);

    println!("📐 Initial config: 640x480");

    // 3. Create and start stream
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler {
        count: count.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    println!("▶️  Capture started\n");

    // 4. Check synchronization clock (macOS 13.0+)
    #[cfg(feature = "macos_13_0")]
    if let Some(clock) = stream.synchronization_clock() {
        println!("⏱️  Sync clock available:");
        let time = clock.time();
        println!("   Current time: {}/{} seconds", time.value, time.timescale);
    }

    // Capture at low resolution for 2 seconds
    std::thread::sleep(Duration::from_secs(2));
    let frames_low = count.load(Ordering::Relaxed);
    println!("📊 Frames at 640x480: {frames_low}");

    // 5. Update configuration to higher resolution
    println!("\n🔄 Updating to 1920x1080...");
    let new_config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    match stream.update_configuration(&new_config) {
        Ok(()) => println!("✅ Configuration updated"),
        Err(e) => println!("❌ Update failed: {e:?}"),
    }

    // Capture at high resolution for 2 seconds
    std::thread::sleep(Duration::from_secs(2));
    let frames_high = count.load(Ordering::Relaxed);
    println!(
        "📊 Frames at 1920x1080: {} (total: {})",
        frames_high - frames_low,
        frames_high
    );

    // 6. Update content filter (switch to window if available)
    let windows = content.windows();
    if let Some(window) = windows.iter().find(|w| w.is_on_screen()) {
        println!("\n🔄 Switching to window capture...");
        println!("   Window: {}", window.title().unwrap_or_default());

        let window_filter = SCContentFilter::with().window(window).build();

        match stream.update_content_filter(&window_filter) {
            Ok(()) => println!("✅ Filter updated to window"),
            Err(e) => println!("❌ Filter update failed: {e:?}"),
        }

        std::thread::sleep(Duration::from_secs(2));
        let frames_window = count.load(Ordering::Relaxed);
        println!(
            "📊 Frames from window: {} (total: {})",
            frames_window - frames_high,
            frames_window
        );
    }

    // 7. Stop capture
    stream.stop_capture()?;
    println!("\n⏹️  Capture stopped");
    println!(
        "✅ Total frames captured: {}",
        count.load(Ordering::Relaxed)
    );

    Ok(())
}
