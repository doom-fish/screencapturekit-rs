//! egui Screen Viewer
//!
//! A full working egui application that displays real-time screen capture.
//! This example shows:
//! - Capturing frames as textures
//! - Displaying in an egui window
//! - Smooth 60 FPS rendering
//!
//! Run with: `cargo run --example 20_egui_viewer`

use eframe::egui;
use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Shared frame buffer between capture thread and UI
struct SharedFrame {
    data: Vec<u8>,
    width: usize,
    height: usize,
    frame_count: AtomicUsize,
    dirty: AtomicBool,
}

impl SharedFrame {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
            frame_count: AtomicUsize::new(0),
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
            frame.frame_count.fetch_add(1, Ordering::Relaxed);
            frame.dirty.store(true, Ordering::Release);
        }
    }
}

/// egui application state
struct ScreenViewerApp {
    shared_frame: Arc<Mutex<SharedFrame>>,
    texture: Option<egui::TextureHandle>,
    stream: Option<SCStream>,
    frame_count: usize,
    last_width: usize,
    last_height: usize,
}

impl ScreenViewerApp {
    fn new(shared_frame: Arc<Mutex<SharedFrame>>, stream: SCStream) -> Self {
        Self {
            shared_frame,
            texture: None,
            stream: Some(stream),
            frame_count: 0,
            last_width: 0,
            last_height: 0,
        }
    }
}

impl eframe::App for ScreenViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new frame
        let should_update = if let Ok(frame) = self.shared_frame.lock() {
            frame.dirty.load(Ordering::Acquire)
        } else {
            false
        };

        if should_update {
            if let Ok(frame) = self.shared_frame.lock() {
                if frame.dirty.swap(false, Ordering::AcqRel) && !frame.data.is_empty() {
                    self.frame_count = frame.frame_count.load(Ordering::Relaxed);
                    self.last_width = frame.width;
                    self.last_height = frame.height;

                    // Create egui image from RGBA data
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width, frame.height],
                        &frame.data,
                    );

                    // Update or create texture
                    self.texture = Some(ctx.load_texture(
                        "screen_capture",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
        }

        // Top panel with stats
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🖥️ Screen Capture Viewer");
                ui.separator();
                ui.label(format!(
                    "Frame: {} | Size: {}x{}",
                    self.frame_count, self.last_width, self.last_height
                ));
            });
        });

        // Central panel with the captured screen
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref texture) = self.texture {
                // Calculate size to fit in available space while maintaining aspect ratio
                let available = ui.available_size();
                let tex_size = texture.size_vec2();
                let scale = (available.x / tex_size.x)
                    .min(available.y / tex_size.y)
                    .min(1.0);
                let display_size = tex_size * scale;

                // Center the image
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), display_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for first frame...");
                });
            }
        });

        // Request continuous repaint for smooth updates
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Stop capture when app exits
        if let Some(ref mut stream) = self.stream {
            let _ = stream.stop_capture();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  egui Screen Viewer\n");

    // Set up screen capture
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

    let filter = SCContentFilter::new()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Capture at reasonable size for UI display
    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&CMTime::new(1, 30)); // 30 FPS capture

    let shared_frame = Arc::new(Mutex::new(SharedFrame::new()));
    let handler = FrameHandler {
        frame: shared_frame.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...");
    stream.start_capture()?;

    // Run egui app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 800.0])
            .with_title("Screen Capture Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "Screen Capture Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(ScreenViewerApp::new(shared_frame, stream)))),
    )?;

    Ok(())
}
