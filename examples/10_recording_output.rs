//! Recording Output Example
//!
//! Demonstrates direct video file recording (macOS 15.0+).
//!
//! Run with: cargo run --example 10_recording_output --features macos_15_0

#[cfg(not(feature = "macos_15_0"))]
fn main() {
    eprintln!("This example requires the 'macos_15_0' feature flag.");
    eprintln!("Run with: cargo run --example 10_recording_output --features macos_15_0");
}

#[cfg(feature = "macos_15_0")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::recording_output::{
        SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    };
    use std::path::PathBuf;

    println!("=== Recording Output Example (macOS 15.0+) ===\n");

    // Create output configuration
    let mut config = SCRecordingOutputConfiguration::new();

    // Set output file path
    let output_path = PathBuf::from("/tmp/screen_recording.mp4");
    config.set_output_url(&output_path);
    println!("Output path: {:?}", output_path);

    // Set video codec (H.264 or HEVC)
    config.set_video_codec(SCRecordingOutputCodec::H264);
    println!("Video codec: H.264");

    // Set bitrate (10 Mbps)
    config.set_average_bitrate(10_000_000);
    println!("Bitrate: 10 Mbps");

    // Create recording output
    if let Some(recording_output) = SCRecordingOutput::new(&config) {
        println!("\n✅ Recording output created successfully!");
        println!("   Pointer: {:?}", recording_output.as_ptr());

        // Clone test
        let cloned = recording_output.clone();
        println!("   Clone pointer: {:?}", cloned.as_ptr());
    } else {
        println!("\n⚠️  Recording output creation failed.");
        println!("   This may happen if macOS 15.0+ is not available.");
    }

    // Test configuration cloning
    let config_clone = config.clone();
    println!("\n📋 Configuration cloned successfully");
    println!("   Original ptr: {:?}", config.as_ptr());
    println!("   Clone ptr: {:?}", config_clone.as_ptr());

    // Test different codecs
    println!("\n🎬 Available codecs:");
    println!("   - H264 = {}", SCRecordingOutputCodec::H264 as i32);
    println!("   - HEVC = {}", SCRecordingOutputCodec::HEVC as i32);

    println!("\n✅ Recording output example completed!");
    Ok(())
}
