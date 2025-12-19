//! Metal Texture Creation and Rendering
//!
//! Demonstrates creating Metal textures from captured frames for GPU rendering.
//! This example shows:
//! - Creating a Metal device and command queue
//! - Compiling built-in shaders with `SHADER_SOURCE`
//! - Zero-copy texture creation from `IOSurface`
//! - Creating render pipelines for different pixel formats
//! - Setting up uniforms and rendering with textures
//!
//! Note: This example demonstrates the API without creating a window.
//! See `16_full_metal_app` for a complete windowed application.

use screencapturekit::metal::{
    MTLPixelFormat, MetalDevice, MetalRenderPipelineDescriptor, MetalRenderPipelineState, Uniforms,
    SHADER_SOURCE,
};
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Renderer holding Metal state
struct Renderer {
    device: MetalDevice,
    pipeline_textured: MetalRenderPipelineState,
    pipeline_ycbcr: MetalRenderPipelineState,
}

impl Renderer {
    fn new() -> Result<Self, String> {
        let device = MetalDevice::system_default().ok_or("No Metal device")?;

        // Compile built-in shaders
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .map_err(|e| format!("Shader compile error: {e}"))?;

        // Pipeline for BGRA/L10R single-plane formats
        let pipeline_textured = {
            let vert = library
                .get_function("vertex_fullscreen")
                .ok_or("vertex_fullscreen not found")?;
            let frag = library
                .get_function("fragment_textured")
                .ok_or("fragment_textured not found")?;
            let desc = MetalRenderPipelineDescriptor::new();
            desc.set_vertex_function(&vert);
            desc.set_fragment_function(&frag);
            desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);
            device
                .create_render_pipeline_state(&desc)
                .ok_or("Failed to create textured pipeline")?
        };

        // Pipeline for YCbCr biplanar formats (420v/420f)
        let pipeline_ycbcr = {
            let vert = library
                .get_function("vertex_fullscreen")
                .ok_or("vertex_fullscreen not found")?;
            let frag = library
                .get_function("fragment_ycbcr")
                .ok_or("fragment_ycbcr not found")?;
            let desc = MetalRenderPipelineDescriptor::new();
            desc.set_vertex_function(&vert);
            desc.set_fragment_function(&frag);
            desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);
            device
                .create_render_pipeline_state(&desc)
                .ok_or("Failed to create ycbcr pipeline")?
        };

        Ok(Self {
            device,
            pipeline_textured,
            pipeline_ycbcr,
        })
    }
}

struct Handler {
    renderer: Arc<Renderer>,
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        let n = self.count.fetch_add(1, Ordering::Relaxed);

        // Log every 60 frames
        if n % 60 != 0 {
            return;
        }

        let Some(pixel_buffer) = sample.image_buffer() else {
            return;
        };

        let Some(surface) = pixel_buffer.io_surface() else {
            println!("⚠️  Frame {n} - Not IOSurface-backed");
            return;
        };

        // Create Metal textures from IOSurface (zero-copy)
        let Some(textures) = surface.create_metal_textures(&self.renderer.device) else {
            println!("⚠️  Frame {n} - Failed to create textures");
            return;
        };

        println!("\n📹 Frame {n}");
        println!("   Dimensions: {}x{}", textures.width, textures.height);

        // Select appropriate pipeline based on pixel format
        let pipeline = if textures.is_ycbcr() {
            println!("   Format: {} (YCbCr biplanar)", textures.pixel_format);
            println!(
                "   Plane 0 (Y): {}x{}",
                textures.plane0.width(),
                textures.plane0.height()
            );
            if let Some(ref plane1) = textures.plane1 {
                println!("   Plane 1 (CbCr): {}x{}", plane1.width(), plane1.height());
            }
            println!("   Pipeline: fragment_ycbcr");
            &self.renderer.pipeline_ycbcr
        } else {
            println!("   Format: {} (single-plane)", textures.pixel_format);
            println!(
                "   Texture: {}x{}",
                textures.plane0.width(),
                textures.plane0.height()
            );
            println!("   Pipeline: fragment_textured");
            &self.renderer.pipeline_textured
        };

        // Create uniforms for aspect-ratio-preserving rendering
        // In a real app, viewport_size would come from your window/layer size
        let viewport_width = 1920.0;
        let viewport_height = 1080.0;
        let uniforms = Uniforms::from_captured_textures(viewport_width, viewport_height, &textures);

        // Create uniform buffer (in a real app, reuse this buffer)
        if let Some(uniform_buffer) = self.renderer.device.create_buffer_with_data(&uniforms) {
            println!(
                "   Uniforms: viewport={:?}, texture={:?}",
                uniforms.viewport_size, uniforms.texture_size
            );
            println!("   Uniform buffer: {} bytes", uniform_buffer.length());
        }

        // In a real rendering loop, you would:
        // 1. Get a drawable from MetalLayer: layer.next_drawable()
        // 2. Create command buffer: command_queue.command_buffer()
        // 3. Create render pass with drawable texture
        // 4. Create encoder and set pipeline, uniforms, textures
        // 5. Draw triangle strip (4 vertices for fullscreen quad)
        // 6. End encoding, present drawable, commit

        println!("   ✅ Ready to render (pipeline={:p})", pipeline.as_ptr());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Metal Texture Creation and Rendering\n");

    // Create renderer with compiled shaders and pipelines
    let renderer = Arc::new(Renderer::new()?);
    println!("Metal device: {}", renderer.device.name());
    println!("Pipelines created:");
    println!("  - fragment_textured (for BGRA, L10R)");
    println!("  - fragment_ycbcr (for 420v, 420f)");

    // Get display to capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    let filter = SCContentFilter::with()
        .display(&display)
        .exclude_windows(&[])
        .build();

    // Use YCbCr format for efficient capture (common in screen capture)
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::YCbCr_420v);

    let handler = Handler {
        renderer,
        count: Arc::new(AtomicUsize::new(0)),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("\nStarting capture (YCbCr 420v format)...\n");
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;

    println!("\n✅ Done");
    Ok(())
}
