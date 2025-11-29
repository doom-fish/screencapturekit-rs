//! Metal Renderer with Overlay UI
//!
//! A real GUI application demonstrating:
//! - Metal rendering with compiled shaders
//! - Screen capture via ScreenCaptureKit with zero-copy IOSurface textures
//! - System content picker (macOS 14.0+) for user-selected capture
//! - Interactive overlay menu with bitmap font rendering
//! - Real-time audio waveform visualization with vertical gain meters
//! - Proper Rust/Metal data structure alignment (`#[repr(C)]`)
//!
//! ## Features Demonstrated
//!
//! - **SCContentSharingPicker**: System UI for selecting displays/windows/apps
//! - **SCPickerResult**: Get filter + metadata (dimensions, scale) from picker
//! - **IOSurface**: Zero-copy GPU texture access for Metal rendering
//! - **Audio capture**: Real-time system audio + microphone with waveform visualization
//!
//! ## Controls
//!
//! Menu navigation (when menu visible):
//! - `UP/DOWN` - Navigate menu items
//! - `SPACE/ENTER` - Select item
//! - `ESC/H` - Hide menu
//!
//! Direct controls (when menu hidden):
//! - `P` - Open content picker
//! - `SPACE` - Start/stop capture
//! - `W` - Toggle waveform display
//! - `C` - Open config menu
//! - `M` - Toggle microphone
//! - `H` - Show menu
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
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerMode, 
    SCPickedSource, SCPickerOutcome,
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
    float2 texture_size;
    float time;
    float padding[3];
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

// Fullscreen quad vertex shader with aspect ratio preservation
vertex TexturedVertexOut vertex_fullscreen(
    uint vid [[vertex_id]],
    constant Uniforms& uniforms [[buffer(0)]]
) {
    TexturedVertexOut out;
    
    // Calculate aspect ratios
    float viewportAspect = uniforms.viewport_size.x / uniforms.viewport_size.y;
    float textureAspect = uniforms.texture_size.x / uniforms.texture_size.y;
    
    // Scale to fit while preserving aspect ratio (letterbox/pillarbox)
    float scaleX = 1.0;
    float scaleY = 1.0;
    
    if (textureAspect > viewportAspect) {
        // Texture is wider - pillarbox (black bars top/bottom)
        scaleY = viewportAspect / textureAspect;
    } else {
        // Texture is taller - letterbox (black bars left/right)
        scaleX = textureAspect / viewportAspect;
    }
    
    // Generate quad vertices centered in viewport
    float2 positions[4] = {
        float2(-scaleX, -scaleY),
        float2( scaleX, -scaleY),
        float2(-scaleX,  scaleY),
        float2( scaleX,  scaleY)
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

/// Uniforms buffer (32 bytes, 16-byte aligned for GPU)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Uniforms {
    viewport_size: [f32; 2], // 8 bytes
    texture_size: [f32; 2],  // 8 bytes - for aspect ratio calculation
    time: f32,               // 4 bytes
    _padding: [f32; 3],      // 12 bytes (ensures 32-byte alignment)
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
    show_config: bool,
    config_selection: usize,
    menu_selection: usize,
}

/// Stream configuration options that can be changed at runtime
#[derive(Clone)]
struct StreamConfig {
    fps: u32,
    show_cursor: bool,
    captures_audio: bool,
    captures_mic: bool,
    mic_device_idx: Option<usize>,  // Index into available microphones, None = default
    scales_to_fit: bool,
    preserves_aspect_ratio: bool,
    queue_depth: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            fps: 60,
            show_cursor: true,
            captures_audio: true,
            captures_mic: true,
            mic_device_idx: None,  // Use system default
            scales_to_fit: true,
            preserves_aspect_ratio: true,
            queue_depth: 8,
        }
    }
}

