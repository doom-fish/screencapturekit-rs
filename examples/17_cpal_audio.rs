//! Example: cpal audio playback integration
//!
//! Demonstrates capturing system audio and playing it back through cpal.
//!
//! Run with: cargo run --example 17_cpal_audio --features cpal
//!
//! Note: Requires screen recording permission and audio output device.

use screencapturekit::cpal_adapter::{create_output_callback, SckAudioInputStream};
use screencapturekit::prelude::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 cpal Audio Capture Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
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

    // Create SCK audio input stream
    let mut input = SckAudioInputStream::new(&filter)?;
    let buffer = input.ring_buffer().clone();

    println!(
        "🔊 Audio config: {}Hz, {} channels",
        input.sample_rate(),
        input.channels()
    );

    // Start capture - the callback receives samples (we also log)
    input.start(|samples, info| {
        static mut SAMPLE_COUNT: usize = 0;
        unsafe {
            SAMPLE_COUNT += samples.len();
            if SAMPLE_COUNT % (info.sample_rate as usize * 2) < samples.len() {
                println!(
                    "📥 Captured {} samples total",
                    SAMPLE_COUNT / info.channels as usize
                );
            }
        }
    })?;

    println!("🎬 Capture started");

    // Setup cpal output
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device found")?;

    println!("🔈 Output device: {}", device.name()?);

    let stream_config = input.cpal_config();
    let output_stream = device.build_output_stream(
        &stream_config,
        create_output_callback(buffer),
        |err| eprintln!("Audio output error: {}", err),
        None,
    )?;

    output_stream.play()?;
    println!("▶️  Audio playback started");
    println!();
    println!("Playing captured system audio for 10 seconds...");
    println!("(Make sure to play some audio on your Mac!)");

    std::thread::sleep(std::time::Duration::from_secs(10));

    // Cleanup
    drop(output_stream);
    input.stop()?;

    println!();
    println!("✅ Done!");

    Ok(())
}
