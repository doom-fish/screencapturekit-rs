//! Metal Renderer with Overlay UI
//!
//! A real GUI application demonstrating:
//! - Metal rendering with compiled shaders
//! - Screen capture via ScreenCaptureKit with zero-copy IOSurface textures
//! - System content picker (macOS 14.0+) for user-selected capture
//! - Interactive overlay menu with bitmap font rendering
//! - Real-time audio waveform visualization with vertical gain meters
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

// =============================================================================
// Module: font - Bitmap font for text rendering
// =============================================================================
mod font {
    pub struct BitmapFont {
        glyphs: [u64; 128],
    }

    impl BitmapFont {
        pub fn new() -> Self {
            let mut glyphs = [0u64; 128];
            glyphs[b' ' as usize] = 0x0000_0000_0000_0000;
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
            for c in b'a'..=b'z' { glyphs[c as usize] = glyphs[(c - 32) as usize]; }
            glyphs[b':' as usize] = 0x0018_1800_1818_0000;
            glyphs[b'.' as usize] = 0x0000_0000_0018_1800;
            glyphs[b'-' as usize] = 0x0000_007E_0000_0000;
            glyphs[b'[' as usize] = 0x3C30_3030_3030_3C00;
            glyphs[b']' as usize] = 0x3C0C_0C0C_0C0C_3C00;
            glyphs[b'>' as usize] = 0x6030_180C_1830_6000;
            Self { glyphs }
        }

        pub fn glyph(&self, c: char) -> u64 {
            let idx = c as usize;
            if idx < 128 { self.glyphs[idx] } else { 0 }
        }

        pub fn pixel_set(&self, glyph: u64, x: usize, y: usize) -> bool {
            if x >= 8 || y >= 8 { return false; }
            let row = (glyph >> (56 - y * 8)) & 0xFF;
            (row >> (7 - x)) & 1 == 1
        }
    }
}

// =============================================================================
// Module: waveform - Audio waveform buffer
// =============================================================================
mod waveform {
    pub struct WaveformBuffer {
        samples: Vec<f32>,
        write_pos: usize,
    }

    impl WaveformBuffer {
        pub fn new(capacity: usize) -> Self {
            Self { samples: vec![0.0; capacity], write_pos: 0 }
        }

        pub fn push(&mut self, data: &[f32]) {
            for &s in data {
                self.samples[self.write_pos] = s;
                self.write_pos = (self.write_pos + 1) % self.samples.len();
            }
        }

        pub fn display_samples(&self, count: usize) -> Vec<f32> {
            let count = count.min(self.samples.len());
            let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
            (0..count).map(|i| self.samples[(start + i) % self.samples.len()]).collect()
        }

        pub fn rms(&self, count: usize) -> f32 {
            let count = count.min(self.samples.len());
            if count == 0 { return 0.0; }
            let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
            let sum: f32 = (0..count).map(|i| {
                let s = self.samples[(start + i) % self.samples.len()];
                s * s
            }).sum();
            (sum / count as f32).sqrt()
        }
    }
}

// =============================================================================
// Module: vertex - GPU data structures and vertex buffer
// =============================================================================
mod vertex {
    use std::mem::size_of;
    use metal::*;
    use super::font::BitmapFont;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Vertex {
        pub position: [f32; 2],
        pub color: [f32; 4],
    }

    impl Vertex {
        pub const fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
            Self { position: [x, y], color: [r, g, b, a] }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Uniforms {
        pub viewport_size: [f32; 2],
        pub texture_size: [f32; 2],
        pub time: f32,
        pub _padding: [f32; 3],
    }

    pub struct VertexBufferBuilder {
        vertices: Vec<Vertex>,
    }

    impl VertexBufferBuilder {
        pub fn new() -> Self { Self { vertices: vec![] } }
        pub fn clear(&mut self) { self.vertices.clear(); }

        pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
            let tl = Vertex::new(x, y, color[0], color[1], color[2], color[3]);
            let tr = Vertex::new(x + w, y, color[0], color[1], color[2], color[3]);
            let bl = Vertex::new(x, y + h, color[0], color[1], color[2], color[3]);
            let br = Vertex::new(x + w, y + h, color[0], color[1], color[2], color[3]);
            self.vertices.extend_from_slice(&[tl, tr, bl, tr, br, bl]);
        }

        pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: [f32; 4]) {
            self.rect(x, y, w, thickness, color);
            self.rect(x, y + h - thickness, w, thickness, color);
            self.rect(x, y, thickness, h, color);
            self.rect(x + w - thickness, y, thickness, h, color);
        }

        pub fn text(&mut self, font: &BitmapFont, text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) {
            let scale_i = scale as i32;
            let scale_f = scale_i as f32;
            let mut cx = x.floor() as i32;
            let y_i = y.floor() as i32;
            for c in text.chars() {
                let glyph = font.glyph(c);
                for py in 0..8 {
                    for px in 0..8 {
                        if font.pixel_set(glyph, px, py) {
                            self.rect((cx + px as i32 * scale_i) as f32, (y_i + py as i32 * scale_i) as f32, scale_f, scale_f, color);
                        }
                    }
                }
                cx += 8 * scale_i;
            }
        }

        pub fn waveform(&mut self, samples: &[f32], x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
            if samples.is_empty() { return; }
            let center_y = y + h / 2.0;
            let half_h = h / 2.0;
            let step = samples.len() as f32 / w;
            let bar_w = 2.0;
            for i in 0..(w as usize / bar_w as usize) {
                let sample_idx = ((i as f32 * bar_w) * step) as usize;
                let sample = samples.get(sample_idx).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                let bar_h = sample.abs() * half_h;
                let bar_y = if sample >= 0.0 { center_y - bar_h } else { center_y };
                self.rect(x + i as f32 * bar_w, bar_y, bar_w - 1.0, bar_h, color);
            }
        }

        pub fn vu_meter_vertical(&mut self, level: f32, x: f32, y: f32, w: f32, h: f32, label: &str, font: &BitmapFont) {
            self.rect(x, y, w, h, [0.1, 0.1, 0.1, 0.9]);
            let db = if level > 0.0 { 20.0 * level.log10() } else { -60.0 };
            let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
            let fill_h = normalized * h;
            let green_end = h * 0.6;
            let yellow_end = h * 0.85;
            if fill_h > 0.0 { self.rect(x, y + h - fill_h.min(green_end), w, fill_h.min(green_end), [0.2, 0.9, 0.2, 1.0]); }
            if fill_h > green_end { self.rect(x, y + h - green_end - (fill_h - green_end).min(yellow_end - green_end), w, (fill_h - green_end).min(yellow_end - green_end), [0.9, 0.9, 0.2, 1.0]); }
            if fill_h > yellow_end { self.rect(x, y + h - yellow_end - (fill_h - yellow_end), w, fill_h - yellow_end, [0.9, 0.2, 0.2, 1.0]); }
            self.rect_outline(x, y, w, h, 1.0, [0.5, 0.5, 0.5, 0.8]);
            self.text(font, label, x, y + h + 4.0, 1.0, [0.8, 0.8, 0.8, 1.0]);
        }

        pub fn build(&self, device: &Device) -> Buffer {
            device.new_buffer_with_data(self.vertices.as_ptr().cast(), (self.vertices.len() * size_of::<Vertex>()) as u64, MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged)
        }

        pub fn vertex_count(&self) -> usize { self.vertices.len() }
    }
}

// =============================================================================
// Module: overlay - Menu state and configuration
// =============================================================================
mod overlay {
    use screencapturekit::prelude::*;

    pub struct OverlayState {
        pub show_help: bool,
        pub show_waveform: bool,
        pub show_config: bool,
        pub config_selection: usize,
        pub menu_selection: usize,
    }

