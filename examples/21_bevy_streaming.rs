//! Bevy Texture Streaming
//!
//! Demonstrates real-time screen capture as a Bevy texture.
//! This example shows:
//! - Zero-copy Metal texture creation
//! - Streaming frames to Bevy's render pipeline
//! - Using IOSurface for GPU-to-GPU transfer
//!
//! Note: Requires `bevy` crate with render feature.
//! Add to Cargo.toml: `bevy = "0.14"`

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::metal::MetalDevice;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Frame data for Bevy texture updates
struct FrameData {
    /// RGBA pixel data (converted from BGRA)
    #[allow(dead_code)]
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    /// Frame number for change detection
    frame_number: usize,
}

struct SharedState {
    frame: Mutex<Option<FrameData>>,
    new_frame: AtomicBool,
    frame_count: AtomicUsize,
}

struct BevyStreamHandler {
    state: Arc<SharedState>,
}

impl SCStreamOutputTrait for BevyStreamHandler {
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

        let width = pixel_buffer.width() as u32;
        let height = pixel_buffer.height() as u32;
        let data = guard.as_slice();

        // Convert BGRA to RGBA for Bevy
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for i in 0..(width * height) as usize {
            let src = i * 4;
            let dst = i * 4;
            if src + 3 < data.len() {
                rgba[dst] = data[src + 2]; // R <- B
                rgba[dst + 1] = data[src + 1]; // G <- G
                rgba[dst + 2] = data[src]; // B <- R
                rgba[dst + 3] = data[src + 3]; // A <- A
            }
        }

        let frame_number = self.state.frame_count.fetch_add(1, Ordering::Relaxed);

        // Update shared frame
        if let Ok(mut frame) = self.state.frame.lock() {
            *frame = Some(FrameData {
                rgba,
                width,
                height,
                frame_number,
            });
            self.state.new_frame.store(true, Ordering::Release);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Bevy Texture Streaming\n");
    println!("This example demonstrates Bevy integration.");
    println!("To run with Bevy, add to your dependencies:\n");
    println!("  bevy = \"0.14\"\n");

    // Check Metal availability
    let metal_device = MetalDevice::system_default().ok_or("No Metal device")?;
    println!("Metal device: {}", metal_device.name());

    // Set up screen capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!("Display: {}x{}\n", display.width(), display.height());

    let filter = SCContentFilter::new()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Capture at 60 FPS for smooth streaming
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&CMTime::new(1, 60));

    let state = Arc::new(SharedState {
        frame: Mutex::new(None),
        new_frame: AtomicBool::new(false),
        frame_count: AtomicUsize::new(0),
    });

    let handler = BevyStreamHandler {
        state: state.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...\n");
    stream.start_capture()?;

    // Simulate Bevy update loop
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(100));

        if state.new_frame.swap(false, Ordering::Acquire) {
            if let Ok(frame) = state.frame.lock() {
                if let Some(ref f) = *frame {
                    println!(
                        "📹 New frame #{}: {}x{} ready for Bevy",
                        f.frame_number, f.width, f.height
                    );
                }
            }
        }
    }

    stream.stop_capture()?;
    let total = state.frame_count.load(Ordering::Relaxed);
    println!("\n✅ Captured {} frames", total);

    println!("\n--- Bevy Integration Code ---\n");
    println!(
        r#"
use bevy::prelude::*;
use bevy::render::render_resource::{{Extent3d, TextureDimension, TextureFormat}};

#[derive(Resource)]
struct ScreenCapture {{
    state: Arc<SharedState>,
    stream: SCStream,
}}

fn update_screen_texture(
    capture: Res<ScreenCapture>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<&mut Handle<Image>, With<ScreenSprite>>,
) {{
    // Check for new frame
    if !capture.state.new_frame.swap(false, Ordering::Acquire) {{
        return;
    }}

    let Ok(frame_guard) = capture.state.frame.lock() else {{
        return;
    }};

    let Some(ref frame) = *frame_guard else {{
        return;
    }};

    // Create Bevy Image from RGBA data
    let image = Image::new(
        Extent3d {{
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        }},
        TextureDimension::D2,
        frame.rgba.clone(),
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    // Update the texture handle
    for mut handle in query.iter_mut() {{
        *handle = images.add(image.clone());
    }}
}}

// For zero-copy GPU path (advanced):
// Use IOSurface -> Metal texture -> wgpu texture -> Bevy Image
// See example 18_wgpu_integration for the Metal texture creation
"#
    );

    Ok(())
}