impl StreamConfig {
    const OPTIONS: &'static [&'static str] = &[
        "FPS",
        "Cursor",
        "Audio",
        "Mic",
        "Mic Device",
        "Scale Fit",
        "Aspect",
        "Queue",
    ];
    
    fn option_count() -> usize {
        Self::OPTIONS.len()
    }
    
    fn option_name(idx: usize) -> &'static str {
        Self::OPTIONS.get(idx).unwrap_or(&"?")
    }
    
    fn option_value(&self, idx: usize) -> String {
        match idx {
            0 => format!("{}", self.fps),
            1 => if self.show_cursor { "On" } else { "Off" }.to_string(),
            2 => if self.captures_audio { "On" } else { "Off" }.to_string(),
            3 => if self.captures_mic { "On" } else { "Off" }.to_string(),
            4 => {
                // Mic device selection
                let devices = AudioInputDevice::list();
                match self.mic_device_idx {
                    None => "Default".to_string(),
                    Some(idx) => devices.get(idx)
                        .map(|d| d.name.chars().take(10).collect::<String>())
                        .unwrap_or_else(|| "?".to_string()),
                }
            }
            5 => if self.scales_to_fit { "On" } else { "Off" }.to_string(),
            6 => if self.preserves_aspect_ratio { "On" } else { "Off" }.to_string(),
            7 => format!("{}", self.queue_depth),
            _ => "?".to_string(),
        }
    }
    
    fn toggle_or_adjust(&mut self, idx: usize, increase: bool) {
        match idx {
            0 => {
                // FPS: cycle through 15, 30, 60, 120
                let fps_options = [15, 30, 60, 120];
                let current_idx = fps_options.iter().position(|&f| f == self.fps).unwrap_or(2);
                let new_idx = if increase {
                    (current_idx + 1) % fps_options.len()
                } else {
                    (current_idx + fps_options.len() - 1) % fps_options.len()
                };
                self.fps = fps_options[new_idx];
            }
            1 => self.show_cursor = !self.show_cursor,
            2 => self.captures_audio = !self.captures_audio,
            3 => self.captures_mic = !self.captures_mic,
            4 => {
                // Cycle through available microphones
                let devices = AudioInputDevice::list();
                if devices.is_empty() {
                    return;
                }
                match self.mic_device_idx {
                    None => {
                        self.mic_device_idx = Some(if increase { 0 } else { devices.len() - 1 });
                    }
                    Some(idx) => {
                        if increase {
                            if idx + 1 >= devices.len() {
                                self.mic_device_idx = None;
                            } else {
                                self.mic_device_idx = Some(idx + 1);
                            }
                        } else if idx == 0 {
                            self.mic_device_idx = None;
                        } else {
                            self.mic_device_idx = Some(idx - 1);
                        }
                    }
                }
            }
            5 => self.scales_to_fit = !self.scales_to_fit,
            6 => self.preserves_aspect_ratio = !self.preserves_aspect_ratio,
            7 => {
                // Queue depth: cycle through 3, 5, 8, 12
                let queue_options = [3, 5, 8, 12];
                let current_idx = queue_options.iter().position(|&q| q == self.queue_depth).unwrap_or(2);
                let new_idx = if increase {
                    (current_idx + 1) % queue_options.len()
                } else {
                    (current_idx + queue_options.len() - 1) % queue_options.len()
                };
                self.queue_depth = queue_options[new_idx];
            }
            _ => {}
        }
    }
    
    /// Build an SCStreamConfiguration from current settings
    fn to_stream_config(&self, width: u32, height: u32) -> SCStreamConfiguration {
        let mut config = SCStreamConfiguration::new()
            .with_width(width)
            .with_height(height)
            .with_fps(self.fps)
            .with_shows_cursor(self.show_cursor)
            .with_captures_audio(self.captures_audio)
            .with_excludes_current_process_audio(true)
            .with_captures_microphone(self.captures_mic)
            .with_channel_count(2)
            .with_sample_rate(48000)
            .with_scales_to_fit(self.scales_to_fit)
            .with_preserves_aspect_ratio(self.preserves_aspect_ratio)
            .with_queue_depth(self.queue_depth)
            .with_pixel_format(screencapturekit::stream::configuration::PixelFormat::BGRA);
        
        // Set microphone device if specified
        if let Some(idx) = self.mic_device_idx {
            let devices = AudioInputDevice::list();
            if let Some(device) = devices.get(idx) {
                config.set_microphone_capture_device_id(&device.id);
            }
        }
        
        config
    }
}

impl OverlayState {
    fn new() -> Self {
        Self {
            show_help: true,
            show_waveform: true,
            show_config: false,
            config_selection: 0,
            menu_selection: 0,
        }
    }
    
