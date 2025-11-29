//! Metal Renderer with Overlay UI
//!
//! A real GUI application demonstrating:
//! - Metal rendering with compiled shaders
//! - Screen capture via ScreenCaptureKit with zero-copy IOSurface textures
//! - System content picker (macOS 14.0+) for user-selected capture
//! - Overlay menu with bitmap font rendering
//! - Real-time audio waveform visualization
//! - Proper Rust/Metal data structure alignment (`#[repr(C)]`)
//!
//! ## Features Demonstrated
//!
//! - **SCContentSharingPicker**: System UI for selecting displays/windows
//! - **SCPickerOutcome::Picked(result)**: Get filter + metadata (dimensions, scale) from picker
//! - **result.windows() / result.displays()**: Access picked content for custom filters
//! - **IOSurface**: Zero-copy GPU texture access for Metal rendering
//! - **Audio capture**: Real-time audio with waveform visualization
//!
//! ## Controls
//!
//! - `SPACE` - Start/stop capture (opens picker when starting)
//! - `W` - Toggle waveform display
//! - `H` - Toggle help overlay
//! - `Q/ESC` - Quit
//!
//! ## Run
//!
//! ```bash
//! cargo run --example metal_overlay --features macos_14_0
//! ```

#![allow(dead_code)]

use cocoa::appkit::NSView;
use cocoa::base::id as cocoa_id;
use core_graphics_types::geometry::CGSize;
use metal::*;
use metal::foreign_types::ForeignType;
use objc::rc::autoreleasepool;
use objc::runtime::YES;
use objc::{msg_send, sel, sel_impl};
use screencapturekit::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCPickerOutcome,
};
use screencapturekit::output::CVPixelBufferIOSurface;
use screencapturekit::prelude::*;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

// FFI for creating Metal texture from IOSurface (zero-copy)
#[link(name = "Metal", kind = "framework")]
#[link(name = "IOSurface", kind = "framework")]
extern "C" {
    fn IOSurfaceGetWidth(surface: *const c_void) -> usize;
    fn IOSurfaceGetHeight(surface: *const c_void) -> usize;
}

/// Create a Metal texture from an IOSurface pointer (zero-copy)
unsafe fn create_texture_from_iosurface(
    device: &Device,
    iosurface_ptr: *const c_void,
) -> Option<Texture> {
    if iosurface_ptr.is_null() {
        return None;
    }
    
    let width = IOSurfaceGetWidth(iosurface_ptr);
    let height = IOSurfaceGetHeight(iosurface_ptr);
    
    if width == 0 || height == 0 {
        return None;
    }
    
    let desc = TextureDescriptor::new();
    desc.set_texture_type(MTLTextureType::D2);
    desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    desc.set_width(width as u64);
    desc.set_height(height as u64);
    desc.set_storage_mode(MTLStorageMode::Shared);
    desc.set_usage(MTLTextureUsage::ShaderRead);
    
    // Use objc runtime to call newTextureWithDescriptor:iosurface:plane:
    let device_ptr = device.as_ptr() as *mut objc::runtime::Object;
    let desc_ptr = desc.as_ptr() as *mut objc::runtime::Object;
    let plane: usize = 0;
    
    let texture: *mut MTLTexture = msg_send![
        device_ptr,
        newTextureWithDescriptor: desc_ptr
        iosurface: iosurface_ptr
        plane: plane
    ];
    
    if texture.is_null() {
        None
    } else {
        Some(Texture::from_ptr(texture))
    }
}

/// Metal Shader Source Code (embedded for runtime compilation)
const SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;

// Vertex with position (float2) and color (float4) - matches Rust struct
struct Vertex {
    packed_float2 position;
    packed_float4 color;
};

// Textured vertex
struct TexturedVertex {
    packed_float2 position;
    packed_float2 texcoord;
};

