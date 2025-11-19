//! Screen capture permissions example
//!
//! Demonstrates how to check and handle screen recording permissions.
//!
//! This example shows:
//! - Requesting screen recording permission
//! - Handling permission errors
//! - Verifying successful capture

use screencapturekit::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

struct FrameCounter {
    count: Arc<AtomicUsize>,
    received: Arc<AtomicBool>,
}

impl SCStreamOutputTrait for FrameCounter {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        self.received.store(true, Ordering::SeqCst);
        let count = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        if count % 30 == 0 {
            println!("Captured {} frames", count);
        }
    }
}

fn main() {
    println!("=== ScreenCaptureKit Permission Test ===");
    println!();
    println!("This example will attempt to capture the screen.");
    println!("If this is the first time running, you'll need to grant Screen Recording");
    println!("permission in System Settings > Privacy & Security > Screen Recording");
    println!();
    
    // Get shareable content
    println!("Getting shareable content...");
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to get shareable content: {}", e);
            eprintln!("You may need to grant Screen Recording permission");
            return;
        }
    };
    
    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("No displays found!");
        return;
    }
    
    println!("Found {} display(s)", displays.len());
    let display = &displays[0];
    println!("Using display: {:?}", display.display_id());
    
    // Create filter
    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);
    
    // Create configuration
    let frame_time = CMTime::new(1, 60);
    let config = SCStreamConfiguration::build()
        .set_width(1920).ok().unwrap()
        .set_height(1080).ok().unwrap()
        .set_pixel_format(PixelFormat::BGRA).ok().unwrap()
        .set_shows_cursor(true).ok().unwrap()
        .set_minimum_frame_interval(&frame_time).ok().unwrap();
    
    // Create output handler
    let frame_count = Arc::new(AtomicUsize::new(0));
    let received = Arc::new(AtomicBool::new(false));
    
    let output = FrameCounter {
        count: frame_count.clone(),
        received: received.clone(),
    };
    
    // Create stream
    println!("Creating stream...");
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(output, SCStreamOutputType::Screen);
    
    // Start capture
    println!("Starting capture...");
    match stream.start_capture() {
        Ok(_) => println!("✓ Capture started successfully!"),
        Err(e) => {
            eprintln!("✗ Failed to start capture: {}", e);
            eprintln!("  Make sure Screen Recording permission is granted");
            return;
        }
    }
    
    // Wait for frames
    println!("Capturing for 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    // Stop capture
    println!("Stopping capture...");
    match stream.stop_capture() {
        Ok(_) => println!("✓ Capture stopped"),
        Err(e) => eprintln!("✗ Failed to stop capture: {}", e),
    }
    
    let count = frame_count.load(Ordering::SeqCst);
    let got_frames = received.load(Ordering::SeqCst);
    
    println!();
    println!("=== Results ===");
    println!("Total frames captured: {}", count);
    println!("Frames received: {}", got_frames);
    
    if got_frames && count > 0 {
        println!();
        println!("✓ SUCCESS! Screen capture is working correctly");
        println!("  Average FPS: {:.1}", count as f64 / 5.0);
    } else {
        println!();
        println!("✗ No frames captured");
        println!("  Please check Screen Recording permission in System Settings");
    }
}
