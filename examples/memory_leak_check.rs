//! Memory Leak Detection Example
//!
//! This example demonstrates how to check for memory leaks using macOS's `leaks` tool.
//! It creates and destroys streams multiple times, then uses the `leaks` command to
//! verify no memory is leaked.
//!
//! # Usage
//! ```sh
//! cargo run --example memory_leak_check
//! ```
//!
//! # Note
//! This uses the macOS `leaks` command which requires running as a standalone process.
//! Some Apple framework leaks in ScreenCaptureKit itself are expected and ignored.

use screencapturekit::{
    cm::CMSampleBuffer,
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
    },
};
use std::{process::Command, thread, time::Duration};

/// Simple handler that processes sample buffers
struct LeakTestHandler;

impl LeakTestHandler {
    fn new() -> Self {
        Self
    }
}

impl SCStreamOutputTrait for LeakTestHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        // Access the sample to ensure it's valid
        let _timestamp = sample.presentation_timestamp();
    }
}

fn main() {
    println!("🔍 Memory Leak Detection Test");
    println!("==============================\n");

    let iterations = 4;
    let capture_duration = Duration::from_millis(500);

    println!("Configuration:");
    println!("  • Iterations: {iterations}");
    println!("  • Capture duration per iteration: {capture_duration:?}");
    println!();

    // Run multiple capture cycles
    for i in 1..=iterations {
        println!("📹 Iteration {i}/{iterations}...");

        let stream = create_capture_stream();

        if let Err(e) = stream.start_capture() {
            eprintln!("  ⚠️  Failed to start capture: {e}");
            continue;
        }

        thread::sleep(capture_duration);

        if let Err(e) = stream.stop_capture() {
            eprintln!("  ⚠️  Failed to stop capture: {e}");
        }

        // Explicitly drop the stream
        drop(stream);
        println!("  ✓ Stream created, captured, and dropped");
    }

    println!("\n🧪 Running leak analysis...\n");

    // Run the macOS leaks command
    let result = check_for_leaks();

    match result {
        LeakResult::NoLeaks => {
            println!("✅ No memory leaks detected!");
        }
        LeakResult::AppleFrameworkLeaksOnly => {
            println!("✅ No leaks in our code (Apple framework leaks detected but ignored)");
        }
        LeakResult::LeaksDetected(details) => {
            println!("❌ Memory leaks detected in our code!");
            println!("\nDetails:\n{details}");
            std::process::exit(1);
        }
        LeakResult::Error(msg) => {
            println!("⚠️  Could not run leak check: {msg}");
            std::process::exit(2);
        }
    }
}

fn create_capture_stream() -> SCStream {
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let displays = content.displays();
    let display = displays.first().expect("No display found");

    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(100)
        .with_height(100)
        .with_captures_audio(true);

    let mut stream = SCStream::new(&filter, &config);

    // Add handlers for both video and audio
    stream.add_output_handler(LeakTestHandler::new(), SCStreamOutputType::Screen);
    stream.add_output_handler(LeakTestHandler::new(), SCStreamOutputType::Audio);

    stream
}

enum LeakResult {
    NoLeaks,
    AppleFrameworkLeaksOnly,
    LeaksDetected(String),
    Error(String),
}

fn check_for_leaks() -> LeakResult {
    let pid = std::process::id();

    let output = match Command::new("leaks")
        .args([pid.to_string(), "-c".to_string()])
        .output()
    {
        Ok(output) => output,
        Err(e) => return LeakResult::Error(format!("Failed to execute leaks command: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print raw output for debugging
    if !stdout.is_empty() {
        println!("leaks stdout:\n{stdout}");
    }
    if !stderr.is_empty() {
        println!("leaks stderr:\n{stderr}");
    }

    // Check for no leaks
    if stdout.contains("0 leaks for 0 total leaked bytes") {
        return LeakResult::NoLeaks;
    }

    // Check if all leaks are from Apple frameworks (not our code)
    let apple_framework_leaks = stdout.contains("CMCapture")
        || stdout.contains("FigRemoteOperationReceiver")
        || stdout.contains("SCStream(SCContentSharing)")
        || stdout.contains("CoreMedia")
        || stdout.contains("VideoToolbox");

    let our_code_leaks = stdout.contains("screencapturekit");

    if apple_framework_leaks && !our_code_leaks {
        return LeakResult::AppleFrameworkLeaksOnly;
    }

    LeakResult::LeaksDetected(stdout.to_string())
}
