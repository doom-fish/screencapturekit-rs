//! Screen Capture Server
//!
//! Captures the screen and streams frames over TCP.
//! Run with: `cargo run --example 23_client_server_server`

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Shared frame buffer
struct SharedFrame {
    data: Vec<u8>, // RGBA pixels
    dirty: AtomicBool,
}

impl SharedFrame {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            dirty: AtomicBool::new(false),
        }
    }
}

struct FrameHandler {
    frame: Arc<Mutex<SharedFrame>>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        let Some(pixel_buffer) = sample.image_buffer() else {
            return;
        };

        let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) else {
            return;
        };

        let width = pixel_buffer.width();
        let height = pixel_buffer.height();
        let data = guard.as_slice();

        // Convert BGRA to RGBA
        let mut rgba = vec![0u8; width * height * 4];
        for i in 0..(width * height) {
            let idx = i * 4;
            if idx + 3 < data.len() {
                rgba[idx] = data[idx + 2]; // R
                rgba[idx + 1] = data[idx + 1]; // G
                rgba[idx + 2] = data[idx]; // B
                rgba[idx + 3] = data[idx + 3]; // A
            }
        }

        if let Ok(mut frame) = self.frame.lock() {
            frame.data = rgba;
            frame.dirty.store(true, Ordering::Release);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Screen Capture Server\n");

    // Set up capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!(
        "Capturing display: {}x{}",
        display.width(),
        display.height()
    );

    let filter = SCContentFilter::with()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&CMTime::new(1, 30));

    let shared_frame = Arc::new(Mutex::new(SharedFrame::new()));
    let handler = FrameHandler {
        frame: shared_frame.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:9999")?;
    println!("📡 Listening on 127.0.0.1:9999");
    println!("Run the client to connect.\n");

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut client) => {
                println!("✅ Client connected: {:?}", client.peer_addr());
                let frame_ref = shared_frame.clone();

                std::thread::spawn(move || {
                    loop {
                        // Wait for dirty frame
                        std::thread::sleep(std::time::Duration::from_millis(33));

                        let frame_data = {
                            if let Ok(frame) = frame_ref.lock() {
                                if frame.dirty.swap(false, Ordering::AcqRel) {
                                    Some(frame.data.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(data) = frame_data {
                            // Send raw RGBA frame
                            if client.write_all(&data).is_err() {
                                println!("Client disconnected");
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {e}"),
        }
    }

    Ok(())
}
