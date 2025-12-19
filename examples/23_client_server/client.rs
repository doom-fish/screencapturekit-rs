//! Screen Capture Client
//!
//! Connects to the server and displays received frames using egui.
//! Run with: `cargo run --example 23_client_server_client`

use eframe::egui;
use std::io::Read;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

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

struct ViewerApp {
    shared_frame: Arc<Mutex<SharedFrame>>,
    texture: Option<egui::TextureHandle>,
    connected: Arc<AtomicBool>,
}

impl ViewerApp {
    fn new(shared_frame: Arc<Mutex<SharedFrame>>, connected: Arc<AtomicBool>) -> Self {
        Self {
            shared_frame,
            texture: None,
            connected,
        }
    }
}

impl eframe::App for ViewerApp {
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
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [WIDTH as usize, HEIGHT as usize],
                        &frame.data,
                    );
                    self.texture = Some(ctx.load_texture(
                        "remote_screen",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📺 Remote Screen Viewer");
                ui.separator();
                if self.connected.load(Ordering::Relaxed) {
                    ui.label("🟢 Connected");
                } else {
                    ui.label("🔴 Disconnected");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref texture) = self.texture {
                let available = ui.available_size();
                let tex_size = texture.size_vec2();
                let scale = (available.x / tex_size.x)
                    .min(available.y / tex_size.y)
                    .min(1.0);
                let display_size = tex_size * scale;

                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), display_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for frames...");
                });
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📺 Screen Capture Client\n");

    let shared_frame = Arc::new(Mutex::new(SharedFrame::new()));
    let connected = Arc::new(AtomicBool::new(false));

    // Network thread
    let frame_ref = shared_frame.clone();
    let connected_ref = connected.clone();

    std::thread::spawn(move || {
        loop {
            println!("Connecting to 127.0.0.1:9999...");
            match TcpStream::connect("127.0.0.1:9999") {
                Ok(mut stream) => {
                    println!("✅ Connected to server");
                    connected_ref.store(true, Ordering::Relaxed);

                    let frame_size = (WIDTH * HEIGHT * 4) as usize;
                    loop {
                        // Read raw RGBA frame
                        let mut rgba_data = vec![0u8; frame_size];
                        if stream.read_exact(&mut rgba_data).is_err() {
                            break;
                        }

                        if let Ok(mut frame) = frame_ref.lock() {
                            frame.data = rgba_data;
                            frame.dirty.store(true, Ordering::Release);
                        }
                    }

                    connected_ref.store(false, Ordering::Relaxed);
                    println!("Disconnected from server");
                }
                Err(e) => {
                    eprintln!("Connection failed: {e}");
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // Run GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 800.0])
            .with_title("Remote Screen Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "Remote Screen Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(ViewerApp::new(shared_frame, connected)))),
    )?;

    Ok(())
}