    impl OverlayState {
        pub fn new() -> Self {
            Self { show_help: true, show_waveform: true, show_config: false, config_selection: 0, menu_selection: 0 }
        }
        pub const MENU_ITEMS: &'static [&'static str] = &["Picker", "Source", "Capture", "Config", "Quit"];
        pub fn menu_count() -> usize { Self::MENU_ITEMS.len() }
    }

    pub struct ConfigMenu;
    impl ConfigMenu {
        pub const OPTIONS: &'static [&'static str] = &["FPS", "Cursor", "Audio", "Mic", "Mic Device", "Scale Fit", "Aspect", "Queue"];
        pub const FPS_OPTIONS: [u32; 4] = [15, 30, 60, 120];
        pub const QUEUE_OPTIONS: [u32; 4] = [3, 5, 8, 12];
        pub fn option_count() -> usize { Self::OPTIONS.len() }
        pub fn option_name(idx: usize) -> &'static str { Self::OPTIONS.get(idx).unwrap_or(&"?") }

        pub fn option_value(config: &SCStreamConfiguration, mic_device_idx: Option<usize>, idx: usize) -> String {
            match idx {
                0 => format!("{}", config.fps()),
                1 => if config.get_shows_cursor() { "On" } else { "Off" }.to_string(),
                2 => if config.get_captures_audio() { "On" } else { "Off" }.to_string(),
                3 => if config.get_captures_microphone() { "On" } else { "Off" }.to_string(),
                4 => match mic_device_idx {
                    None => "Default".to_string(),
                    Some(idx) => AudioInputDevice::list().get(idx).map(|d| d.name.chars().take(10).collect()).unwrap_or_else(|| "?".to_string()),
                },
                5 => if config.get_scales_to_fit() { "On" } else { "Off" }.to_string(),
                6 => if config.get_preserves_aspect_ratio() { "On" } else { "Off" }.to_string(),
                7 => format!("{}", config.get_queue_depth()),
                _ => "?".to_string(),
            }
        }

        pub fn toggle_or_adjust(config: &mut SCStreamConfiguration, mic_device_idx: &mut Option<usize>, idx: usize, increase: bool) {
            match idx {
                0 => {
                    let current_fps = config.fps();
                    let current_idx = Self::FPS_OPTIONS.iter().position(|&f| f == current_fps).unwrap_or(2);
                    let new_idx = if increase { (current_idx + 1) % Self::FPS_OPTIONS.len() } else { (current_idx + Self::FPS_OPTIONS.len() - 1) % Self::FPS_OPTIONS.len() };
                    config.set_fps(Self::FPS_OPTIONS[new_idx]);
                }
                1 => { config.set_shows_cursor(!config.get_shows_cursor()); }
                2 => { config.set_captures_audio(!config.get_captures_audio()); }
                3 => { config.set_captures_microphone(!config.get_captures_microphone()); }
                4 => {
                    let devices = AudioInputDevice::list();
                    if devices.is_empty() { return; }
                    match *mic_device_idx {
                        None => *mic_device_idx = Some(if increase { 0 } else { devices.len() - 1 }),
                        Some(idx) => {
                            if increase { *mic_device_idx = if idx + 1 >= devices.len() { None } else { Some(idx + 1) }; }
                            else { *mic_device_idx = if idx == 0 { None } else { Some(idx - 1) }; }
                        }
                    }
                    if let Some(idx) = *mic_device_idx {
                        if let Some(device) = devices.get(idx) { config.set_microphone_capture_device_id(&device.id); }
                    }
                }
                5 => { config.set_scales_to_fit(!config.get_scales_to_fit()); }
                6 => { config.set_preserves_aspect_ratio(!config.get_preserves_aspect_ratio()); }
                7 => {
                    let current_depth = config.get_queue_depth();
                    let current_idx = Self::QUEUE_OPTIONS.iter().position(|&q| q == current_depth).unwrap_or(2);
                    let new_idx = if increase { (current_idx + 1) % Self::QUEUE_OPTIONS.len() } else { (current_idx + Self::QUEUE_OPTIONS.len() - 1) % Self::QUEUE_OPTIONS.len() };
                    config.set_queue_depth(Self::QUEUE_OPTIONS[new_idx]);
                }
                _ => {}
            }
        }
    }

    pub fn default_stream_config() -> SCStreamConfiguration {
        SCStreamConfiguration::new()
            .with_width(1920).with_height(1080).with_fps(60)
            .with_shows_cursor(true).with_captures_audio(true)
            .with_excludes_current_process_audio(true).with_captures_microphone(true)
            .with_channel_count(2).with_sample_rate(48000)
            .with_scales_to_fit(true).with_preserves_aspect_ratio(true)
            .with_queue_depth(8)
            .with_pixel_format(screencapturekit::stream::configuration::PixelFormat::BGRA)
    }
}

// =============================================================================
// Module: capture - Screen capture handler
// =============================================================================
mod capture {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use screencapturekit::output::{IOSurface, CVPixelBufferIOSurface};
    use screencapturekit::prelude::*;
    use super::waveform::WaveformBuffer;

    pub struct CaptureState {
        pub frame_count: AtomicUsize,
        pub audio_waveform: Mutex<WaveformBuffer>,
        pub mic_waveform: Mutex<WaveformBuffer>,
        pub latest_surface: Mutex<Option<IOSurface>>,
    }

