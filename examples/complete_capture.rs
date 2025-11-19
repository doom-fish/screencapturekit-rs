//! Complete capture example
//!
//! Demonstrates capturing both video and audio simultaneously.
//!
//! This example shows:
//! - Setting up video and audio capture
//! - Processing multiple output types
//! - Monitoring frame rates and buffer counts

use screencapturekit::{
    cm::CMSampleBuffer,
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration,
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        sc_stream::SCStream,
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct CaptureHandler {
    video_count: Arc<AtomicUsize>,
    audio_count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for CaptureHandler {
    fn did_output_sample_buffer(&self, _sample_buffer: CMSampleBuffer, output_type: SCStreamOutputType) {
        match output_type {
            SCStreamOutputType::Screen => {
                let count = self.video_count.fetch_add(1, Ordering::Relaxed);
                if count % 30 == 0 {
                    println!("📹 Video frame {}", count);
                }
            }
            SCStreamOutputType::Audio => {
                let count = self.audio_count.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    println!("🔊 Audio buffer {}", count);
                }
            }
            SCStreamOutputType::Microphone => {
                // Handle microphone audio (macOS 15.0+)
                println!("🎤 Microphone buffer");
            }
        }
    }
}

fn main() {
    println!("🎬 Starting complete capture test...\n");

    // Get shareable content
    println!("📋 Fetching shareable content...");
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Failed to get shareable content: {:?}", e);
            return;
        }
    };
    
    let mut displays = content.displays();
    let windows = content.windows();
    
    println!("Found {} displays and {} windows", displays.len(), windows.len());
    
    if displays.is_empty() {
        println!("❌ No displays found!");
        return;
    }

    // Create content filter for the main display
    let display = displays.remove(0);
    println!("\n🖥️  Capturing display");
    
    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);

    // Configure stream for both video and audio
    let config = SCStreamConfiguration::build()
        .set_width(1920)
        .and_then(|c| c.set_height(1080))
        .and_then(|c| c.set_captures_audio(true))
        .and_then(|c| c.set_sample_rate(48000))
        .and_then(|c| c.set_channel_count(2));

    let config = match config {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Failed to configure stream: {:?}", e);
            return;
        }
    };

    println!("\n⚙️  Stream configured");

    // Create stream
    let mut stream = SCStream::new(&filter, &config);
    println!("\n✅ Stream created");

    // Set up output handler
    let video_count = Arc::new(AtomicUsize::new(0));
    let audio_count = Arc::new(AtomicUsize::new(0));
    
    let handler = CaptureHandler {
        video_count: video_count.clone(),
        audio_count: audio_count.clone(),
    };

    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    println!("✅ Output handler added");

    // Start capture
    println!("\n🎥 Starting capture...");
    match stream.start_capture() {
        Ok(_) => println!("✅ Capture started successfully!"),
        Err(e) => {
            println!("❌ Failed to start capture: {:?}", e);
            return;
        }
    }

    // Capture for 5 seconds
    println!("\n⏱️  Capturing for 5 seconds...\n");
    thread::sleep(Duration::from_secs(5));

    // Stop capture
    println!("\n🛑 Stopping capture...");
    match stream.stop_capture() {
        Ok(_) => println!("✅ Capture stopped successfully!"),
        Err(e) => println!("❌ Failed to stop capture: {:?}", e),
    }

    // Print statistics
    let final_video = video_count.load(Ordering::Relaxed);
    let final_audio = audio_count.load(Ordering::Relaxed);
    
    println!("\n📊 Capture Statistics:");
    println!("  Video frames: {}", final_video);
    println!("  Audio buffers: {}", final_audio);
    println!("  Avg video FPS: {:.1}", final_video as f64 / 5.0);
    println!("  Avg audio buffers/sec: {:.1}", final_audio as f64 / 5.0);

    if final_video > 0 && final_audio > 0 {
        println!("\n✅ SUCCESS: Both video and audio captured!");
    } else if final_video > 0 {
        println!("\n⚠️  WARNING: Only video captured (no audio)");
    } else {
        println!("\n❌ FAILED: No frames captured");
    }
}
