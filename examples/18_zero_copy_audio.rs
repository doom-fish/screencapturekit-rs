//! Example: Zero-copy audio capture
//!
//! Demonstrates zero-copy audio capture using ZeroCopyAudioStream.
//! The callback receives a direct pointer to CMSampleBuffer data.
//!
//! Run with: cargo run --example 18_zero_copy_audio --features cpal

use screencapturekit::cpal_adapter::ZeroCopyAudioStream;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Zero-Copy Audio Capture Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Get display to capture
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let display = displays.first().ok_or("No display found")?;

    println!(
        "📺 Capturing display: {}x{}",
        display.width(),
        display.height()
    );

    // Create content filter
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Counters for statistics
    let sample_count = Arc::new(AtomicUsize::new(0));
    let callback_count = Arc::new(AtomicUsize::new(0));
    let sample_count_clone = Arc::clone(&sample_count);
    let callback_count_clone = Arc::clone(&callback_count);

    println!("🚀 Starting zero-copy capture...");
    println!();

    // Start zero-copy capture
    // The callback runs on SCStream's thread with direct access to CMSampleBuffer data
    let stream = ZeroCopyAudioStream::start(&filter, move |samples, info| {
        // This is ZERO-COPY: `samples` points directly into CMSampleBuffer memory
        // No intermediate buffers, no copies!
        
        let count = callback_count_clone.fetch_add(1, Ordering::Relaxed);
        sample_count_clone.fetch_add(samples.len(), Ordering::Relaxed);

        // Example: compute RMS (root mean square) for volume level
        if count % 100 == 0 {
            let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
            let db = 20.0 * rms.max(1e-10).log10();
            println!(
                "📊 Callback #{}: {} samples, {:.1} dB ({}Hz, {} ch)",
                count,
                samples.len(),
                db,
                info.sample_rate,
                info.channels
            );
        }
    })?;

    println!("✅ Capture running for 5 seconds...");
    println!("   (Play some audio on your Mac to see levels)");
    println!();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Stop and show stats
    drop(stream);

    let total_samples = sample_count.load(Ordering::Relaxed);
    let total_callbacks = callback_count.load(Ordering::Relaxed);

    println!();
    println!("📈 Statistics:");
    println!("   Total callbacks: {}", total_callbacks);
    println!("   Total samples: {}", total_samples);
    println!(
        "   Avg samples/callback: {}",
        if total_callbacks > 0 {
            total_samples / total_callbacks
        } else {
            0
        }
    );
    println!();
    println!("✅ Done!");

    Ok(())
}
