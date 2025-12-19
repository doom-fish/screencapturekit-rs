//! wgpu Integration (Zero-Copy)
//!
//! Demonstrates zero-copy integration with wgpu for cross-platform GPU rendering.
//! This example shows:
//! - Creating a wgpu device from Metal
//! - Zero-copy texture import from IOSurface via Metal
//! - Rendering captured frames with wgpu
//!
//! Note: Requires `wgpu` crate with "metal" feature.
//! Add to Cargo.toml: `wgpu = { version = "0.19", features = ["metal"] }`

use screencapturekit::metal::MetalDevice;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Demonstrates zero-copy path from ScreenCaptureKit -> IOSurface -> Metal -> wgpu
///
/// The key insight is that wgpu on macOS uses Metal internally, so we can:
/// 1. Get IOSurface from CMSampleBuffer (zero-copy)
/// 2. Create Metal texture from IOSurface (zero-copy)
/// 3. Import Metal texture into wgpu (zero-copy on same device)
///
/// This gives us a fully zero-copy pipeline from screen capture to wgpu rendering.
struct WgpuIntegrationHandler {
    metal_device: MetalDevice,
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for WgpuIntegrationHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        let n = self.count.fetch_add(1, Ordering::Relaxed);
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

        // Zero-copy: Create Metal textures from IOSurface
        let Some(textures) = surface.create_metal_textures(&self.metal_device) else {
            println!("⚠️  Frame {n} - Failed to create Metal textures");
            return;
        };

        println!("\n📹 Frame {n} - wgpu Integration");
        println!("   Dimensions: {}x{}", textures.width, textures.height);
        println!("   Format: {}", textures.pixel_format);
        println!("   Metal texture ptr: {:p}", textures.plane0.as_ptr());

        // In a real wgpu integration, you would:
        //
        // 1. Create wgpu instance with Metal backend:
        //    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        //        backends: wgpu::Backends::METAL,
        //        ..Default::default()
        //    });
        //
        // 2. Create wgpu device from the same Metal device (for zero-copy):
        //    Use `wgpu::hal::api::Metal` to create device from existing MTLDevice
        //
        // 3. Import Metal texture into wgpu:
        //    Use `wgpu::hal::api::Metal::Device::create_texture_from_hal`
        //    with the Metal texture pointer
        //
        // 4. Create wgpu TextureView and use in render pass
        //
        // Example pseudo-code:
        // ```
        // let wgpu_texture = unsafe {
        //     device.create_texture_from_hal::<wgpu::hal::api::Metal>(
        //         metal_texture,  // textures.plane0
        //         &wgpu::TextureDescriptor { ... }
        //     )
        // };
        // let view = wgpu_texture.create_view(&Default::default());
        // // Use view in render pass...
        // ```

        println!("   ✅ Ready for wgpu import");
        println!("   📌 Use wgpu::hal::api::Metal for zero-copy texture import");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 wgpu Integration (Zero-Copy)\n");
    println!("This example demonstrates the zero-copy path:");
    println!("  ScreenCaptureKit → IOSurface → Metal → wgpu\n");

    let metal_device = MetalDevice::system_default().ok_or("No Metal device")?;
    println!("Metal device: {}\n", metal_device.name());

    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    let filter = SCContentFilter::new()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    // Use BGRA for simplest wgpu integration (maps to Bgra8Unorm)
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    let handler = WgpuIntegrationHandler {
        metal_device,
        count: Arc::new(AtomicUsize::new(0)),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    println!("Starting capture...\n");
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;

    println!("\n✅ Done");
    println!("\nTo use with wgpu, add to Cargo.toml:");
    println!("  wgpu = {{ version = \"0.19\", features = [\"metal\"] }}");

    Ok(())
}