    impl CaptureState {
        pub fn new() -> Self {
            Self {
                frame_count: AtomicUsize::new(0),
                audio_waveform: Mutex::new(WaveformBuffer::new(4096)),
                mic_waveform: Mutex::new(WaveformBuffer::new(4096)),
                latest_surface: Mutex::new(None),
            }
        }
    }

    pub struct CaptureHandler { pub state: Arc<CaptureState> }
    impl Clone for CaptureHandler {
        fn clone(&self) -> Self { Self { state: Arc::clone(&self.state) } }
    }
    unsafe impl Send for CaptureHandler {}
    unsafe impl Sync for CaptureHandler {}

    impl SCStreamOutputTrait for CaptureHandler {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
            match output_type {
                SCStreamOutputType::Screen => {
                    self.state.frame_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(pixel_buffer) = sample.image_buffer() {
                        if pixel_buffer.is_backed_by_iosurface() {
                            if let Some(surface) = pixel_buffer.iosurface() {
                                *self.state.latest_surface.lock().unwrap() = Some(surface);
                            }
                        }
                    }
                }
                SCStreamOutputType::Audio | SCStreamOutputType::Microphone => {
                    if let Some(audio_buffer_list) = sample.audio_buffer_list() {
                        if let Some(buffer) = audio_buffer_list.get(0) {
                            let samples: Vec<f32> = buffer.data().chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4]))).collect();
                            if !samples.is_empty() {
                                let waveform = if matches!(output_type, SCStreamOutputType::Audio) { &self.state.audio_waveform } else { &self.state.mic_waveform };
                                waveform.lock().unwrap().push(&samples);
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Module: renderer - Metal rendering helpers
// =============================================================================
mod renderer {
    use std::ffi::c_void;
    use metal::*;
    use metal::foreign_types::ForeignType;
    use objc::{msg_send, sel, sel_impl};

    #[link(name = "Metal", kind = "framework")]
    #[link(name = "IOSurface", kind = "framework")]
    extern "C" {
        fn IOSurfaceGetWidth(surface: *const c_void) -> usize;
        fn IOSurfaceGetHeight(surface: *const c_void) -> usize;
    }

    pub unsafe fn create_texture_from_iosurface(device: &Device, iosurface_ptr: *const c_void) -> Option<Texture> {
        if iosurface_ptr.is_null() { return None; }
        let width = IOSurfaceGetWidth(iosurface_ptr);
        let height = IOSurfaceGetHeight(iosurface_ptr);
        if width == 0 || height == 0 { return None; }
        let desc = TextureDescriptor::new();
        desc.set_texture_type(MTLTextureType::D2);
        desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        desc.set_width(width as u64);
        desc.set_height(height as u64);
        desc.set_storage_mode(MTLStorageMode::Shared);
        desc.set_usage(MTLTextureUsage::ShaderRead);
        let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object iosurface: iosurface_ptr plane: 0usize];
        if texture.is_null() { None } else { Some(Texture::from_ptr(texture)) }
    }

    pub fn create_pipeline(device: &Device, library: &Library, vertex_fn: &str, fragment_fn: &str) -> RenderPipelineState {
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

    pub const SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;
struct Vertex { packed_float2 position; packed_float4 color; };
struct Uniforms { float2 viewport_size; float2 texture_size; float time; float padding[3]; };
struct VertexOut { float4 position [[position]]; float4 color; };
struct TexturedVertexOut { float4 position [[position]]; float2 texcoord; };
vertex VertexOut vertex_colored(const device Vertex* vertices [[buffer(0)]], constant Uniforms& uniforms [[buffer(1)]], uint vid [[vertex_id]]) {
    VertexOut out; float2 pos = vertices[vid].position; float2 ndc = (pos / uniforms.viewport_size) * 2.0 - 1.0; ndc.y = -ndc.y;
    out.position = float4(ndc, 0.0, 1.0); out.color = float4(vertices[vid].color); return out;
}
fragment float4 fragment_colored(VertexOut in [[stage_in]]) { return in.color; }
vertex TexturedVertexOut vertex_fullscreen(uint vid [[vertex_id]], constant Uniforms& uniforms [[buffer(0)]]) {
    TexturedVertexOut out; float va = uniforms.viewport_size.x / uniforms.viewport_size.y; float ta = uniforms.texture_size.x / uniforms.texture_size.y;
    float sx = ta > va ? 1.0 : ta / va; float sy = ta > va ? va / ta : 1.0;
    float2 positions[4] = { float2(-sx, -sy), float2(sx, -sy), float2(-sx, sy), float2(sx, sy) };
    float2 texcoords[4] = { float2(0.0, 1.0), float2(1.0, 1.0), float2(0.0, 0.0), float2(1.0, 0.0) };
    out.position = float4(positions[vid], 0.0, 1.0); out.texcoord = texcoords[vid]; return out;
}
fragment float4 fragment_textured(TexturedVertexOut in [[stage_in]], texture2d<float> tex [[texture(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear); return tex.sample(s, in.texcoord);
}
"#;
}

// =============================================================================
// Module: ui - UI drawing functions
// =============================================================================
mod ui {
    use super::font::BitmapFont;
    use super::overlay::{ConfigMenu, OverlayState};
    use super::vertex::VertexBufferBuilder;
    use screencapturekit::prelude::*;

    impl VertexBufferBuilder {
        pub fn help_overlay(&mut self, font: &BitmapFont, vw: f32, vh: f32, is_capturing: bool, source_name: &str, menu_selection: usize) {
            let base_scale = (vw.min(vh) / 600.0).clamp(1.0, 3.0);
            let scale = 2.0 * base_scale;
            let line_h = 28.0 * base_scale;
            let padding = 24.0 * base_scale;
            let has_source = !source_name.is_empty() && source_name != "None";
            let menu_values: [&str; 5] = ["Open", "", if is_capturing { "Stop" } else if has_source { "Start" } else { "-" }, "Open", ""];
            let box_w = (550.0 * base_scale).min(vw * 0.9);
            let box_h = (line_h * 8.0 + padding * 2.0).min(vh * 0.7);
            let x = (vw - box_w) / 2.0;
            let y = (vh - box_h) / 2.0;
            self.rect(x, y, box_w, box_h, [0.0, 0.0, 0.0, 0.9]);
            self.rect_outline(x, y, box_w, box_h, 3.0 * base_scale, [0.4, 0.6, 1.0, 1.0]);
            let mut ly = y + padding;
            let text_x = x + padding + 20.0 * base_scale;
            self.text(font, "MENU", text_x, ly, scale * 1.2, [1.0, 0.8, 0.2, 1.0]);
            ly += line_h * 1.5;
            for (i, (item, value)) in OverlayState::MENU_ITEMS.iter().zip(menu_values.iter()).enumerate() {
                let is_selected = i == menu_selection;
                if is_selected {
                    self.rect(x + 4.0, ly - 6.0, box_w - 8.0, line_h + 4.0, [0.2, 0.5, 1.0, 0.9]);
                    self.text(font, ">", x + padding, ly, scale, [1.0, 1.0, 1.0, 1.0]);
                }
                self.text(font, item, text_x, ly, scale, [1.0, 1.0, 1.0, 1.0]);
                if i == 1 {
                    let dv = if source_name.is_empty() || source_name == "None" { "None" } else { source_name };
                    let t: String = if dv.len() > 20 { format!("{}...", &dv.chars().take(17).collect::<String>()) } else { dv.to_string() };
                    let vx = x + box_w - padding - t.len() as f32 * 8.0 * scale;
                    self.text(font, &t, vx, ly, scale, if has_source { [0.4, 0.9, 1.0, 1.0] } else { [0.5, 0.5, 0.5, 1.0] });
                } else if !value.is_empty() {
                    let vx = x + box_w - padding - value.len() as f32 * 8.0 * scale;
                    self.text(font, value, vx, ly, scale, if is_selected { [1.0, 1.0, 0.5, 1.0] } else { [0.5, 0.5, 0.5, 1.0] });
                }
                ly += line_h;
            }
            ly += line_h * 0.3;
            self.text(font, "UP/DOWN  SPACE/ENTER  ESC", text_x, ly, scale * 0.6, [0.5, 0.5, 0.5, 1.0]);
        }

        pub fn config_menu(&mut self, font: &BitmapFont, vw: f32, vh: f32, config: &SCStreamConfiguration, mic_device_idx: Option<usize>, selection: usize, is_capturing: bool) {
            let base_scale = (vw.min(vh) / 600.0).clamp(1.0, 3.0);
            let scale = 2.0 * base_scale;
            let line_h = 28.0 * base_scale;
            let padding = 24.0 * base_scale;
            let option_count = ConfigMenu::option_count();
            let box_w = (550.0 * base_scale).min(vw * 0.9);
            let box_h = (line_h * (option_count as f32 + 3.0) + padding * 2.0).min(vh * 0.7);
            let x = (vw - box_w) / 2.0;
            let y = (vh - box_h) / 2.0;
            self.rect(x, y, box_w, box_h, [0.05, 0.05, 0.15, 0.95]);
            self.rect_outline(x, y, box_w, box_h, 3.0 * base_scale, [0.3, 0.5, 0.8, 1.0]);
            let mut ly = y + padding;
            let text_x = x + padding + 20.0 * base_scale;
            self.text(font, "CONFIG", text_x, ly, scale * 1.2, [0.4, 0.9, 1.0, 1.0]);
            ly += line_h * 1.5;
            for i in 0..option_count {
                let is_selected = i == selection;
                if is_selected {
                    self.rect(x + 4.0, ly - 6.0, box_w - 8.0, line_h + 4.0, [0.2, 0.4, 0.8, 0.8]);
                    self.text(font, ">", x + padding, ly, scale, [1.0, 1.0, 1.0, 1.0]);
                }
                let name = ConfigMenu::option_name(i);
                let value = ConfigMenu::option_value(config, mic_device_idx, i);
                self.text(font, name, text_x, ly, scale, [1.0, 1.0, 1.0, 1.0]);
                let t: String = if value.len() > 10 { format!("{}...", &value[..7]) } else { value };
                let vx = x + box_w - padding - t.len() as f32 * 8.0 * scale;
                self.text(font, &t, vx, ly, scale, if is_selected { [0.8, 1.0, 0.8, 1.0] } else { [0.6, 0.6, 0.6, 1.0] });
                ly += line_h;
            }
            ly += line_h * 0.3;
            let hint = if is_capturing { "LEFT/RIGHT  ENTER=Apply  ESC" } else { "LEFT/RIGHT  ESC" };
            self.text(font, hint, text_x, ly, scale * 0.6, [0.6, 0.6, 0.6, 1.0]);
        }
    }
}

// =============================================================================
// Imports and use statements
// =============================================================================
use cocoa::appkit::NSView;
use cocoa::base::id as cocoa_id;
use core_graphics_types::geometry::CGSize;
use metal::*;
use objc::rc::autoreleasepool;
use objc::runtime::YES;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use screencapturekit::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    SCPickedSource, SCPickerOutcome,
};
use screencapturekit::prelude::*;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use capture::{CaptureHandler, CaptureState};
use font::BitmapFont;
use overlay::{default_stream_config, ConfigMenu, OverlayState};
use renderer::{create_pipeline, create_texture_from_iosurface, SHADER_SOURCE};
use vertex::{Uniforms, Vertex, VertexBufferBuilder};

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
    let mut stream_config = default_stream_config();
    let mut mic_device_idx: Option<usize> = None;  // Track mic device selection separately

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
                                            stream_config: &SCStreamConfiguration,
                                            capture_state: &Arc<CaptureState>,
                                            capturing: &Arc<AtomicBool>| {
                            if let Some(ref filter) = current_filter {
                                let (width, height) = capture_size;
                                // Clone config and update dimensions
                                let mut sc_config = stream_config.clone();
                                sc_config.set_width(width);
                                sc_config.set_height(height);
                                
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
                                    let max = ConfigMenu::option_count().saturating_sub(1);
                                    if overlay.config_selection < max {
                                        overlay.config_selection += 1;
                                    }
                                }
                                VirtualKeyCode::Left | VirtualKeyCode::Right => {
                                    let increase = keycode == VirtualKeyCode::Right;
                                    ConfigMenu::toggle_or_adjust(&mut stream_config, &mut mic_device_idx, overlay.config_selection, increase);
                                    // Immediately apply config to running stream
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
                                            if let Err(e) = s.update_configuration(&new_config) {
                                                eprintln!("❌ Config update failed: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    // Toggle current option (same as Right arrow)
                                    ConfigMenu::toggle_or_adjust(&mut stream_config, &mut mic_device_idx, overlay.config_selection, true);
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
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
                                    let new_val = !stream_config.get_captures_microphone();
                                    stream_config.set_captures_microphone(new_val);
                                    println!("🎤 Microphone: {}", if new_val { "On" } else { "Off" });
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
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
                            mic_device_idx,
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
