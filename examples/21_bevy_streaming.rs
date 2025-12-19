//! Bevy Texture Streaming
//!
//! Demonstrates real-time screen capture as a Bevy texture.
//! This example shows:
//! - Real-time screen capture to Bevy sprite
//! - Streaming frames to Bevy's render pipeline
//! - BGRA to RGBA conversion for Bevy compatibility
//!
//! Usage:
//! ```bash
//! cargo run --example 21_bevy_streaming
//! ```

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Frame data for Bevy texture updates
struct FrameData {
    /// RGBA pixel data (converted from BGRA)
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

struct SharedState {
    frame: Mutex<Option<FrameData>>,
    new_frame: AtomicBool,
    frame_count: AtomicUsize,
}

struct StreamHandler {
    state: Arc<SharedState>,
}

impl SCStreamOutputTrait for StreamHandler {
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

        self.state.frame_count.fetch_add(1, Ordering::Relaxed);

        // Update shared frame
        if let Ok(mut frame) = self.state.frame.lock() {
            *frame = Some(FrameData {
                rgba,
                width,
                height,
            });
            self.state.new_frame.store(true, Ordering::Release);
        }
    }
}

/// Resource holding the screen capture state
#[derive(Resource)]
struct ScreenCapture {
    state: Arc<SharedState>,
    #[allow(dead_code)]
    stream: SCStream,
}

/// Marker component for the screen sprite
#[derive(Component)]
struct ScreenSprite;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Screen Capture Viewer".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_screen_texture)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    println!("🎮 Setting up Bevy Screen Capture...\n");

    // Set up screen capture
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content
        .displays()
        .into_iter()
        .next()
        .expect("No displays found");

    println!("Display: {}x{}", display.width(), display.height());

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Capture at 30 FPS
    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&CMTime::new(1, 30));

    let state = Arc::new(SharedState {
        frame: Mutex::new(None),
        new_frame: AtomicBool::new(false),
        frame_count: AtomicUsize::new(0),
    });

    let handler = StreamHandler {
        state: state.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...\n");
    stream.start_capture().expect("Failed to start capture");

    // Create a placeholder texture
    let placeholder = Image::new_fill(
        Extent3d {
            width: 1280,
            height: 720,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[30, 30, 30, 255], // Dark gray
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    let texture_handle = images.add(placeholder);

    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn sprite with the texture
    commands.spawn((
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        ScreenSprite,
    ));

    // Insert screen capture resource
    commands.insert_resource(ScreenCapture { state, stream });
}

fn update_screen_texture(
    capture: Res<ScreenCapture>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<&mut Sprite, With<ScreenSprite>>,
) {
    // Check for new frame
    if !capture.state.new_frame.swap(false, Ordering::Acquire) {
        return;
    }

    let Ok(frame_guard) = capture.state.frame.lock() else {
        return;
    };

    let Some(ref frame) = *frame_guard else {
        return;
    };

    // Create Bevy Image from RGBA data
    let image = Image::new(
        Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        frame.rgba.clone(),
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    // Update the texture handle
    for mut sprite in &mut query {
        sprite.image = images.add(image.clone());
    }

    let count = capture.state.frame_count.load(Ordering::Relaxed);
    if count % 30 == 0 {
        println!("📹 Frame {}: {}x{}", count, frame.width, frame.height);
    }
}