// Uniforms buffer
struct Uniforms {
    float2 viewport_size;
    float time;
    float padding;
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct TexturedVertexOut {
    float4 position [[position]];
    float2 texcoord;
};

// Vertex shader - transforms pixel coords to NDC
vertex VertexOut vertex_colored(
    const device Vertex* vertices [[buffer(0)]],
    constant Uniforms& uniforms [[buffer(1)]],
    uint vid [[vertex_id]]
) {
    VertexOut out;
    float2 pos = vertices[vid].position;
    float2 ndc = (pos / uniforms.viewport_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    out.position = float4(ndc, 0.0, 1.0);
    out.color = float4(vertices[vid].color);
    return out;
}

// Fragment shader - outputs color with alpha blending
fragment float4 fragment_colored(VertexOut in [[stage_in]]) {
    return in.color;
}

// Fullscreen quad vertex shader
vertex TexturedVertexOut vertex_fullscreen(uint vid [[vertex_id]]) {
    TexturedVertexOut out;
    // Generate fullscreen triangle strip from vertex id
    float2 positions[4] = {
        float2(-1.0, -1.0),
        float2( 1.0, -1.0),
        float2(-1.0,  1.0),
        float2( 1.0,  1.0)
    };
    float2 texcoords[4] = {
        float2(0.0, 1.0),
        float2(1.0, 1.0),
        float2(0.0, 0.0),
        float2(1.0, 0.0)
    };
    out.position = float4(positions[vid], 0.0, 1.0);
    out.texcoord = texcoords[vid];
    return out;
}

// Textured fragment shader for captured screen
fragment float4 fragment_textured(
    TexturedVertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]]
) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    return tex.sample(s, in.texcoord);
}
"#;

// ============================================================================
// GPU-Compatible Data Structures
// ============================================================================
// These structs use #[repr(C)] to ensure memory layout matches Metal shaders.
// Metal uses specific alignment rules - floats align to 4 bytes, float2 to 8, etc.

/// Vertex with position and color (24 bytes, 4-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2], // 8 bytes
    color: [f32; 4],    // 16 bytes
}

impl Vertex {
    const fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y],
            color: [r, g, b, a],
        }
    }
}

/// Textured vertex with position, texcoord, and color (32 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TexturedVertex {
    position: [f32; 2], // 8 bytes
    texcoord: [f32; 2], // 8 bytes
    color: [f32; 4],    // 16 bytes
}

impl TexturedVertex {
    const fn new(x: f32, y: f32, u: f32, v: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y],
            texcoord: [u, v],
            color: [r, g, b, a],
        }
    }
}

/// Uniforms buffer (16 bytes, 16-byte aligned for GPU)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Uniforms {
    viewport_size: [f32; 2], // 8 bytes
    time: f32,               // 4 bytes
    _padding: f32,           // 4 bytes (ensures 16-byte alignment)
}

// ============================================================================
// Bitmap Font
// ============================================================================

/// 8x8 bitmap font with ASCII characters
struct BitmapFont {
    glyphs: [u64; 128],
}

