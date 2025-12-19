//! Audio Capture
//!
//! Demonstrates capturing audio along with video.
//! This example shows:
//! - Enabling audio capture
//! - Configuring audio settings
//! - Handling both video and audio callbacks

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Handler {
    video_count: Arc<AtomicUsize>,
    audio_count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        match output_type {
            SCStreamOutputType::Screen => {
                let n = self.video_count.fetch_add(1, Ordering::Relaxed);
                if n % 60 == 0 {
                    println!("📹 Video: {n} frames");
                }
            }
            SCStreamOutputType::Audio => {
                let n = self.audio_count.fetch_add(1, Ordering::Relaxed);
                if n % 100 == 0 {
                    println!("🔊 Audio: {n} buffers");
                }
            }
            SCStreamOutputType::Microphone => {
                println!("🎤 Microphone buffer");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔊 Audio + Video Capture\n");

    // 1. Get display
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    // 2. Create filter
    let filter = SCContentFilter::with()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // 3. Configure with audio enabled
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_captures_audio(true) // Enable audio
        .with_sample_rate(48000) // 48kHz
        .with_channel_count(2); // Stereo

    // 4. Start capture
    let handler = Handler {
        video_count: Arc::new(AtomicUsize::new(0)),
        audio_count: Arc::new(AtomicUsize::new(0)),
    };

    let video_count = handler.video_count.clone();
    let audio_count = handler.audio_count.clone();

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...\n");
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;

    println!("\n✅ Captured:");
    println!("   Video: {} frames", video_count.load(Ordering::Relaxed));
    println!("   Audio: {} buffers", audio_count.load(Ordering::Relaxed));

    Ok(())
}
