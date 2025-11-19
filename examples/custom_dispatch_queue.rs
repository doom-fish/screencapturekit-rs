//! Example demonstrating custom dispatch queue usage with SCStream
//!
//! This example shows how to provide your own dispatch queue for handling
//! stream output callbacks, allowing more control over the threading behavior.

use screencapturekit::prelude::*;

struct MyOutputHandler;

impl SCStreamOutputTrait for MyOutputHandler {
    fn did_output_sample_buffer(&self, _sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                println!("Received screen frame");
            }
            SCStreamOutputType::Audio => {
                println!("Received audio buffer");
            }
            SCStreamOutputType::Microphone => {
                println!("Received microphone buffer");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Custom Dispatch Queue Example");
    println!("==============================\n");

    // Get shareable content
    println!("Fetching shareable content...");
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    
    if displays.is_empty() {
        eprintln!("No displays available");
        return Ok(());
    }
    
    let display = &displays[0];
    println!("Using display: {}x{}", display.width(), display.height());

    // Create filter and configuration
    #[allow(deprecated)]
    let filter = SCContentFilter::new()
        .with_display_excluding_windows(display, &[]);
    
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?;

    // Example 1: Using default queue (backward compatible)
    println!("\nExample 1: Using default queue");
    let mut stream1 = SCStream::new(&filter, &config);
    let handler1 = MyOutputHandler;
    stream1.add_output_handler(handler1, SCStreamOutputType::Screen);
    println!("Added handler with default queue");

    // Example 2: Using custom high-priority queue
    println!("\nExample 2: Using custom high-priority queue");
    let custom_queue = DispatchQueue::new(
        "com.example.capture.highprio", 
        DispatchQoS::UserInteractive
    );
    
    let mut stream2 = SCStream::new(&filter, &config);
    let handler2 = MyOutputHandler;
    stream2.add_output_handler_with_queue(
        handler2, 
        SCStreamOutputType::Screen,
        Some(&custom_queue)
    );
    println!("Added handler with custom high-priority queue");

    // Example 3: Using custom background queue for processing
    println!("\nExample 3: Using custom background queue");
    let background_queue = DispatchQueue::new(
        "com.example.capture.background", 
        DispatchQoS::Background
    );
    
    let mut stream3 = SCStream::new(&filter, &config);
    let handler3 = MyOutputHandler;
    stream3.add_output_handler_with_queue(
        handler3, 
        SCStreamOutputType::Screen,
        Some(&background_queue)
    );
    println!("Added handler with custom background queue");

    println!("\nAll examples configured successfully!");
    println!("\nAvailable QoS levels:");
    println!("  - Background: For maintenance or cleanup tasks");
    println!("  - Utility: For tasks that may take some time");
    println!("  - Default: Standard priority");
    println!("  - UserInitiated: For user-initiated tasks");
    println!("  - UserInteractive: For UI-affecting tasks (highest priority)");

    Ok(())
}