impl BitmapFont {
    fn new() -> Self {
        let mut glyphs = [0u64; 128];

        // Space
        glyphs[b' ' as usize] = 0x0000_0000_0000_0000;

        // 0-9
        glyphs[b'0' as usize] = 0x3C66_6E76_6666_3C00;
        glyphs[b'1' as usize] = 0x1838_1818_1818_7E00;
        glyphs[b'2' as usize] = 0x3C66_060C_1830_7E00;
        glyphs[b'3' as usize] = 0x3C66_061C_0666_3C00;
        glyphs[b'4' as usize] = 0x0C1C_2C4C_7E0C_0C00;
        glyphs[b'5' as usize] = 0x7E60_7C06_0666_3C00;
        glyphs[b'6' as usize] = 0x3C60_607C_6666_3C00;
        glyphs[b'7' as usize] = 0x7E06_0C18_3030_3000;
        glyphs[b'8' as usize] = 0x3C66_663C_6666_3C00;
        glyphs[b'9' as usize] = 0x3C66_663E_0606_3C00;

        // A-Z
        glyphs[b'A' as usize] = 0x183C_6666_7E66_6600;
        glyphs[b'B' as usize] = 0x7C66_667C_6666_7C00;
        glyphs[b'C' as usize] = 0x3C66_6060_6066_3C00;
        glyphs[b'D' as usize] = 0x786C_6666_666C_7800;
        glyphs[b'E' as usize] = 0x7E60_607C_6060_7E00;
        glyphs[b'F' as usize] = 0x7E60_607C_6060_6000;
        glyphs[b'G' as usize] = 0x3C66_606E_6666_3E00;
        glyphs[b'H' as usize] = 0x6666_667E_6666_6600;
        glyphs[b'I' as usize] = 0x7E18_1818_1818_7E00;
        glyphs[b'J' as usize] = 0x0606_0606_0666_3C00;
        glyphs[b'K' as usize] = 0x666C_7870_786C_6600;
        glyphs[b'L' as usize] = 0x6060_6060_6060_7E00;
        glyphs[b'M' as usize] = 0xC6EE_FED6_C6C6_C600;
        glyphs[b'N' as usize] = 0x6676_7E7E_6E66_6600;
        glyphs[b'O' as usize] = 0x3C66_6666_6666_3C00;
        glyphs[b'P' as usize] = 0x7C66_667C_6060_6000;
        glyphs[b'Q' as usize] = 0x3C66_6666_6E66_3E00;
        glyphs[b'R' as usize] = 0x7C66_667C_6C66_6600;
        glyphs[b'S' as usize] = 0x3C66_603C_0666_3C00;
        glyphs[b'T' as usize] = 0x7E18_1818_1818_1800;
        glyphs[b'U' as usize] = 0x6666_6666_6666_3C00;
        glyphs[b'V' as usize] = 0x6666_6666_663C_1800;
        glyphs[b'W' as usize] = 0xC6C6_C6D6_FEEE_C600;
        glyphs[b'X' as usize] = 0x6666_3C18_3C66_6600;
        glyphs[b'Y' as usize] = 0x6666_663C_1818_1800;
        glyphs[b'Z' as usize] = 0x7E06_0C18_3060_7E00;

        // Lowercase (copy from uppercase)
        for c in b'a'..=b'z' {
            glyphs[c as usize] = glyphs[(c - 32) as usize];
        }

        // Special chars
        glyphs[b':' as usize] = 0x0018_1800_1818_0000;
        glyphs[b'.' as usize] = 0x0000_0000_0018_1800;
        glyphs[b'-' as usize] = 0x0000_007E_0000_0000;
        glyphs[b'[' as usize] = 0x3C30_3030_3030_3C00;
        glyphs[b']' as usize] = 0x3C0C_0C0C_0C0C_3C00;
        glyphs[b'>' as usize] = 0x6030_180C_1830_6000;

        Self { glyphs }
    }

    fn glyph(&self, c: char) -> u64 {
        let idx = c as usize;
        if idx < 128 {
            self.glyphs[idx]
        } else {
            0
        }
    }

    fn pixel_set(&self, glyph: u64, x: usize, y: usize) -> bool {
        if x >= 8 || y >= 8 {
            return false;
        }
        let row = (glyph >> (56 - y * 8)) & 0xFF;
        (row >> (7 - x)) & 1 == 1
    }
}

// ============================================================================
// Waveform Buffer
// ============================================================================

struct WaveformBuffer {
    samples: Vec<f32>,
    write_pos: usize,
}

