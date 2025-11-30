//! Closure-based Handlers
//!
//! Demonstrates using closures instead of structs for stream handlers.
//! This example shows:
//! - Using closures as output handlers
//! - Using closures with custom dispatch queues
//! - Using `ErrorHandler` for delegate callbacks
//! - Combining multiple handler types

use screencapturekit::dispatch_queue::{DispatchQoS, DispatchQueue};
use screencapturekit::prelude::*;
use screencapturekit::stream::ErrorHandler;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Closure-based Handlers\n");

    // Get display
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!("Display: {}x{}\n", display.width(), display.height());

    // Create filter and config
    let filter = SCContentFilter::builder()
        .display(&display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    // =========================================================================
    // Example 1: Simple closure handler
    // =========================================================================
    println!("📌 Example 1: Simple closure handler");

    let frame_count = Arc::new(AtomicUsize::new(0));
    let count_clone = frame_count.clone();

    let mut stream = SCStream::new(&filter, &config);

    // Add handler using a closure directly
    stream.add_output_handler(
        move |_sample: CMSampleBuffer, output_type: SCStreamOutputType| {
            if output_type == SCStreamOutputType::Screen {
                let n = count_clone.fetch_add(1, Ordering::Relaxed);
                if n % 30 == 0 {
                    println!("   📹 Frame {n}");
                }
            }
        },
        SCStreamOutputType::Screen,
    );

    stream.start_capture()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.stop_capture()?;

    println!(
        "   ✅ Captured {} frames\n",
        frame_count.load(Ordering::Relaxed)
    );

    // =========================================================================
    // Example 2: Closure with custom dispatch queue
    // =========================================================================
    println!("📌 Example 2: Closure with custom dispatch queue");

    let frame_count = Arc::new(AtomicUsize::new(0));
    let count_clone = frame_count.clone();

    let mut stream = SCStream::new(&filter, &config);

    // Create a high-priority queue for frame processing
    let queue = DispatchQueue::new("com.example.capture", DispatchQoS::UserInteractive);

    // Add handler with custom queue
    stream.add_output_handler_with_queue(
        move |_sample: CMSampleBuffer, _output_type: SCStreamOutputType| {
            let n = count_clone.fetch_add(1, Ordering::Relaxed);
            if n % 30 == 0 {
                println!("   📹 Frame {n} (on custom queue)");
            }
        },
        SCStreamOutputType::Screen,
        Some(&queue),
    );

    stream.start_capture()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.stop_capture()?;

    println!(
        "   ✅ Captured {} frames\n",
        frame_count.load(Ordering::Relaxed)
    );

    // =========================================================================
    // Example 3: ErrorHandler for delegate callbacks
    // =========================================================================
    println!("📌 Example 3: ErrorHandler for delegate callbacks");

    // Create an error handler from a closure
    let error_handler = ErrorHandler::new(|error| {
        eprintln!("   ❌ Stream error: {error}");
    });

    // Create stream with delegate
    let mut stream = SCStream::new_with_delegate(&filter, &config, error_handler);

    let frame_count = Arc::new(AtomicUsize::new(0));
    let count_clone = frame_count.clone();

    stream.add_output_handler(
        move |_sample: CMSampleBuffer, _output_type: SCStreamOutputType| {
            count_clone.fetch_add(1, Ordering::Relaxed);
        },
        SCStreamOutputType::Screen,
    );

    stream.start_capture()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.stop_capture()?;

    println!(
        "   ✅ Captured {} frames (with error handling)\n",
        frame_count.load(Ordering::Relaxed)
    );

    // =========================================================================
    // Example 4: Multiple handlers
    // =========================================================================
    println!("📌 Example 4: Multiple handlers on same stream");

    let video_count = Arc::new(AtomicUsize::new(0));
    let stats_count = Arc::new(AtomicUsize::new(0));

    let video_clone = video_count.clone();
    let stats_clone = stats_count.clone();

    let mut stream = SCStream::new(&filter, &config);

    // Handler 1: Count frames
    stream.add_output_handler(
        move |_sample: CMSampleBuffer, _output_type: SCStreamOutputType| {
            video_clone.fetch_add(1, Ordering::Relaxed);
        },
        SCStreamOutputType::Screen,
    );

    // Handler 2: Log every 60th frame
    stream.add_output_handler(
        move |_sample: CMSampleBuffer, _output_type: SCStreamOutputType| {
            let n = stats_clone.fetch_add(1, Ordering::Relaxed);
            if n % 60 == 0 {
                println!("   📊 Stats checkpoint at frame {n}");
            }
        },
        SCStreamOutputType::Screen,
    );

    stream.start_capture()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    stream.stop_capture()?;

    println!(
        "   ✅ Handler 1 counted: {} frames",
        video_count.load(Ordering::Relaxed)
    );
    println!(
        "   ✅ Handler 2 counted: {} frames\n",
        stats_count.load(Ordering::Relaxed)
    );

    println!("✨ All closure handler examples complete!");
    Ok(())
}
