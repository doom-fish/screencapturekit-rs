//! SCFrameStatus Example
//!
//! Demonstrates how to use SCFrameStatus to efficiently process frames
//! by filtering out idle, blank, and suspended frames.

use screencapturekit::{
    cm::{CMSampleBuffer, SCFrameStatus},
    prelude::*,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct FrameStatusHandler {
    total_frames: Arc<AtomicUsize>,
    complete_frames: Arc<AtomicUsize>,
    idle_frames: Arc<AtomicUsize>,
    blank_frames: Arc<AtomicUsize>,
    suspended_frames: Arc<AtomicUsize>,
    started_frames: Arc<AtomicUsize>,
    stopped_frames: Arc<AtomicUsize>,
    unknown_frames: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameStatusHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if matches!(of_type, SCStreamOutputType::Screen) {
            let count = self.total_frames.fetch_add(1, Ordering::Relaxed);

            // Get and categorize frame status
            match sample.get_frame_status() {
                Some(SCFrameStatus::Complete) => {
                    self.complete_frames.fetch_add(1, Ordering::Relaxed);
                    if count < 3 {
                        println!("Frame {}: ✅ Complete - Full frame content", count);
                    }
                }
                Some(SCFrameStatus::Idle) => {
                    self.idle_frames.fetch_add(1, Ordering::Relaxed);
                    if count < 3 {
                        println!("Frame {}: 😴 Idle - No changes from previous", count);
                    }
                    // Skip processing idle frames to save CPU
                    return;
                }
                Some(SCFrameStatus::Blank) => {
                    self.blank_frames.fetch_add(1, Ordering::Relaxed);
                    println!("Frame {}: ⬜ Blank - Empty frame", count);
                    return;
                }
                Some(SCFrameStatus::Suspended) => {
                    self.suspended_frames.fetch_add(1, Ordering::Relaxed);
                    println!("Frame {}: ⏸️  Suspended - Capture paused", count);
                    return;
                }
                Some(SCFrameStatus::Started) => {
                    self.started_frames.fetch_add(1, Ordering::Relaxed);
                    println!("Frame {}: 🎬 Started - First frame", count);
                }
                Some(SCFrameStatus::Stopped) => {
                    self.stopped_frames.fetch_add(1, Ordering::Relaxed);
                    println!("Frame {}: 🛑 Stopped - Last frame", count);
                }
                None => {
                    self.unknown_frames.fetch_add(1, Ordering::Relaxed);
                    if count < 3 {
                        println!("Frame {}: ❓ Unknown - No status available", count);
                    }
                }
            }

            // Only process frames with actual content
            if let Some(status) = sample.get_frame_status() {
                if status.has_content() {
                    // Process the frame (e.g., encode, analyze, save)
                    if count % 60 == 0 {
                        println!("Frame {}: Processing complete frame...", count);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 SCFrameStatus Example\n");
    println!("This example demonstrates how frame status helps optimize frame processing.\n");

    // Get shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();

    if displays.is_empty() {
        return Err("No displays available".into());
    }

    let display = &displays[0];
    println!("Capturing from display: {}x{}\n", display.width(), display.height());

    // Create filter and configuration
    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);

    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?;

    // Create counters
    let total_frames = Arc::new(AtomicUsize::new(0));
    let complete_frames = Arc::new(AtomicUsize::new(0));
    let idle_frames = Arc::new(AtomicUsize::new(0));
    let blank_frames = Arc::new(AtomicUsize::new(0));
    let suspended_frames = Arc::new(AtomicUsize::new(0));
    let started_frames = Arc::new(AtomicUsize::new(0));
    let stopped_frames = Arc::new(AtomicUsize::new(0));
    let unknown_frames = Arc::new(AtomicUsize::new(0));

    // Create handler
    let handler = FrameStatusHandler {
        total_frames: Arc::clone(&total_frames),
        complete_frames: Arc::clone(&complete_frames),
        idle_frames: Arc::clone(&idle_frames),
        blank_frames: Arc::clone(&blank_frames),
        suspended_frames: Arc::clone(&suspended_frames),
        started_frames: Arc::clone(&started_frames),
        stopped_frames: Arc::clone(&stopped_frames),
        unknown_frames: Arc::clone(&unknown_frames),
    };

    // Create and start stream
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture for 5 seconds...\n");
    stream.start_capture()?;

    std::thread::sleep(Duration::from_secs(5));

    stream.stop_capture()?;

    // Print detailed statistics
    println!("\n╔════════════════════════════════════════╗");
    println!("║        Frame Status Statistics         ║");
    println!("╠════════════════════════════════════════╣");
    
    let total = total_frames.load(Ordering::Relaxed);
    let complete = complete_frames.load(Ordering::Relaxed);
    let idle = idle_frames.load(Ordering::Relaxed);
    let blank = blank_frames.load(Ordering::Relaxed);
    let suspended = suspended_frames.load(Ordering::Relaxed);
    let started = started_frames.load(Ordering::Relaxed);
    let stopped = stopped_frames.load(Ordering::Relaxed);
    let unknown = unknown_frames.load(Ordering::Relaxed);

    println!("║ Total frames:      {:>6}             ║", total);
    println!("║ ✅ Complete:        {:>6} ({:>5.1}%)    ║", 
        complete, complete as f64 / total as f64 * 100.0);
    println!("║ 😴 Idle:            {:>6} ({:>5.1}%)    ║", 
        idle, idle as f64 / total as f64 * 100.0);
    
    if blank > 0 {
        println!("║ ⬜ Blank:           {:>6} ({:>5.1}%)    ║", 
            blank, blank as f64 / total as f64 * 100.0);
    }
    if suspended > 0 {
        println!("║ ⏸️  Suspended:       {:>6} ({:>5.1}%)    ║", 
            suspended, suspended as f64 / total as f64 * 100.0);
    }
    if started > 0 {
        println!("║ 🎬 Started:         {:>6}             ║", started);
    }
    if stopped > 0 {
        println!("║ 🛑 Stopped:         {:>6}             ║", stopped);
    }
    if unknown > 0 {
        println!("║ ❓ Unknown:         {:>6} ({:>5.1}%)    ║", 
            unknown, unknown as f64 / total as f64 * 100.0);
    }
    
    println!("╚════════════════════════════════════════╝\n");

    // Calculate efficiency
    let skipped = idle + blank + suspended;
    let processed = complete + started;
    
    println!("💡 Efficiency Insights:");
    println!("   • Frames skipped: {} ({:.1}%)", 
        skipped, skipped as f64 / total as f64 * 100.0);
    println!("   • Frames processed: {} ({:.1}%)", 
        processed, processed as f64 / total as f64 * 100.0);
    println!("   • CPU saved: ~{:.1}% by skipping idle/blank frames", 
        skipped as f64 / total as f64 * 100.0);

    println!("\n📚 Tips:");
    println!("   • Use status.has_content() to filter processable frames");
    println!("   • Use status.is_complete() to verify full frame data");
    println!("   • Skip Idle frames when screen hasn't changed");
    println!("   • Skip Blank/Suspended frames to save processing");

    Ok(())
}