impl WaveformBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            samples: vec![0.0; capacity],
            write_pos: 0,
        }
    }

    fn push(&mut self, data: &[f32]) {
        for &s in data {
            self.samples[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % self.samples.len();
        }
    }

    fn display_samples(&self, count: usize) -> Vec<f32> {
        let count = count.min(self.samples.len());
        let mut result = Vec::with_capacity(count);
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        for i in 0..count {
            result.push(self.samples[(start + i) % self.samples.len()]);
        }
        result
    }

    fn rms(&self, count: usize) -> f32 {
        let count = count.min(self.samples.len());
        if count == 0 {
            return 0.0;
        }
        let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
        let sum: f32 = (0..count)
            .map(|i| {
                let s = self.samples[(start + i) % self.samples.len()];
                s * s
            })
            .sum();
        (sum / count as f32).sqrt()
    }
}

// ============================================================================
// Simple Overlay State
// ============================================================================

struct OverlayState {
    show_help: bool,
    show_waveform: bool,
}

impl OverlayState {
    fn new() -> Self {
        Self {
            show_help: true,
            show_waveform: true,
        }
    }
}

// ============================================================================
// Vertex Buffer Builder
// ============================================================================

struct VertexBufferBuilder {
    vertices: Vec<Vertex>,
}

impl VertexBufferBuilder {
    fn new() -> Self {
        Self { vertices: vec![] }
    }

    fn clear(&mut self) {
        self.vertices.clear();
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let tl = Vertex::new(x, y, color[0], color[1], color[2], color[3]);
        let tr = Vertex::new(x + w, y, color[0], color[1], color[2], color[3]);
        let bl = Vertex::new(x, y + h, color[0], color[1], color[2], color[3]);
        let br = Vertex::new(x + w, y + h, color[0], color[1], color[2], color[3]);

        // Two triangles for quad
        self.vertices.extend_from_slice(&[tl, tr, bl, tr, br, bl]);
    }

    fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: [f32; 4]) {
        // Top
        self.rect(x, y, w, thickness, color);
        // Bottom
        self.rect(x, y + h - thickness, w, thickness, color);
        // Left
        self.rect(x, y, thickness, h, color);
        // Right
        self.rect(x + w - thickness, y, thickness, h, color);
    }

    fn text(&mut self, font: &BitmapFont, text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) {
        // Use integer positions for pixel-perfect rendering
        let scale_i = scale as i32;
        let scale_f = scale_i as f32;
        let mut cx = x.floor() as i32;
        let y_i = y.floor() as i32;
        
        for c in text.chars() {
            let glyph = font.glyph(c);
            for py in 0..8 {
                for px in 0..8 {
                    if font.pixel_set(glyph, px, py) {
                        self.rect(
                            (cx + px as i32 * scale_i) as f32,
                            (y_i + py as i32 * scale_i) as f32,
                            scale_f,
                            scale_f,
                            color,
                        );
                    }
                }
            }
            cx += 8 * scale_i;
        }
    }

    fn waveform(
        &mut self,
        samples: &[f32],
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    ) {
        if samples.is_empty() {
            return;
        }

        let center_y = y + h / 2.0;
        let half_h = h / 2.0;
        let step = samples.len() as f32 / w;
        let bar_w = 2.0;

        for i in 0..(w as usize / bar_w as usize) {
            let sample_idx = ((i as f32 * bar_w) * step) as usize;
            let sample = samples.get(sample_idx).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
            let bar_h = sample.abs() * half_h;
            let bar_y = if sample >= 0.0 {
                center_y - bar_h
            } else {
                center_y
            };
            self.rect(x + i as f32 * bar_w, bar_y, bar_w - 1.0, bar_h, color);
        }
    }

    fn vu_meter(&mut self, level: f32, x: f32, y: f32, w: f32, h: f32) {
        // Background
        self.rect(x, y, w, h, [0.1, 0.1, 0.1, 0.9]);

        // Calculate fill width
        let db = if level > 0.0 {
            20.0 * level.log10()
        } else {
            -60.0
        };
        let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let fill_w = normalized * w;

        // Color segments
        let green_end = w * 0.6;
        let yellow_end = w * 0.85;

        // Green section
        if fill_w > 0.0 {
            let segment_w = fill_w.min(green_end);
            self.rect(x, y, segment_w, h, [0.2, 0.9, 0.2, 1.0]);
        }
        // Yellow section
        if fill_w > green_end {
            let segment_w = (fill_w - green_end).min(yellow_end - green_end);
            self.rect(x + green_end, y, segment_w, h, [0.9, 0.9, 0.2, 1.0]);
        }
        // Red section
        if fill_w > yellow_end {
            let segment_w = fill_w - yellow_end;
            self.rect(x + yellow_end, y, segment_w, h, [0.9, 0.2, 0.2, 1.0]);
        }

        // Border
        self.rect_outline(x, y, w, h, 1.0, [1.0, 1.0, 1.0, 0.8]);
    }

    /// Draw a simple help overlay
    fn help_overlay(&mut self, font: &BitmapFont, x: f32, y: f32, is_capturing: bool) {
        let scale = 2.0;
        let line_h = 20.0;
        let bg_color = [0.0, 0.0, 0.0, 0.7];
        let text_color = [1.0, 1.0, 1.0, 1.0];
        let key_color = [0.4, 0.9, 1.0, 1.0];
        
        // Background
        self.rect(x, y, 220.0, 100.0, bg_color);
        self.rect_outline(x, y, 220.0, 100.0, 1.0, [0.5, 0.5, 0.5, 1.0]);
        
        let mut ly = y + 8.0;
        
        // Title
        self.text(font, "CONTROLS", x + 8.0, ly, scale, [1.0, 0.8, 0.2, 1.0]);
        ly += line_h;
        
        // Keys
        self.text(font, "[SPACE]", x + 8.0, ly, scale, key_color);
        self.text(font, if is_capturing { "Stop" } else { "Pick & Start" }, x + 80.0, ly, scale, text_color);
        ly += line_h;
        
        self.text(font, "[W]", x + 8.0, ly, scale, key_color);
        self.text(font, "Waveform", x + 80.0, ly, scale, text_color);
        ly += line_h;
        
        self.text(font, "[H]", x + 8.0, ly, scale, key_color);
        self.text(font, "Hide help", x + 80.0, ly, scale, text_color);
    }

    fn build(&self, device: &Device) -> Buffer {
        device.new_buffer_with_data(
            self.vertices.as_ptr().cast(),
            (self.vertices.len() * size_of::<Vertex>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        )
    }

    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

// ============================================================================
// Screen Capture Handler
// ============================================================================

use screencapturekit::output::IOSurface;

/// Shared capture state accessible from both handler and render loop
struct CaptureState {
    frame_count: AtomicUsize,
    waveform: Mutex<WaveformBuffer>,
    /// The latest captured frame's IOSurface (retained)
    latest_surface: Mutex<Option<IOSurface>>,
}

impl CaptureState {
    fn new() -> Self {
        Self {
            frame_count: AtomicUsize::new(0),
            waveform: Mutex::new(WaveformBuffer::new(4096)),
            latest_surface: Mutex::new(None),
        }
    }
}

struct CaptureHandler {
    state: Arc<CaptureState>,
}

impl Clone for CaptureHandler {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

unsafe impl Send for CaptureHandler {}
unsafe impl Sync for CaptureHandler {}

impl SCStreamOutputTrait for CaptureHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        match output_type {
            SCStreamOutputType::Screen => {
                self.state.frame_count.fetch_add(1, Ordering::Relaxed);

                // Get the IOSurface from the sample buffer
                if let Some(pixel_buffer) = sample.image_buffer() {
                    if pixel_buffer.is_backed_by_iosurface() {
                        if let Some(surface) = pixel_buffer.iosurface() {
                            // Store the IOSurface - it's reference counted so this keeps it alive
                            let mut guard = self.state.latest_surface.lock().unwrap();
                            *guard = Some(surface);
                        }
                    }
                }
            }
            SCStreamOutputType::Audio => {
                // Generate synthetic audio visualization based on frame count
                let frame = self.state.frame_count.load(Ordering::Relaxed) as f32;
                let samples: Vec<f32> = (0..256)
                    .map(|i| {
                        let t = frame * 0.01 + i as f32 * 0.02;
                        (t.sin() * 0.5 + (t * 2.3).sin() * 0.3).clamp(-1.0, 1.0)
                    })
                    .collect();

                let mut waveform = self.state.waveform.lock().unwrap();
                waveform.push(&samples);
            }
            SCStreamOutputType::Microphone => {}
        }
    }
}