    const MENU_ITEMS: &'static [&'static str] = &["Picker", "Source", "Capture", "Config", "Quit"];
    
    fn menu_count() -> usize {
        Self::MENU_ITEMS.len()
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

    /// Draw a vertical VU meter (for gain visualization)
    fn vu_meter_vertical(&mut self, level: f32, x: f32, y: f32, w: f32, h: f32, label: &str, font: &BitmapFont) {
        // Background
        self.rect(x, y, w, h, [0.1, 0.1, 0.1, 0.9]);

        // Calculate fill height from bottom
        let db = if level > 0.0 {
            20.0 * level.log10()
        } else {
            -60.0
        };
        let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let fill_h = normalized * h;

        // Color segments (from bottom: green -> yellow -> red)
        let green_end = h * 0.6;
        let yellow_end = h * 0.85;

        // Green section (bottom)
        if fill_h > 0.0 {
            let segment_h = fill_h.min(green_end);
            self.rect(x, y + h - segment_h, w, segment_h, [0.2, 0.9, 0.2, 1.0]);
        }
        // Yellow section (middle)
        if fill_h > green_end {
            let segment_h = (fill_h - green_end).min(yellow_end - green_end);
            self.rect(x, y + h - green_end - segment_h, w, segment_h, [0.9, 0.9, 0.2, 1.0]);
        }
        // Red section (top)
        if fill_h > yellow_end {
            let segment_h = fill_h - yellow_end;
            self.rect(x, y + h - yellow_end - segment_h, w, segment_h, [0.9, 0.2, 0.2, 1.0]);
        }

        // Border
        self.rect_outline(x, y, w, h, 1.0, [0.5, 0.5, 0.5, 0.8]);

        // Label below meter
        self.text(font, label, x, y + h + 4.0, 1.0, [0.8, 0.8, 0.8, 1.0]);
        
        // dB markers
        let marker_color = [0.4, 0.4, 0.4, 0.8];
        // -60dB (bottom)
        self.rect(x - 4.0, y + h - 2.0, 4.0, 1.0, marker_color);
        // -30dB (middle-low)
        self.rect(x - 4.0, y + h * 0.5 - 1.0, 4.0, 1.0, marker_color);
        // -12dB (middle-high)
        self.rect(x - 4.0, y + h * 0.2 - 1.0, 4.0, 1.0, marker_color);
        // 0dB (top)
        self.rect(x - 4.0, y, 4.0, 1.0, [0.9, 0.2, 0.2, 0.8]);
    }

    /// Draw a simple help overlay with menu navigation
    fn help_overlay(&mut self, font: &BitmapFont, viewport_w: f32, viewport_h: f32, is_capturing: bool, source_name: &str, menu_selection: usize) {
        // Responsive scaling based on viewport
        let base_scale = (viewport_w.min(viewport_h) / 600.0).clamp(1.0, 3.0);
        let scale = 2.0 * base_scale;
        let line_h = 28.0 * base_scale;
        let padding = 24.0 * base_scale;
        
        let bg_color = [0.0, 0.0, 0.0, 0.9];
        let text_color = [1.0, 1.0, 1.0, 1.0];
        let selected_bg = [0.2, 0.5, 1.0, 0.9];
        let title_color = [1.0, 0.8, 0.2, 1.0];
        let hint_color = [0.5, 0.5, 0.5, 1.0];
        let source_color = [0.4, 0.9, 1.0, 1.0];
        
        let has_source = !source_name.is_empty() && source_name != "None";
        let menu_items = ["Picker", "Source", "Capture", "Config", "Quit"];
        let menu_values: [&str; 5] = [
            "Open",
            "", // Source value handled specially below
            if is_capturing { "Stop" } else if has_source { "Start" } else { "-" },
            "Open",
            "",
        ];
        
        // Calculate box dimensions - much wider menu
        let box_w = (550.0 * base_scale).min(viewport_w * 0.9);
        let box_h = (line_h * (menu_items.len() as f32 + 3.0) + padding * 2.0).min(viewport_h * 0.7);
        
        // Center the box
        let x = (viewport_w - box_w) / 2.0;
        let y = (viewport_h - box_h) / 2.0;
        
        // Background with border
        self.rect(x, y, box_w, box_h, bg_color);
        self.rect_outline(x, y, box_w, box_h, 3.0 * base_scale, [0.4, 0.6, 1.0, 1.0]);
        
        let mut ly = y + padding;
        let text_x = x + padding + 20.0 * base_scale; // Space for arrow
        
        // Title
        self.text(font, "MENU", text_x, ly, scale * 1.2, title_color);
        ly += line_h * 1.5;
        
        // Menu items with space-between justification
        for (i, (item, value)) in menu_items.iter().zip(menu_values.iter()).enumerate() {
            let is_selected = i == menu_selection;
            
            // Selection highlight
            if is_selected {
                self.rect(x + 4.0, ly - 6.0, box_w - 8.0, line_h + 4.0, selected_bg);
                // Arrow indicator
                self.text(font, ">", x + padding, ly, scale, [1.0, 1.0, 1.0, 1.0]);
            }
            
            // Label on left
            self.text(font, item, text_x, ly, scale, text_color);
            
            // Value on right - special handling for Source row
            if i == 1 {
                // Source row - show picked source info
                let display_value = if source_name.is_empty() || source_name == "None" { "None" } else { source_name };
                let truncated: String = if display_value.len() > 20 {
                    format!("{}...", &display_value.chars().take(17).collect::<String>())
                } else {
                    display_value.to_string()
                };
                let char_w = 8.0 * scale;
                let value_w = truncated.len() as f32 * char_w;
                let value_x = x + box_w - padding - value_w;
                self.text(font, &truncated, value_x, ly, scale, if has_source { source_color } else { hint_color });
            } else if !value.is_empty() {
                let char_w = 8.0 * scale; // Font is 8x8 pixels
                let value_w = value.len() as f32 * char_w;
                let value_x = x + box_w - padding - value_w;
                self.text(font, value, value_x, ly, scale, if is_selected { [1.0, 1.0, 0.5, 1.0] } else { hint_color });
            }
            ly += line_h;
        }
        
        // Hint at bottom
        ly += line_h * 0.3;
        self.text(font, "UP/DOWN  SPACE/ENTER  ESC", text_x, ly, scale * 0.6, hint_color);
    }

    /// Draw stream configuration menu
    fn config_menu(
        &mut self,
        font: &BitmapFont,
        viewport_w: f32,
        viewport_h: f32,
        config: &StreamConfig,
        selection: usize,
        is_capturing: bool,
    ) {
        let base_scale = (viewport_w.min(viewport_h) / 600.0).clamp(1.0, 3.0);
        let scale = 2.0 * base_scale;
        let line_h = 28.0 * base_scale;
        let padding = 24.0 * base_scale;
        
        let bg_color = [0.05, 0.05, 0.15, 0.95];
        let text_color = [1.0, 1.0, 1.0, 1.0];
        let selected_bg = [0.2, 0.4, 0.8, 0.8];
        let title_color = [0.4, 0.9, 1.0, 1.0];
        let hint_color = [0.6, 0.6, 0.6, 1.0];
        let value_color = [0.8, 1.0, 0.8, 1.0];
        
        let option_count = StreamConfig::option_count();
        let box_w = (550.0 * base_scale).min(viewport_w * 0.9);
        let box_h = (line_h * (option_count as f32 + 3.0) + padding * 2.0).min(viewport_h * 0.7);
        
        let x = (viewport_w - box_w) / 2.0;
        let y = (viewport_h - box_h) / 2.0;
        
        // Background
        self.rect(x, y, box_w, box_h, bg_color);
        self.rect_outline(x, y, box_w, box_h, 3.0 * base_scale, [0.3, 0.5, 0.8, 1.0]);
        
        let mut ly = y + padding;
        let text_x = x + padding + 20.0 * base_scale; // Space for arrow
        
        // Title
        self.text(font, "CONFIG", text_x, ly, scale * 1.2, title_color);
        ly += line_h * 1.5;
        
        // Options with space-between layout
        for i in 0..option_count {
            let is_selected = i == selection;
            
            // Selection highlight
            if is_selected {
                self.rect(x + 4.0, ly - 6.0, box_w - 8.0, line_h + 4.0, selected_bg);
                // Arrow indicator
                self.text(font, ">", x + padding, ly, scale, [1.0, 1.0, 1.0, 1.0]);
            }
            
            let name = StreamConfig::option_name(i);
            let value = config.option_value(i);
            
            // Label on left
            self.text(font, name, text_x, ly, scale, text_color);
            
            // Value on right - aligned to right edge of box
            let max_value_chars = 10;
            let truncated: String = if value.len() > max_value_chars {
                format!("{}...", &value[..max_value_chars-3])
            } else {
                value
            };
            let char_w = 8.0 * scale; // Font is 8x8 pixels
            let value_w = truncated.len() as f32 * char_w;
            let value_x = x + box_w - padding - value_w;
            self.text(font, &truncated, value_x, ly, scale, if is_selected { value_color } else { hint_color });
            
            ly += line_h;
        }
        
        // Hint at bottom
        ly += line_h * 0.3;
        let hint = if is_capturing { "LEFT/RIGHT  ENTER=Apply  ESC" } else { "LEFT/RIGHT  ESC" };
        self.text(font, hint, text_x, ly, scale * 0.6, hint_color);
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
    audio_waveform: Mutex<WaveformBuffer>,
    mic_waveform: Mutex<WaveformBuffer>,
    /// The latest captured frame's IOSurface (retained)
    latest_surface: Mutex<Option<IOSurface>>,
}

impl CaptureState {
    fn new() -> Self {
        Self {
            frame_count: AtomicUsize::new(0),
            audio_waveform: Mutex::new(WaveformBuffer::new(4096)),
            mic_waveform: Mutex::new(WaveformBuffer::new(4096)),
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
                // Extract real audio samples from the buffer
                if let Some(audio_buffer_list) = sample.audio_buffer_list() {
                    if let Some(buffer) = audio_buffer_list.get(0) {
                        let data = buffer.data();
                        // Convert bytes to f32 samples (assuming 32-bit float audio)
                        let samples: Vec<f32> = data
                            .chunks_exact(4)
                            .map(|chunk| {
                                let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                                f32::from_le_bytes(bytes)
                            })
                            .collect();
                        
                        if !samples.is_empty() {
                            let mut waveform = self.state.audio_waveform.lock().unwrap();
                            waveform.push(&samples);
                        }
                    }
                }
            }
            SCStreamOutputType::Microphone => {
                // Extract microphone audio samples
                if let Some(audio_buffer_list) = sample.audio_buffer_list() {
                    if let Some(buffer) = audio_buffer_list.get(0) {
                        let data = buffer.data();
                        // Convert bytes to f32 samples (assuming 32-bit float audio)
                        let samples: Vec<f32> = data
                            .chunks_exact(4)
                            .map(|chunk| {
                                let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                                f32::from_le_bytes(bytes)
                            })
                            .collect();
                        
                        if !samples.is_empty() {
                            let mut waveform = self.state.mic_waveform.lock().unwrap();
                            waveform.push(&samples);
                        }
                    }
                }
            }
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
    let mut stream_config = StreamConfig::default();

    // Helper to format picked source for display
    fn format_picked_source(source: &SCPickedSource) -> String {
        match source {
            SCPickedSource::Window(name) => format!("[W] {}", name.chars().take(20).collect::<String>()),
            SCPickedSource::Display(id) => format!("[D] Display {}", id),
            SCPickedSource::Application(name) => format!("[A] {}", name.chars().take(20).collect::<String>()),
            SCPickedSource::Unknown => "None".to_string(),
        }
    }

    // Screen capture setup
    let mut stream: Option<SCStream> = None;
    let mut current_filter: Option<SCContentFilter> = None;
    let mut capture_size: (u32, u32) = (1920, 1080);
    let mut picked_source = SCPickedSource::Unknown;
    
    // Shared state for picker callback results (filter, width, height, source info)
    type PickerResult = Option<(SCContentFilter, u32, u32, SCPickedSource)>;
    let pending_picker: Arc<Mutex<PickerResult>> = Arc::new(Mutex::new(None));

    let mut vertex_builder = VertexBufferBuilder::new();
    let mut time = 0.0f32;

    println!("🎮 Press SPACE to open content picker");

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;
            
            // Check for pending picker results - just store the filter, don't auto-start capture
            if let Ok(mut pending) = pending_picker.try_lock() {
                if let Some((filter, width, height, source)) = pending.take() {
                    println!("✅ Content selected: {}x{} - {}", width, height, format_picked_source(&source));
                    capture_size = (width, height);
                    current_filter = Some(filter);
                    picked_source = source;
                    println!("ℹ️  Press Capture to start streaming");
                }
            }

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
                        // Helper closure to open picker
                        let open_picker = |pending_picker: &Arc<Mutex<PickerResult>>| {
                            println!("📺 Opening content picker...");
                            let mut config = SCContentSharingPickerConfiguration::new();
                            // Allow all picker modes: windows, displays, applications
                            config.set_allowed_picker_modes(&[
                                SCContentSharingPickerMode::SingleWindow,
                                SCContentSharingPickerMode::MultipleWindows,
                                SCContentSharingPickerMode::SingleDisplay,
                                SCContentSharingPickerMode::SingleApplication,
                                SCContentSharingPickerMode::MultipleApplications,
                            ]);
                            let pending = Arc::clone(pending_picker);
                            
                            SCContentSharingPicker::show(&config, move |outcome| {
                                match outcome {
                                    SCPickerOutcome::Picked(result) => {
                                        let (width, height) = result.pixel_size();
                                        let filter = result.filter();
                                        let source = result.source();
                                        
                                        if let Ok(mut pending) = pending.lock() {
                                            *pending = Some((filter, width, height, source));
                                        }
                                    }
                                    SCPickerOutcome::Cancelled => {
                                        println!("⚠️  Picker cancelled");
                                    }
                                    SCPickerOutcome::Error(e) => {
                                        eprintln!("❌ Picker error: {}", e);
                                    }
                                }
                            });
                        };
                        
                        // Helper closure to start capture with existing filter
                        let start_capture = |stream: &mut Option<SCStream>,
                                            current_filter: &Option<SCContentFilter>,
                                            capture_size: (u32, u32),
                                            stream_config: &StreamConfig,
                                            capture_state: &Arc<CaptureState>,
                                            capturing: &Arc<AtomicBool>| {
                            if let Some(ref filter) = current_filter {
                                let (width, height) = capture_size;
                                let sc_config = stream_config.to_stream_config(width, height);
                                
                                let handler = CaptureHandler {
                                    state: Arc::clone(capture_state),
                                };

                                let mut s = SCStream::new(filter, &sc_config);
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Screen);
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Audio);
                                s.add_output_handler(handler, SCStreamOutputType::Microphone);
                                
                                match s.start_capture() {
                                    Ok(()) => {
                                        capturing.store(true, Ordering::Relaxed);
                                        *stream = Some(s);
                                        println!("✅ Capture started");
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to start capture: {:?}", e);
                                    }
                                }
                            } else {
                                println!("⚠️  No content selected. Open picker first.");
                            }
                        };
                        
                        // Helper closure to stop capture
                        let stop_capture = |stream: &mut Option<SCStream>,
                                           capturing: &Arc<AtomicBool>| {
                            println!("⏹️  Stopping capture...");
                            if let Some(ref mut s) = stream {
                                let _ = s.stop_capture();
                            }
                            *stream = None;
                            capturing.store(false, Ordering::Relaxed);
                            println!("✅ Capture stopped");
                        };
                        
                        // Handle menu navigation when help is shown
                        if overlay.show_help && !overlay.show_config {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    if overlay.menu_selection > 0 {
                                        overlay.menu_selection -= 1;
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    let max = OverlayState::menu_count().saturating_sub(1);
                                    if overlay.menu_selection < max {
                                        overlay.menu_selection += 1;
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    match overlay.menu_selection {
                                        0 => { // Picker
                                            open_picker(&pending_picker);
                                        }
                                        1 => { // Source (info only, do nothing or re-open picker)
                                            // Source row is informational, pressing enter re-opens picker
                                            open_picker(&pending_picker);
                                        }
                                        2 => { // Capture start/stop
                                            if capturing.load(Ordering::Relaxed) {
                                                stop_capture(&mut stream, &capturing);
                                            } else {
                                                start_capture(&mut stream, &current_filter, capture_size, &stream_config, &capture_state, &capturing);
                                            }
                                        }
                                        3 => { // Config
                                            overlay.show_config = true;
                                            overlay.show_help = false;
                                        }
                                        4 => { // Quit
                                            *control_flow = ControlFlow::ExitWithCode(0);
                                        }
                                        _ => {}
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::H => {
                                    overlay.show_help = false;
                                }
                                VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
                        }
                        // Handle config menu navigation
                        else if overlay.show_config {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    if overlay.config_selection > 0 {
                                        overlay.config_selection -= 1;
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    let max = StreamConfig::option_count().saturating_sub(1);
                                    if overlay.config_selection < max {
                                        overlay.config_selection += 1;
                                    }
                                }
                                VirtualKeyCode::Left | VirtualKeyCode::Right => {
                                    let increase = keycode == VirtualKeyCode::Right;
                                    stream_config.toggle_or_adjust(overlay.config_selection, increase);
                                    // Immediately apply config to running stream
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let new_config = stream_config.to_stream_config(capture_size.0, capture_size.1);
                                            if let Err(e) = s.update_configuration(&new_config) {
                                                eprintln!("❌ Config update failed: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    // Toggle current option (same as Right arrow)
                                    stream_config.toggle_or_adjust(overlay.config_selection, true);
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let new_config = stream_config.to_stream_config(capture_size.0, capture_size.1);
                                            if let Err(e) = s.update_configuration(&new_config) {
                                                eprintln!("❌ Config update failed: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::Back => {
                                    overlay.show_config = false;
                                    overlay.show_help = true;
                                }
                                VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
                        }
                        // Default key handling (no menu shown)
                        else {
                            match keycode {
                                VirtualKeyCode::Space => {
                                    // Toggle capture on/off
                                    if capturing.load(Ordering::Relaxed) {
                                        stop_capture(&mut stream, &capturing);
                                    } else {
                                        start_capture(&mut stream, &current_filter, capture_size, &stream_config, &capture_state, &capturing);
                                    }
                                }
                                VirtualKeyCode::P => {
                                    open_picker(&pending_picker);
                                }
                                VirtualKeyCode::W => {
                                    overlay.show_waveform = !overlay.show_waveform;
                                }
                                VirtualKeyCode::H => {
                                    overlay.show_help = true;
                                }
                                VirtualKeyCode::C => {
                                    overlay.show_config = true;
                                }
                                VirtualKeyCode::M => {
                                    stream_config.captures_mic = !stream_config.captures_mic;
                                    println!("🎤 Microphone: {}", if stream_config.captures_mic { "On" } else { "Off" });
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let new_config = stream_config.to_stream_config(capture_size.0, capture_size.1);
                                            match s.update_configuration(&new_config) {
                                                Ok(()) => println!("✅ Config updated"),
                                                Err(e) => eprintln!("❌ Config update failed: {:?}", e),
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
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
                        "H=Menu".to_string()
                    };
                    vertex_builder.text(&font, &status, 8.0, 8.0, 2.0, [0.2, 1.0, 0.3, 1.0]);

                    // Waveform bar at top - 100% width
                    if overlay.show_waveform && capturing.load(Ordering::Relaxed) {
                        let wave_h = 60.0;
                        let bar_y = 36.0; // Below status bar
                        let meter_w = 24.0;
                        let padding = 8.0;
                        
                        // Waveform background - full width
                        vertex_builder.rect(0.0, bar_y, width, wave_h + 8.0, [0.08, 0.08, 0.1, 0.9]);
                        
                        // Calculate waveform area (leave space for meters on right)
                        let meters_space = meter_w * 2.0 + padding * 3.0;
                        let wave_w = width - meters_space - padding;
                        let wave_x = padding;
                        let wave_y = bar_y + 4.0;

                        // System audio waveform - full width
                        let audio_samples = capture_state.audio_waveform.lock().unwrap().display_samples(512);
                        vertex_builder.waveform(
                            &audio_samples,
                            wave_x,
                            wave_y,
                            wave_w,
                            wave_h,
                            [0.3, 1.0, 0.4, 0.8],
                        );

                        // Vertical meters on the right
                        let meters_x = width - meters_space + padding;
                        
                        // System audio vertical meter
                        let audio_level = capture_state.audio_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            audio_level,
                            meters_x,
                            wave_y,
                            meter_w,
                            wave_h,
                            "S",
                            &font,
                        );

                        // Microphone vertical meter
                        let mic_level = capture_state.mic_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            mic_level,
                            meters_x + meter_w + padding,
                            wave_y,
                            meter_w,
                            wave_h,
                            "M",
                            &font,
                        );
                    }

                    // Help overlay - responsive centered
                    if overlay.show_help {
                        let source_str = format_picked_source(&picked_source);
                        vertex_builder.help_overlay(&font, width, height, capturing.load(Ordering::Relaxed), &source_str, overlay.menu_selection);
                    }
                    
                    // Config menu overlay
                    if overlay.show_config {
                        vertex_builder.config_menu(
                            &font,
                            width,
                            height,
                            &stream_config,
                            overlay.config_selection,
                            capturing.load(Ordering::Relaxed),
                        );
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
                        texture_size: [tex_width, tex_height],
                        time,
                        _padding: [0.0; 3],
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
                        encoder.set_vertex_buffer(0, Some(&uniforms_buffer), 0);
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
