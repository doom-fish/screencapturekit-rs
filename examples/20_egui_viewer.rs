//! egui Screen Viewer
//!
//! Demonstrates real-time screen capture display using egui.
//! This example shows:
//! - Capturing frames as textures
//! - Displaying in an egui window
//! - Smooth 60 FPS rendering
//!
//! Note: Requires `eframe` crate.
//! Add to Cargo.toml: `eframe = "0.29"`

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Shared frame buffer between capture thread and UI
struct SharedFrame {
    data: Vec<u8>,
    width: usize,
    height: usize,
    dirty: AtomicBool,
}

impl SharedFrame {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
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

        // Convert BGRA to RGBA for egui
        let mut rgba = vec![0u8; width * height * 4];
        for i in 0..(width * height) {
            let src = i * 4;
            let dst = i * 4;
            if src + 3 < data.len() {
                rgba[dst] = data[src + 2]; // R <- B
                rgba[dst + 1] = data[src + 1]; // G <- G
                rgba[dst + 2] = data[src]; // B <- R
                rgba[dst + 3] = data[src + 3]; // A <- A
            }
        }

        // Update shared frame
        if let Ok(mut frame) = self.frame.lock() {
            frame.data = rgba;
            frame.width = width;
            frame.height = height;
            frame.dirty.store(true, Ordering::Relaxed);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  egui Screen Viewer\n");
    println!("This example demonstrates egui integration.");
    println!("To run with a full GUI, add eframe to your dependencies:\n");
    println!("  eframe = \"0.29\"\n");

    // Set up screen capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!("Display: {}x{}", display.width(), display.height());

    let filter = SCContentFilter::new()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Capture at reasonable size for UI display
    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&CMTime::new(1, 60)); // 60 FPS

    let shared_frame = Arc::new(Mutex::new(SharedFrame::new()));
    let handler = FrameHandler {
        frame: shared_frame.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("\nStarting capture...\n");
    stream.start_capture()?;

    // Simulate frame updates (in real app, egui would poll this)
    for i in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Ok(frame) = shared_frame.lock() {
            if frame.dirty.load(Ordering::Relaxed) {
                println!(
                    "📹 Frame update {}: {}x{} ({} bytes)",
                    i,
                    frame.width,
                    frame.height,
                    frame.data.len()
                );
            }
        }
    }

    stream.stop_capture()?;

    println!("\n✅ Done");
    println!("\n--- egui Integration Code ---\n");
    println!(
        r#"
// In your egui App::update():
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {{
    // Check for new frame
    if let Ok(frame) = self.shared_frame.lock() {{
        if frame.dirty.swap(false, Ordering::Relaxed) {{
            // Create or update texture
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [frame.width, frame.height],
                &frame.data,
            );
            self.texture = Some(ctx.load_texture(
                "screen",
                image,
                egui::TextureOptions::LINEAR,
            ));
        }}
    }}

    // Display the texture
    egui::CentralPanel::default().show(ctx, |ui| {{
        if let Some(ref texture) = self.texture {{
            ui.image(texture);
        }}
    }});

    // Request continuous updates
    ctx.request_repaint();
}}
"#
    );

    Ok(())
}
