//! FFmpeg Real-Time Encoding
//!
//! Demonstrates real-time video encoding using FFmpeg.
//! This example shows:
//! - Capturing frames at 30 FPS
//! - Piping raw frames to FFmpeg for H.264/HEVC encoding
//! - Zero-copy frame access via IOSurface
//!
//! Requirements:
//! - FFmpeg installed: `brew install ffmpeg`
//!
//! Usage:
//! ```bash
//! cargo run --example 19_ffmpeg_encoding
//! # Creates output.mp4 in current directory
//! ```

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct FFmpegEncoder {
    process: Mutex<Option<Child>>,
    frame_count: AtomicUsize,
    running: AtomicBool,
}

impl FFmpegEncoder {
    fn new(width: u32, height: u32, output_path: &str) -> Result<Self, std::io::Error> {
        // Start FFmpeg process
        // Input: raw BGRA frames from stdin
        // Output: H.264 encoded MP4
        let process = Command::new("ffmpeg")
            .args([
                "-y", // Overwrite output
                "-f",
                "rawvideo", // Input format
                "-pixel_format",
                "bgra", // BGRA pixel format
                "-video_size",
                &format!("{width}x{height}"),
                "-framerate",
                "30", // Input framerate
                "-i",
                "-", // Read from stdin
                "-c:v",
                "libx264", // H.264 codec
                "-preset",
                "ultrafast", // Fast encoding
                "-tune",
                "zerolatency", // Low latency
                "-pix_fmt",
                "yuv420p", // Output pixel format
                "-crf",
                "23", // Quality (lower = better)
                output_path,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(Self {
            process: Mutex::new(Some(process)),
            frame_count: AtomicUsize::new(0),
            running: AtomicBool::new(true),
        })
    }

    fn write_frame(&self, data: &[u8]) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        let mut guard = self.process.lock().unwrap();
        if let Some(ref mut process) = *guard {
            if let Some(ref mut stdin) = process.stdin {
                if stdin.write_all(data).is_ok() {
                    self.frame_count.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
            }
        }
        false
    }

    fn finish(&self) -> Result<(), std::io::Error> {
        self.running.store(false, Ordering::Relaxed);
        let mut guard = self.process.lock().unwrap();
        if let Some(mut process) = guard.take() {
            // Close stdin to signal EOF
            drop(process.stdin.take());
            // Wait for FFmpeg to finish
            process.wait()?;
        }
        Ok(())
    }
}

struct EncodingHandler {
    encoder: Arc<FFmpegEncoder>,
    expected_size: usize,
}

impl SCStreamOutputTrait for EncodingHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        let Some(pixel_buffer) = sample.image_buffer() else {
            return;
        };

        // Lock pixel buffer for CPU access
        let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) else {
            return;
        };

        let data = guard.as_slice();

        // Verify size matches expected (width * height * 4 bytes per pixel)
        if data.len() >= self.expected_size {
            // Write frame to FFmpeg
            if self.encoder.write_frame(&data[..self.expected_size]) {
                let count = self.encoder.frame_count.load(Ordering::Relaxed);
                if count % 30 == 0 {
                    println!("📹 Encoded {} frames ({:.1}s)", count, count as f64 / 30.0);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 FFmpeg Real-Time Encoding\n");

    // Check if FFmpeg is available
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("❌ FFmpeg not found. Install with: brew install ffmpeg");
        return Ok(());
    }
    println!("✅ FFmpeg found\n");

    let width: u32 = 1280;
    let height: u32 = 720;
    let output_path = "output.mp4";
    let duration_secs = 5;

    println!("Configuration:");
    println!("  Resolution: {width}x{height}");
    println!("  Duration: {duration_secs}s");
    println!("  Output: {output_path}");
    println!("  Codec: H.264 (libx264)\n");

    // Create FFmpeg encoder
    let encoder = Arc::new(FFmpegEncoder::new(width, height, output_path)?);
    println!("Started FFmpeg encoder\n");

    // Set up screen capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    let filter = SCContentFilter::with()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Configure for encoding: BGRA format, 30 FPS
    let frame_interval = CMTime::new(1, 30); // 30 FPS
    let config = SCStreamConfiguration::new()
        .with_width(width)
        .with_height(height)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&frame_interval);

    let expected_size = (width * height * 4) as usize;
    let handler = EncodingHandler {
        encoder: encoder.clone(),
        expected_size,
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...\n");
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(duration_secs));

    stream.stop_capture()?;
    println!("\nStopped capture");

    // Finish encoding
    encoder.finish()?;
    let total_frames = encoder.frame_count.load(Ordering::Relaxed);
    println!("Finished encoding: {} frames", total_frames);

    // Check output file
    if let Ok(metadata) = std::fs::metadata(output_path) {
        let size_mb = metadata.len() as f64 / 1_000_000.0;
        println!("\n✅ Created {output_path} ({size_mb:.2} MB)");
        println!("   Play with: open {output_path}");
    }

    Ok(())
}
