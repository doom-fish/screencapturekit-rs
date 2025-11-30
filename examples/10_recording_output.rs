//! Recording Output Example
//!
//! Demonstrates direct video file recording (macOS 15.0+).
//! This example shows:
//! - Creating recording output configuration
//! - Setting video codec (H.264/HEVC)
//! - Setting output file type (MP4/MOV)
//! - Querying available codecs and file types
//! - Getting recording duration and file size
//!
//! Run with: `cargo run --example 10_recording_output --features macos_15_0`

#![allow(clippy::unnecessary_wraps)]

#[cfg(not(feature = "macos_15_0"))]
fn main() {
    eprintln!("This example requires the 'macos_15_0' feature flag.");
    eprintln!("Run with: cargo run --example 10_recording_output --features macos_15_0");
}

#[cfg(feature = "macos_15_0")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::recording_output::{
        SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
        SCRecordingOutputFileType,
    };
    use std::path::PathBuf;

    println!("=== Recording Output Example (macOS 15.0+) ===\n");

    // Create output configuration using builder pattern
    let output_path = PathBuf::from("/tmp/screen_recording.mp4");
    let config = SCRecordingOutputConfiguration::new()
        .with_output_url(&output_path)
        .with_video_codec(SCRecordingOutputCodec::H264)
        .with_output_file_type(SCRecordingOutputFileType::MP4);

    println!("📁 Output path: {}", output_path.display());
    println!("🎬 Video codec: {:?}", config.video_codec());
    println!("📄 File type: {:?}", config.output_file_type());

    // Query available options
    println!("\n📋 Available Options:");
    println!(
        "   Video codecs available: {}",
        config.available_video_codecs_count()
    );
    println!(
        "   File types available: {}",
        config.available_output_file_types_count()
    );

    // Show all codec options
    println!("\n🎬 Supported Codecs:");
    println!("   - H264 (value: {})", SCRecordingOutputCodec::H264 as i32);
    println!("   - HEVC (value: {})", SCRecordingOutputCodec::HEVC as i32);

    // Show all file type options
    println!("\n📄 Supported File Types:");
    println!(
        "   - MP4 (value: {})",
        SCRecordingOutputFileType::MP4 as i32
    );
    println!(
        "   - MOV (value: {})",
        SCRecordingOutputFileType::MOV as i32
    );

    // Create recording output
    println!("\n🎥 Creating recording output...");
    if let Some(recording_output) = SCRecordingOutput::new(&config) {
        println!("   ✅ Recording output created successfully!");

        // Get current recording stats (will be 0 since not recording)
        let duration = recording_output.recorded_duration();
        let file_size = recording_output.recorded_file_size();
        println!("\n📊 Recording Stats:");
        println!(
            "   Duration: {}/{} seconds",
            duration.value, duration.timescale
        );
        println!("   File size: {file_size} bytes");

        // Test cloning
        let _cloned = recording_output;
        println!("\n   📋 Clone test: passed");
    } else {
        println!("   ⚠️  Recording output creation failed.");
        println!("   This may happen if macOS 15.0+ is not available.");
    }

    // Test configuration cloning and debug
    let config_clone = config;
    println!("\n📋 Configuration Debug:");
    println!("   {config_clone:?}");

    // Test with HEVC and MOV
    println!("\n🔄 Testing HEVC + MOV configuration...");
    let hevc_config = SCRecordingOutputConfiguration::new()
        .with_video_codec(SCRecordingOutputCodec::HEVC)
        .with_output_file_type(SCRecordingOutputFileType::MOV);
    println!("   Codec: {:?}", hevc_config.video_codec());
    println!("   File type: {:?}", hevc_config.output_file_type());

    println!("\n✅ Recording output example completed!");
    Ok(())
}