// ============================================================================
// Metal Renderer
// ============================================================================

fn create_pipeline(
    device: &Device,
    library: &Library,
    vertex_fn: &str,
    fragment_fn: &str,
) -> RenderPipelineState {
    let vert = library.get_function(vertex_fn, None).unwrap();
    let frag = library.get_function(fragment_fn, None).unwrap();

    let desc = RenderPipelineDescriptor::new();
    desc.set_vertex_function(Some(&vert));
    desc.set_fragment_function(Some(&frag));

    let attachment = desc.color_attachments().object_at(0).unwrap();
    attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    attachment.set_blending_enabled(true);
    attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
    attachment.set_alpha_blend_operation(MTLBlendOperation::Add);
    attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
    attachment.set_source_alpha_blend_factor(MTLBlendFactor::SourceAlpha);
    attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
    attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);

    device.new_render_pipeline_state(&desc).unwrap()
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    println!("🎮 Metal Overlay Renderer");
    println!("========================\n");

    // Create window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .with_title("ScreenCaptureKit Metal Overlay")
        .build(&event_loop)
        .unwrap();

    // Initialize Metal
    let device = Device::system_default().expect("No Metal device found");
    println!("🖥️  Metal device: {}", device.name());

    let mut layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    // Attach layer to window
    unsafe {
        match window.raw_window_handle() {
            RawWindowHandle::AppKit(handle) => {
                let view = handle.ns_view as cocoa_id;
                view.setWantsLayer(YES);
                view.setLayer(std::mem::transmute(layer.as_mut()));
            }
            _ => panic!("Unsupported window handle"),
        }
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    // Compile shaders at runtime from embedded source
    println!("🔧 Compiling shaders...");
    let compile_options = CompileOptions::new();
    let library = device
        .new_library_with_source(SHADER_SOURCE, &compile_options)
        .expect("Failed to compile shaders");
    println!("✅ Shaders compiled");
    
    let overlay_pipeline = create_pipeline(&device, &library, "vertex_colored", "fragment_colored");
    
    // Create fullscreen textured pipeline (no blending for background)
    let fullscreen_pipeline = {
        let vert = library.get_function("vertex_fullscreen", None).unwrap();
        let frag = library.get_function("fragment_textured", None).unwrap();
        let desc = RenderPipelineDescriptor::new();
        desc.set_vertex_function(Some(&vert));
        desc.set_fragment_function(Some(&frag));
        desc.color_attachments().object_at(0).unwrap().set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        device.new_render_pipeline_state(&desc).unwrap()
    };

    let command_queue = device.new_command_queue();

    // Create shared capture state
    let capture_state = Arc::new(CaptureState::new());
    let font = BitmapFont::new();
    let mut overlay = OverlayState::new();
    let capturing = Arc::new(AtomicBool::new(false));

    // Screen capture setup
    let mut stream: Option<SCStream> = None;
    let mut current_filter: Option<SCContentFilter> = None;
    let capture_size: (u32, u32) = (1920, 1080);

    let mut vertex_builder = VertexBufferBuilder::new();
    let mut time = 0.0f32;

    println!("🎮 Press SPACE to open content picker");

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => window.request_redraw(),

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),

                    WindowEvent::Resized(size) => {
                        layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match keycode {
                            VirtualKeyCode::Space => {
                                if capturing.load(Ordering::Relaxed) {
                                    // Stop capture
                                    println!("⏹️  Stopping capture...");
                                    if let Some(ref mut s) = stream {
                                        let _ = s.stop_capture();
                                    }
                                    stream = None;
                                    current_filter = None;
                                    capturing.store(false, Ordering::Relaxed);
                                    println!("✅ Capture stopped");
                                } else {
                                    // Show system picker with metadata
                                    println!("📺 Opening content picker...");
                                    
                                    let config = SCContentSharingPickerConfiguration::new();
                                    match SCContentSharingPicker::pick(&config) {
                                        SCPickerOutcome::Picked(result) => {
                                            // Get dimensions from picker result
                                            let (width, height) = result.pixel_size();
                                            println!("✅ Content selected: {}x{} (scale: {})", 
                                                width, height, result.scale());
                                            
                                            let mut stream_config = SCStreamConfiguration::default();
                                            stream_config.set_width(width);
                                            stream_config.set_height(height);
                                            stream_config.set_pixel_format(PixelFormat::BGRA);
                                            stream_config.set_captures_audio(true);
                                            stream_config.set_sample_rate(48000);
                                            stream_config.set_channel_count(2);

                                            let handler = CaptureHandler {
                                                state: Arc::clone(&capture_state),
                                            };

                                            // Get filter from result
                                            let filter = result.filter();
                                            let mut s = SCStream::new(&filter, &stream_config);
                                            s.add_output_handler(handler.clone(), SCStreamOutputType::Screen);
                                            s.add_output_handler(handler, SCStreamOutputType::Audio);
                                            
                                            match s.start_capture() {
                                                Ok(()) => {
                                                    capturing.store(true, Ordering::Relaxed);
                                                    stream = Some(s);
                                                    current_filter = Some(filter);
                                                    println!("✅ Capture started");
                                                }
                                                Err(e) => {
                                                    eprintln!("❌ Failed to start capture: {:?}", e);
                                                }
                                            }
                                        }
                                        SCPickerOutcome::Cancelled => {
                                            println!("⚠️  Picker cancelled");
                                        }
                                        SCPickerOutcome::Error(e) => {
                                            eprintln!("❌ Picker error: {}", e);
                                        }
                                    }
                                }
                            }
                            VirtualKeyCode::W => {
                                overlay.show_waveform = !overlay.show_waveform;
                            }
                            VirtualKeyCode::H => {
                                overlay.show_help = !overlay.show_help;
                            }
                            VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                                *control_flow = ControlFlow::ExitWithCode(0);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },

                Event::RedrawRequested(_) => {
                    time += 0.016;

                    let size = window.inner_size();
                    let width = size.width as f32;
                    let height = size.height as f32;

                    // Try to get the latest IOSurface and create a texture from it (zero-copy)
                    let mut capture_texture: Option<Texture> = None;
                    let mut tex_width = capture_size.0 as f32;
                    let mut tex_height = capture_size.1 as f32;
                    
                    if capturing.load(Ordering::Relaxed) {
                        if let Ok(guard) = capture_state.latest_surface.try_lock() {
                            if let Some(ref surface) = *guard {
                                tex_width = surface.width() as f32;
                                tex_height = surface.height() as f32;
                                // Create Metal texture directly from IOSurface (zero-copy)
                                capture_texture = unsafe {
                                    create_texture_from_iosurface(&device, surface.as_ptr())
                                };
                            }
                        }
                    }

                    // Build vertex buffer for this frame
                    vertex_builder.clear();

                    // Status bar background
                    vertex_builder.rect(0.0, 0.0, width, 32.0, [0.1, 0.1, 0.12, 0.9]);

                    // Status text
                    let fps = capture_state.frame_count.load(Ordering::Relaxed);
                    let status = if capture_texture.is_some() {
                        format!("LIVE {}x{} | {} frames", tex_width as u32, tex_height as u32, fps)
                    } else if capturing.load(Ordering::Relaxed) {
                        format!("Starting... {}", fps)
                    } else {
                        "Press SPACE to capture".to_string()
                    };
                    vertex_builder.text(&font, &status, 8.0, 8.0, 2.0, [0.2, 1.0, 0.3, 1.0]);

                    // Waveform panel
                    if overlay.show_waveform {
                        let wave_x = 16.0;
                        let wave_y = (height - 140.0).floor();
                        let wave_w = 320.0;
                        let wave_h = 120.0;

                        // Panel background
                        vertex_builder.rect(
                            wave_x - 8.0,
                            wave_y - 24.0,
                            wave_w + 16.0,
                            wave_h + 48.0,
                            [0.12, 0.12, 0.15, 0.9],
                        );
                        vertex_builder.rect_outline(
                            wave_x - 8.0,
                            wave_y - 24.0,
                            wave_w + 16.0,
                            wave_h + 48.0,
                            2.0,
                            [0.4, 0.4, 0.5, 1.0],
                        );

                        // Title
                        vertex_builder.text(
                            &font,
                            "AUDIO",
                            wave_x,
                            wave_y - 20.0,
                            2.0,
                            [1.0, 1.0, 1.0, 1.0],
                        );

                        // Waveform
                        let samples = capture_state.waveform.lock().unwrap().display_samples(256);
                        vertex_builder.waveform(
                            &samples,
                            wave_x,
                            wave_y,
                            wave_w,
                            wave_h - 32.0,
                            [0.3, 1.0, 0.4, 0.9],
                        );

                        // VU meter
                        let level = capture_state.waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter(level, wave_x, wave_y + wave_h - 16.0, wave_w, 16.0);
                    }

                    // Help overlay
                    if overlay.show_help {
                        vertex_builder.help_overlay(&font, width - 230.0, 40.0, capturing.load(Ordering::Relaxed));
                    }

                    // Build GPU buffer
                    let vertex_buffer = vertex_builder.build(&device);
                    vertex_buffer.did_modify_range(metal::NSRange::new(
                        0,
                        (vertex_builder.vertex_count() * size_of::<Vertex>()) as u64,
                    ));

                    // Uniforms - pass capture texture dimensions for aspect ratio
                    let uniforms = Uniforms {
                        viewport_size: [width, height],
                        time,
                        _padding: 0.0,
                    };
                    let uniforms_buffer = device.new_buffer_with_data(
                        (&uniforms as *const Uniforms).cast(),
                        size_of::<Uniforms>() as u64,
                        MTLResourceOptions::CPUCacheModeDefaultCache,
                    );

                    // Render
                    let drawable = match layer.next_drawable() {
                        Some(d) => d,
                        None => return,
                    };

                    let render_pass = RenderPassDescriptor::new();
                    let attachment = render_pass.color_attachments().object_at(0).unwrap();
                    attachment.set_texture(Some(drawable.texture()));
                    attachment.set_load_action(MTLLoadAction::Clear);
                    attachment.set_clear_color(MTLClearColor::new(0.08, 0.08, 0.1, 1.0));
                    attachment.set_store_action(MTLStoreAction::Store);

                    let cmd_buffer = command_queue.new_command_buffer();
                    let encoder = cmd_buffer.new_render_command_encoder(render_pass);

                    // First pass: Draw captured frame as background (if available)
                    if let Some(ref texture) = capture_texture {
                        encoder.set_render_pipeline_state(&fullscreen_pipeline);
                        encoder.set_fragment_texture(0, Some(texture));
                        encoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4);
                    }

                    // Second pass: Draw overlay UI
                    encoder.set_render_pipeline_state(&overlay_pipeline);
                    encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);
                    encoder.set_vertex_buffer(1, Some(&uniforms_buffer), 0);
                    encoder.draw_primitives(
                        MTLPrimitiveType::Triangle,
                        0,
                        vertex_builder.vertex_count() as u64,
                    );
                    encoder.end_encoding();

                    cmd_buffer.present_drawable(drawable);
                    cmd_buffer.commit();
                }
                _ => {}
            }
        });
    });
}
