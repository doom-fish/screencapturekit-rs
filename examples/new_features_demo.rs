//! Example demonstrating new ScreenCaptureKit features
//! 
//! This example shows how to use:
//! - HDR capture (macOS 15.0+)
//! - Microphone capture (macOS 15.0+)
//! - Stream naming
//! - excludesCurrentProcessAudio
//! - scalesToFit and preservesAspectRatio
//!
//! Requires: --features macos_15_0

use screencapturekit::prelude::*;
#[cfg(feature = "macos_15_0")]
use screencapturekit::stream::configuration::stream_properties::SCCaptureDynamicRange;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct NewFeaturesHandler {
    frame_count: Arc<AtomicUsize>,
    audio_count: Arc<AtomicUsize>,
    mic_count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for NewFeaturesHandler {
    fn did_output_sample_buffer(
        &self,
        _sample: screencapturekit::cm::CMSampleBuffer,
        of_type: SCStreamOutputType,
    ) {
        match of_type {
            SCStreamOutputType::Screen => {
                let count = self.frame_count.fetch_add(1, Ordering::Relaxed);
                if count % 60 == 0 {
                    println!("📹 HDR Frame: {}", count);
                }
            }
            SCStreamOutputType::Audio => {
                let count = self.audio_count.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    println!("🔊 System Audio: {}", count);
                }
            }
            SCStreamOutputType::Microphone => {
                let count = self.mic_count.fetch_add(1, Ordering::Relaxed);
                if count % 50 == 0 {
                    println!("🎤 Microphone: {}", count);
                }
            }
        }
    }
}

#[cfg(feature = "macos_15_0")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ScreenCaptureKit - New Features Demo\n");

    // Get shareable content
    println!("📋 Fetching shareable content...");
    let content = SCShareableContent::get()?;
    let displays = content.displays();

    if displays.is_empty() {
        eprintln!("❌ No displays found");
        return Ok(());
    }

    let display = &displays[0];
    println!("✅ Using display: {} ({}x{})", 
        display.display_id(),
        display.width(),
        display.height()
    );

    // Create filter
    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);

    // Configure stream with new features
    let config = SCStreamConfiguration::build()
        // Basic dimensions with new scaling options
        .set_width(1920)?
        .set_height(1080)?
        .set_scales_to_fit(true)?
        .set_preserves_aspect_ratio(true)?
        
        // HDR capture (macOS 15.0+)
        .set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay)?
        
        // Stream identification
        .set_stream_name(Some("new-features-demo"))?
        
        // Audio configuration with new features
        .set_captures_audio(true)?
        .set_sample_rate(48000)?
        .set_channel_count(2)?
        .set_excludes_current_process_audio(true)?
        
        // Microphone capture (macOS 15.0+)
        .set_captures_microphone(true)?;
        // Note: You can also specify a device ID:
        // .set_microphone_capture_device_id(Some("device-id-here"))?;

    println!("\n⚙️  Configuration:");
    println!("  - Resolution: {}x{}", config.get_width(), config.get_height());
    println!("  - Scales to fit: {}", config.get_scales_to_fit());
    println!("  - Preserves aspect ratio: {}", config.get_preserves_aspect_ratio());
    println!("  - Dynamic range: {:?}", config.get_capture_dynamic_range());
    if let Some(name) = config.get_stream_name() {
        println!("  - Stream name: {}", name);
    }
    println!("  - Excludes current process audio: {}", config.get_excludes_current_process_audio());
    println!("  - Captures microphone: {}", config.get_captures_microphone());

    // Create handler
    let handler = NewFeaturesHandler {
        frame_count: Arc::new(AtomicUsize::new(0)),
        audio_count: Arc::new(AtomicUsize::new(0)),
        mic_count: Arc::new(AtomicUsize::new(0)),
    };

    // Create and start stream
    let mut stream = SCStream::new(&filter, &config);
    
    println!("\n📡 Adding output handlers...");
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("▶️  Starting capture...\n");
    stream.start_capture()?;

    println!("⏳ Capturing for 10 seconds...");
    println!("   (Microphone requires macOS 15.0+)");
    println!("   (HDR requires macOS 15.0+)\n");
    
    thread::sleep(Duration::from_secs(10));

    println!("\n⏹️  Stopping capture...");
    stream.stop_capture()?;

    println!("✅ Demo complete!");
    Ok(())
}

#[cfg(not(feature = "macos_15_0"))]
fn main() {
    eprintln!("This example requires the macos_15_0 feature flag.");
    eprintln!("Run with: cargo run --example new_features_demo --features macos_15_0");
    std::process::exit(1);
}
