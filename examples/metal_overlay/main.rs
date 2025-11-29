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
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    pub struct WaveformBuffer {
        samples: Vec<f32>,
        write_pos: usize,
        has_received_data: AtomicBool,
        sample_count: AtomicU64,
    }

    impl WaveformBuffer {
        pub fn new(capacity: usize) -> Self {
            Self { 
                samples: vec![0.0; capacity], 
                write_pos: 0,
                has_received_data: AtomicBool::new(false),
                sample_count: AtomicU64::new(0),
            }
        }

        pub fn push(&mut self, data: &[f32]) {
            if !data.is_empty() {
                self.has_received_data.store(true, Ordering::Relaxed);
                self.sample_count.fetch_add(data.len() as u64, Ordering::Relaxed);
            }
            for &s in data {
                self.samples[self.write_pos] = s;
                self.write_pos = (self.write_pos + 1) % self.samples.len();
            }
        }

        pub fn has_data(&self) -> bool {
            self.has_received_data.load(Ordering::Relaxed)
        }
        
        pub fn sample_count(&self) -> u64 {
            self.sample_count.load(Ordering::Relaxed)
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
        
        pub fn peak(&self, count: usize) -> f32 {
            let count = count.min(self.samples.len());
            if count == 0 { return 0.0; }
            let start = (self.write_pos + self.samples.len() - count) % self.samples.len();
            (0..count).map(|i| self.samples[(start + i) % self.samples.len()].abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0)
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
        pub pixel_format: u32,
        pub _padding: [f32; 2],
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
            // Draw background with center line
            self.rect(x, y, w, h, [0.05, 0.05, 0.08, 0.7]);
            let center_y = y + h / 2.0;
            self.rect(x, center_y - 0.5, w, 1.0, [0.3, 0.3, 0.35, 0.5]); // Center line
            
            if samples.is_empty() { return; }
            let half_h = h / 2.0;
            let bar_w = 3.0;
            let gap = 1.0;
            let num_bars = (w / bar_w) as usize;
            if num_bars == 0 { return; }
            let step = samples.len() as f32 / num_bars as f32;
            
            // Calculate RMS for each bar segment for smoother display
            for i in 0..num_bars {
                let start_idx = (i as f32 * step) as usize;
                let end_idx = ((i + 1) as f32 * step) as usize;
                let segment = &samples[start_idx.min(samples.len() - 1)..end_idx.min(samples.len())];
                
                // Use peak value in segment (more responsive than RMS)
                let peak = segment.iter().map(|s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
                
                // Aggressive amplification for visibility (audio is typically quiet)
                let amplified = (peak * 8.0).clamp(0.0, 1.0);
                let bar_h = amplified * half_h;
                
                // Always show at least a small bar if there's any signal
                if bar_h > 0.1 {
                    let bar_x = x + i as f32 * bar_w;
                    // Draw symmetric bars from center
                    self.rect(bar_x, center_y - bar_h, bar_w - gap, bar_h * 2.0, color);
                }
            }
        }

        pub fn vu_meter_vertical(&mut self, level: f32, x: f32, y: f32, w: f32, h: f32, label: &str, font: &BitmapFont) {
            self.rect(x, y, w, h, [0.1, 0.1, 0.1, 0.9]);
            // More sensitive dB conversion for quiet signals
            let db = if level > 0.0001 { 20.0 * level.log10() } else { -80.0 };
            // Map -60dB to 0dB range with more sensitivity at lower levels
            let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
            // Apply a curve to make quiet signals more visible
            let curved = normalized.sqrt();
            let fill_h = curved * h;
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
        pub const MENU_ITEMS: &'static [&'static str] = &["Picker", "Capture", "Config", "Quit"];
        pub fn menu_count() -> usize { Self::MENU_ITEMS.len() }
    }

    pub struct ConfigMenu;
    impl ConfigMenu {
        pub const OPTIONS: &'static [&'static str] = &[
            "FPS",
            "Cursor", 
            "Audio",
            "Mic",
            "Mic Device",
            "Exclude Self",
            "Sample Rate",
            "Channels",
            "Scale Fit",
            "Aspect",
            "Retina",
            "Opaque",
            "Shadows Only",
            "Ignore Shadows",
            "Pixel Format",
            "Queue",
        ];
        pub const FPS_OPTIONS: [u32; 5] = [15, 30, 60, 120, 240];
        pub const QUEUE_OPTIONS: [u32; 4] = [3, 5, 8, 12];
        pub const SAMPLE_RATE_OPTIONS: [i32; 3] = [44100, 48000, 96000];
        pub const CHANNEL_OPTIONS: [i32; 2] = [1, 2];
        pub fn option_count() -> usize { Self::OPTIONS.len() }
        pub fn option_name(idx: usize) -> &'static str { Self::OPTIONS.get(idx).unwrap_or(&"?") }

        pub fn option_value(config: &SCStreamConfiguration, mic_device_idx: Option<usize>, idx: usize) -> String {
            match idx {
                0 => format!("{}", config.fps()),
                1 => if config.shows_cursor() { "On" } else { "Off" }.to_string(),
                2 => if config.captures_audio() { "On" } else { "Off" }.to_string(),
                3 => if config.captures_microphone() { "On" } else { "Off" }.to_string(),
                4 => match mic_device_idx {
                    None => "Default".to_string(),
                    Some(idx) => AudioInputDevice::list().get(idx).map(|d| d.name.chars().take(10).collect()).unwrap_or_else(|| "?".to_string()),
                },
                5 => if config.excludes_current_process_audio() { "On" } else { "Off" }.to_string(),
                6 => format!("{}Hz", config.sample_rate()),
                7 => format!("{}ch", config.channel_count()),
                8 => if config.scales_to_fit() { "On" } else { "Off" }.to_string(),
                9 => if config.preserves_aspect_ratio() { "On" } else { "Off" }.to_string(),
                10 => if config.increase_resolution_for_retina_displays() { "On" } else { "Off" }.to_string(),
                11 => if config.should_be_opaque() { "On" } else { "Off" }.to_string(),
                12 => if config.captures_shadows_only() { "On" } else { "Off" }.to_string(),
                13 => if config.ignores_shadows_display() { "On" } else { "Off" }.to_string(),
                14 => format!("{:?}", config.pixel_format()),
                15 => format!("{}", config.queue_depth()),
                _ => "?".to_string(),
            }
        }

        pub fn toggle_or_adjust(config: &mut SCStreamConfiguration, mic_device_idx: &mut Option<usize>, idx: usize, increase: bool) {
            use screencapturekit::stream::configuration::pixel_format::PixelFormat;
            const PIXEL_FORMATS: [PixelFormat; 4] = [PixelFormat::BGRA, PixelFormat::l10r, PixelFormat::YCbCr_420v, PixelFormat::YCbCr_420f];
            
            match idx {
                0 => { // FPS
                    let current_fps = config.fps();
                    let current_idx = Self::FPS_OPTIONS.iter().position(|&f| f == current_fps).unwrap_or(2);
                    let new_idx = if increase { (current_idx + 1) % Self::FPS_OPTIONS.len() } else { (current_idx + Self::FPS_OPTIONS.len() - 1) % Self::FPS_OPTIONS.len() };
                    config.set_fps(Self::FPS_OPTIONS[new_idx]);
                }
                1 => { config.set_shows_cursor(!config.shows_cursor()); }
                2 => { config.set_captures_audio(!config.captures_audio()); }
                3 => { config.set_captures_microphone(!config.captures_microphone()); }
                4 => { // Mic Device
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
                5 => { config.set_excludes_current_process_audio(!config.excludes_current_process_audio()); }
                6 => { // Sample Rate
                    let current = config.sample_rate();
                    let current_idx = Self::SAMPLE_RATE_OPTIONS.iter().position(|&r| r == current).unwrap_or(1);
                    let new_idx = if increase { (current_idx + 1) % Self::SAMPLE_RATE_OPTIONS.len() } else { (current_idx + Self::SAMPLE_RATE_OPTIONS.len() - 1) % Self::SAMPLE_RATE_OPTIONS.len() };
                    config.set_sample_rate(Self::SAMPLE_RATE_OPTIONS[new_idx]);
                }
                7 => { // Channels
                    let current = config.channel_count();
                    let current_idx = Self::CHANNEL_OPTIONS.iter().position(|&c| c == current).unwrap_or(1);
                    let new_idx = if increase { (current_idx + 1) % Self::CHANNEL_OPTIONS.len() } else { (current_idx + Self::CHANNEL_OPTIONS.len() - 1) % Self::CHANNEL_OPTIONS.len() };
                    config.set_channel_count(Self::CHANNEL_OPTIONS[new_idx]);
                }
                8 => { config.set_scales_to_fit(!config.scales_to_fit()); }
                9 => { config.set_preserves_aspect_ratio(!config.preserves_aspect_ratio()); }
                10 => { config.set_increase_resolution_for_retina_displays(!config.increase_resolution_for_retina_displays()); }
                11 => { config.set_should_be_opaque(!config.should_be_opaque()); }
                12 => { config.set_captures_shadows_only(!config.captures_shadows_only()); }
                13 => { config.set_ignores_shadows_display(!config.ignores_shadows_display()); }
                14 => { // Pixel Format
                    let current = config.pixel_format();
                    let current_idx = PIXEL_FORMATS.iter().position(|&f| f == current).unwrap_or(0);
                    let new_idx = if increase { (current_idx + 1) % PIXEL_FORMATS.len() } else { (current_idx + PIXEL_FORMATS.len() - 1) % PIXEL_FORMATS.len() };
                    config.set_pixel_format(PIXEL_FORMATS[new_idx]);
                }
                15 => { // Queue
                    let current_depth = config.queue_depth();
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
                    // Get audio samples from audio_buffer_list
                    if let Some(audio_buffer_list) = sample.audio_buffer_list() {
                        for buffer in audio_buffer_list.iter() {
                            let data = buffer.data();
                            if data.is_empty() {
                                continue;
                            }
                            let audio_samples: Vec<f32> = data
                                .chunks_exact(4)
                                .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
                                .collect();
                            
                            if !audio_samples.is_empty() {
                                let waveform = if matches!(output_type, SCStreamOutputType::Audio) { &self.state.audio_waveform } else { &self.state.mic_waveform };
                                waveform.lock().unwrap().push(&audio_samples);
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
        fn IOSurfaceGetPixelFormat(surface: *const c_void) -> u32;
        fn IOSurfaceGetPlaneCount(surface: *const c_void) -> usize;
        fn IOSurfaceGetWidthOfPlane(surface: *const c_void, plane: usize) -> usize;
        fn IOSurfaceGetHeightOfPlane(surface: *const c_void, plane: usize) -> usize;
    }
    
    // Pixel format constants (FourCC codes)
    pub const PIXEL_FORMAT_BGRA: u32 = 0x42475241; // 'BGRA'
    pub const PIXEL_FORMAT_L10R: u32 = 0x6C313072; // 'l10r' - ARGB2101010
    pub const PIXEL_FORMAT_420V: u32 = 0x34323076; // '420v' - YCbCr 420 video range
    pub const PIXEL_FORMAT_420F: u32 = 0x34323066; // '420f' - YCbCr 420 full range

    pub struct CaptureTextures {
        pub plane0: Texture,           // Y plane for YCbCr, or BGRA texture
        pub plane1: Option<Texture>,   // CbCr plane for YCbCr formats
        pub pixel_format: u32,
        pub width: usize,
        pub height: usize,
    }

    pub unsafe fn create_textures_from_iosurface(device: &Device, iosurface_ptr: *const c_void) -> Option<CaptureTextures> {
        if iosurface_ptr.is_null() { return None; }
        let width = IOSurfaceGetWidth(iosurface_ptr);
        let height = IOSurfaceGetHeight(iosurface_ptr);
        let pixel_format = IOSurfaceGetPixelFormat(iosurface_ptr);
        let plane_count = IOSurfaceGetPlaneCount(iosurface_ptr);
        
        if width == 0 || height == 0 { return None; }
        
        // Determine Metal pixel format and create appropriate textures
        match pixel_format {
            PIXEL_FORMAT_BGRA => {
                // Single plane BGRA
                let desc = TextureDescriptor::new();
                desc.set_texture_type(MTLTextureType::D2);
                desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
                desc.set_width(width as u64);
                desc.set_height(height as u64);
                desc.set_storage_mode(MTLStorageMode::Shared);
                desc.set_usage(MTLTextureUsage::ShaderRead);
                let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, 
                    newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object 
                    iosurface: iosurface_ptr plane: 0usize];
                if texture.is_null() { return None; }
                Some(CaptureTextures {
                    plane0: Texture::from_ptr(texture),
                    plane1: None,
                    pixel_format,
                    width,
                    height,
                })
            }
            PIXEL_FORMAT_L10R => {
                // 10-bit ARGB2101010
                let desc = TextureDescriptor::new();
                desc.set_texture_type(MTLTextureType::D2);
                desc.set_pixel_format(MTLPixelFormat::BGR10A2Unorm);
                desc.set_width(width as u64);
                desc.set_height(height as u64);
                desc.set_storage_mode(MTLStorageMode::Shared);
                desc.set_usage(MTLTextureUsage::ShaderRead);
                let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, 
                    newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object 
                    iosurface: iosurface_ptr plane: 0usize];
                if texture.is_null() { return None; }
                Some(CaptureTextures {
                    plane0: Texture::from_ptr(texture),
                    plane1: None,
                    pixel_format,
                    width,
                    height,
                })
            }
            PIXEL_FORMAT_420V | PIXEL_FORMAT_420F => {
                // YCbCr 4:2:0 - two planes: Y (R8) and CbCr (RG8)
                if plane_count < 2 { return None; }
                
                // Plane 0: Y (luminance)
                let y_width = IOSurfaceGetWidthOfPlane(iosurface_ptr, 0);
                let y_height = IOSurfaceGetHeightOfPlane(iosurface_ptr, 0);
                let y_desc = TextureDescriptor::new();
                y_desc.set_texture_type(MTLTextureType::D2);
                y_desc.set_pixel_format(MTLPixelFormat::R8Unorm);
                y_desc.set_width(y_width as u64);
                y_desc.set_height(y_height as u64);
                y_desc.set_storage_mode(MTLStorageMode::Shared);
                y_desc.set_usage(MTLTextureUsage::ShaderRead);
                let y_texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, 
                    newTextureWithDescriptor: y_desc.as_ptr() as *mut objc::runtime::Object 
                    iosurface: iosurface_ptr plane: 0usize];
                if y_texture.is_null() { return None; }
                
                // Plane 1: CbCr (chroma)
                let uv_width = IOSurfaceGetWidthOfPlane(iosurface_ptr, 1);
                let uv_height = IOSurfaceGetHeightOfPlane(iosurface_ptr, 1);
                let uv_desc = TextureDescriptor::new();
                uv_desc.set_texture_type(MTLTextureType::D2);
                uv_desc.set_pixel_format(MTLPixelFormat::RG8Unorm);
                uv_desc.set_width(uv_width as u64);
                uv_desc.set_height(uv_height as u64);
                uv_desc.set_storage_mode(MTLStorageMode::Shared);
                uv_desc.set_usage(MTLTextureUsage::ShaderRead);
                let uv_texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, 
                    newTextureWithDescriptor: uv_desc.as_ptr() as *mut objc::runtime::Object 
                    iosurface: iosurface_ptr plane: 1usize];
                if uv_texture.is_null() { return None; }
                
                Some(CaptureTextures {
                    plane0: Texture::from_ptr(y_texture),
                    plane1: Some(Texture::from_ptr(uv_texture)),
                    pixel_format,
                    width,
                    height,
                })
            }
            _ => {
                // Unknown format - try as BGRA
                eprintln!("Unknown pixel format: 0x{:08x}, trying as BGRA", pixel_format);
                let desc = TextureDescriptor::new();
                desc.set_texture_type(MTLTextureType::D2);
                desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
                desc.set_width(width as u64);
                desc.set_height(height as u64);
                desc.set_storage_mode(MTLStorageMode::Shared);
                desc.set_usage(MTLTextureUsage::ShaderRead);
                let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object, 
                    newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object 
                    iosurface: iosurface_ptr plane: 0usize];
                if texture.is_null() { return None; }
                Some(CaptureTextures {
                    plane0: Texture::from_ptr(texture),
                    plane1: None,
                    pixel_format: PIXEL_FORMAT_BGRA,
                    width,
                    height,
                })
            }
        }
    }

    // Keep old function for compatibility
    #[allow(dead_code)]
    pub unsafe fn create_texture_from_iosurface(device: &Device, iosurface_ptr: *const c_void) -> Option<Texture> {
        create_textures_from_iosurface(device, iosurface_ptr).map(|t| t.plane0)
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
struct Uniforms { float2 viewport_size; float2 texture_size; float time; uint pixel_format; float padding[2]; };
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
// BGRA/RGB texture fragment shader
fragment float4 fragment_textured(TexturedVertexOut in [[stage_in]], texture2d<float> tex [[texture(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear); return tex.sample(s, in.texcoord);
}
// YCbCr to RGB conversion (BT.709 matrix for HD video)
float4 ycbcr_to_rgb(float y, float2 cbcr, bool full_range) {
    // Adjust for video vs full range
    float y_adj = full_range ? y : (y - 16.0/255.0) * (255.0/219.0);
    float cb = cbcr.x - 0.5;
    float cr = cbcr.y - 0.5;
    // BT.709 conversion matrix
    float r = y_adj + 1.5748 * cr;
    float g = y_adj - 0.1873 * cb - 0.4681 * cr;
    float b = y_adj + 1.8556 * cb;
    return float4(saturate(float3(r, g, b)), 1.0);
}
// YCbCr biplanar (420v/420f) fragment shader
fragment float4 fragment_ycbcr(TexturedVertexOut in [[stage_in]], 
    texture2d<float> y_tex [[texture(0)]], 
    texture2d<float> cbcr_tex [[texture(1)]],
    constant Uniforms& uniforms [[buffer(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    float y = y_tex.sample(s, in.texcoord).r;
    float2 cbcr = cbcr_tex.sample(s, in.texcoord).rg;
    bool full_range = (uniforms.pixel_format == 0x34323066); // '420f'
    return ycbcr_to_rgb(y, cbcr, full_range);
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
        // Synthwave color constants
        const NEON_PINK: [f32; 4] = [1.0, 0.2, 0.6, 1.0];
        const NEON_CYAN: [f32; 4] = [0.0, 1.0, 0.9, 1.0];
        const NEON_PURPLE: [f32; 4] = [0.7, 0.3, 1.0, 1.0];
        const NEON_YELLOW: [f32; 4] = [1.0, 0.95, 0.3, 1.0];
        const DARK_BG: [f32; 4] = [0.04, 0.02, 0.08, 0.95];
        
        pub fn help_overlay(&mut self, font: &BitmapFont, vw: f32, vh: f32, is_capturing: bool, source_name: &str, menu_selection: usize) {
            let base_scale = (vw.min(vh) / 800.0).clamp(0.8, 2.0);
            let scale = 1.5 * base_scale;
            let line_h = 18.0 * base_scale;
            let padding = 16.0 * base_scale;
            let has_source = !source_name.is_empty() && source_name != "None";
            // Menu values: Picker(Open), Capture(Start/Stop), Config(Open), Quit(empty)
            let menu_values: [&str; 4] = [
                "Open",  // Picker
                if is_capturing { "Stop" } else { "Start" },  // Capture (works even without source for mic-only)
                "Open",  // Config
                ""       // Quit
            ];
            
            let box_w = (320.0 * base_scale).min(vw * 0.8);
            let box_h = (line_h * 7.5 + padding * 2.0).min(vh * 0.7);
            let x = (vw - box_w) / 2.0;
            let y = (vh - box_h) / 2.0;
            
            // Source name as large centered title above the menu
            let source_display = if has_source {
                if source_name.len() > 30 { format!("{}...", &source_name.chars().take(27).collect::<String>()) } else { source_name.to_string() }
            } else {
                "No Source Selected".to_string()
            };
            let title_scale = scale * 1.4;
            let title_actual = (title_scale as i32) as f32;
            let title_w = source_display.len() as f32 * 8.0 * title_actual;
            let title_x = (vw - title_w) / 2.0;
            let title_y = y - line_h * 2.2;
            let title_color = if has_source { Self::NEON_CYAN } else { [0.5, 0.4, 0.6, 1.0] };
            self.text(font, &source_display, title_x, title_y, title_scale, title_color);
            
            // Dark purple background with neon border
            self.rect(x, y, box_w, box_h, Self::DARK_BG);
            self.rect_outline(x, y, box_w, box_h, 2.0, Self::NEON_PINK);
            self.rect_outline(x + 1.0, y + 1.0, box_w - 2.0, box_h - 2.0, 1.0, [0.3, 0.1, 0.4, 0.5]);
            
            let mut ly = y + padding;
            let text_x = x + padding + 12.0 * base_scale;
            
            let actual_scale = (scale as i32) as f32;
            let text_h = 8.0 * actual_scale;
            
            for (i, (item, value)) in OverlayState::MENU_ITEMS.iter().zip(menu_values.iter()).enumerate() {
                let is_selected = i == menu_selection;
                let text_y = ly + (line_h - text_h) / 2.0;
                
                if is_selected {
                    // Selection highlight - purple glow
                    self.rect(x + 3.0, ly, box_w - 6.0, line_h, [0.15, 0.05, 0.25, 0.9]);
                    self.rect(x + 3.0, ly, 2.0, line_h, Self::NEON_PINK);
                    self.text(font, ">", x + padding * 0.5, text_y, scale, Self::NEON_YELLOW);
                }
                
                let item_color = if is_selected {
                    Self::NEON_CYAN
                } else {
                    [0.8, 0.8, 0.9, 1.0]
                };
                
                self.text(font, item, text_x, text_y, scale, item_color);
                
                if !value.is_empty() {
                    let vx = x + box_w - padding - value.len() as f32 * 8.0 * actual_scale;
                    let val_color = if is_selected { Self::NEON_YELLOW } else { [0.5, 0.5, 0.6, 1.0] };
                    self.text(font, value, vx, text_y, scale, val_color);
                }
                ly += line_h;
            }
            
            // Footer
            ly += line_h * 0.2;
            self.rect(x + padding, ly, box_w - padding * 2.0, 1.0, [0.3, 0.15, 0.4, 0.4]);
            ly += line_h * 0.4;
            self.text(font, "ARROWS  ENTER  ESC", text_x, ly, scale * 0.6, [0.5, 0.4, 0.6, 1.0]);
        }

        pub fn config_menu(&mut self, font: &BitmapFont, vw: f32, vh: f32, config: &SCStreamConfiguration, mic_device_idx: Option<usize>, selection: usize, is_capturing: bool, source_name: &str) {
            let base_scale = (vw.min(vh) / 800.0).clamp(0.8, 2.0);
            let scale = 1.5 * base_scale;
            let line_h = 18.0 * base_scale;
            let padding = 16.0 * base_scale;
            let option_count = ConfigMenu::option_count();
            let box_w = (340.0 * base_scale).min(vw * 0.85);
            let box_h = (line_h * (option_count as f32 + 5.0) + padding * 2.0).min(vh * 0.8);
            let x = (vw - box_w) / 2.0;
            let y = (vh - box_h) / 2.0;
            
            // Dark purple background with neon border
            self.rect(x, y, box_w, box_h, Self::DARK_BG);
            self.rect_outline(x, y, box_w, box_h, 2.0, Self::NEON_CYAN);
            self.rect_outline(x + 1.0, y + 1.0, box_w - 2.0, box_h - 2.0, 1.0, [0.1, 0.3, 0.4, 0.5]);
            
            let mut ly = y + padding;
            let text_x = x + padding + 12.0 * base_scale;
            
            // Source heading (larger, centered)
            let source_display = if source_name.is_empty() || source_name == "None" { "No Source" } else { source_name };
            let source_w = source_display.len() as f32 * 8.0 * scale;
            let source_x = x + (box_w - source_w) / 2.0;
            self.text(font, source_display, source_x, ly, scale * 1.1, Self::NEON_YELLOW);
            ly += line_h * 1.5;
            
            // Separator line
            self.rect(x + padding, ly - 4.0, box_w - padding * 2.0, 1.0, Self::NEON_PURPLE);
            ly += line_h * 0.3;
            
            // Title row with live indicator
            self.text(font, "CONFIG", text_x - 4.0, ly, scale * 0.8, Self::NEON_PINK);
            
            // Live indicator
            if is_capturing {
                let live_x = x + box_w - padding - 32.0 * base_scale;
                self.rect(live_x - 3.0, ly - 1.0, 38.0 * base_scale, line_h * 0.9, [0.5, 0.1, 0.15, 0.9]);
                self.text(font, "LIVE", live_x, ly, scale * 0.7, [1.0, 0.3, 0.3, 1.0]);
            }
            
            ly += line_h * 1.0;
            
            let actual_scale = (scale as i32) as f32;
            let text_h = 8.0 * actual_scale;
            
            for i in 0..option_count {
                let is_selected = i == selection;
                let text_y = ly + (line_h - text_h) / 2.0;
                
                if is_selected {
                    self.rect(x + 3.0, ly, box_w - 6.0, line_h, [0.1, 0.05, 0.2, 0.9]);
                    self.rect(x + 3.0, ly, 2.0, line_h, Self::NEON_CYAN);
                    self.text(font, ">", x + padding * 0.5, text_y, scale, Self::NEON_YELLOW);
                }
                
                let name = ConfigMenu::option_name(i);
                let value = ConfigMenu::option_value(config, mic_device_idx, i);
                
                let name_color = if is_selected { [1.0, 1.0, 1.0, 1.0] } else { [0.7, 0.7, 0.8, 1.0] };
                self.text(font, name, text_x, text_y, scale, name_color);
                
                let t: String = if value.len() > 12 { format!("{}...", &value.chars().take(9).collect::<String>()) } else { value };
                let vx = x + box_w - padding - t.len() as f32 * 8.0 * actual_scale;
                
                let value_color = if is_selected {
                    if t == "On" { [0.3, 1.0, 0.5, 1.0] } else if t == "Off" { [1.0, 0.4, 0.4, 1.0] } else { Self::NEON_YELLOW }
                } else {
                    if t == "On" { [0.2, 0.7, 0.4, 1.0] } else if t == "Off" { [0.5, 0.3, 0.3, 1.0] } else { [0.5, 0.5, 0.6, 1.0] }
                };
                self.text(font, &t, vx, text_y, scale, value_color);
                ly += line_h;
            }
            
            // Footer
            ly += line_h * 0.2;
            self.rect(x + padding, ly, box_w - padding * 2.0, 1.0, [0.3, 0.15, 0.4, 0.4]);
            ly += line_h * 0.4;
            let hint = if is_capturing { "L/R  ENTER=Apply  ESC" } else { "LEFT/RIGHT  ESC" };
            self.text(font, hint, text_x, ly, scale * 0.6, [0.5, 0.4, 0.6, 1.0]);
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
use renderer::{create_pipeline, create_textures_from_iosurface, CaptureTextures, SHADER_SOURCE, PIXEL_FORMAT_420V, PIXEL_FORMAT_420F};
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
    
    // Create fullscreen textured pipeline (no blending for background) - for BGRA/RGB formats
    let fullscreen_pipeline = {
        let vert = library.get_function("vertex_fullscreen", None).unwrap();
        let frag = library.get_function("fragment_textured", None).unwrap();
        let desc = RenderPipelineDescriptor::new();
        desc.set_vertex_function(Some(&vert));
        desc.set_fragment_function(Some(&frag));
        desc.color_attachments().object_at(0).unwrap().set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        device.new_render_pipeline_state(&desc).unwrap()
    };
    
    // Create YCbCr pipeline for biplanar YCbCr formats (420v/420f)
    let ycbcr_pipeline = {
        let vert = library.get_function("vertex_fullscreen", None).unwrap();
        let frag = library.get_function("fragment_ycbcr", None).unwrap();
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
            
            // Check for pending picker results - update filter if capturing, otherwise just store
            if let Ok(mut pending) = pending_picker.try_lock() {
                if let Some((filter, width, height, source)) = pending.take() {
                    println!("✅ Content selected: {}x{} - {}", width, height, format_picked_source(&source));
                    capture_size = (width, height);
                    picked_source = source;
                    
                    // If already capturing, update the filter live
                    if capturing.load(Ordering::Relaxed) {
                        if let Some(ref s) = stream {
                            match s.update_content_filter(&filter) {
                                Ok(()) => println!("✅ Source updated live"),
                                Err(e) => eprintln!("❌ Failed to update source: {:?}", e),
                            }
                        }
                    }
                    current_filter = Some(filter);
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
                        // Helper closure to open picker (without stream - for initial selection)
                        let open_picker_no_stream = |pending_picker: &Arc<Mutex<PickerResult>>| {
                            println!("📺 Opening content picker...");
                            let mut config = SCContentSharingPickerConfiguration::new();
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
                        
                        // Helper closure to open picker for existing stream
                        let open_picker_for_stream = |pending_picker: &Arc<Mutex<PickerResult>>, stream: &SCStream| {
                            println!("📺 Opening content picker for stream...");
                            let mut config = SCContentSharingPickerConfiguration::new();
                            config.set_allowed_picker_modes(&[
                                SCContentSharingPickerMode::SingleWindow,
                                SCContentSharingPickerMode::MultipleWindows,
                                SCContentSharingPickerMode::SingleDisplay,
                                SCContentSharingPickerMode::SingleApplication,
                                SCContentSharingPickerMode::MultipleApplications,
                            ]);
                            let pending = Arc::clone(pending_picker);
                            
                            SCContentSharingPicker::show_for_stream(&config, stream, move |outcome| {
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
                                            capturing: &Arc<AtomicBool>,
                                            mic_only: bool| {
                            // Get the filter to use
                            let filter_to_use = if let Some(ref filter) = current_filter {
                                filter.clone()
                            } else if mic_only {
                                // For mic-only capture, we still need a valid display filter
                                // macOS requires a content filter even for audio-only capture
                                println!("🎤 Starting mic-only capture (using main display)");
                                match screencapturekit::shareable_content::SCShareableContent::get() {
                                    Ok(content) => {
                                        let displays = content.displays();
                                        if let Some(display) = displays.first() {
                                            SCContentFilter::builder().display(display).build()
                                        } else {
                                            println!("❌ No displays available for mic-only capture");
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ Failed to get shareable content: {:?}", e);
                                        return;
                                    }
                                }
                            } else {
                                println!("⚠️  No content selected. Open picker first.");
                                return;
                            };
                            
                            let (width, height) = capture_size;
                            // Clone config and update dimensions
                            let mut sc_config = stream_config.clone();
                            sc_config.set_width(width);
                            sc_config.set_height(height);
                            
                            let handler = CaptureHandler {
                                state: Arc::clone(capture_state),
                            };

                            let mut s = SCStream::new(&filter_to_use, &sc_config);
                            if !mic_only {
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Screen);
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Audio);
                            }
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
                                        println!("⬆️  Menu selection: {} ({})", overlay.menu_selection, OverlayState::MENU_ITEMS[overlay.menu_selection]);
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    let max = OverlayState::menu_count().saturating_sub(1);
                                    if overlay.menu_selection < max {
                                        overlay.menu_selection += 1;
                                        println!("⬇️  Menu selection: {} ({})", overlay.menu_selection, OverlayState::MENU_ITEMS[overlay.menu_selection]);
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    match overlay.menu_selection {
                                        0 => { // Picker
                                            if let Some(ref s) = stream {
                                                open_picker_for_stream(&pending_picker, s);
                                            } else {
                                                open_picker_no_stream(&pending_picker);
                                            }
                                        }
                                        1 => { // Capture start/stop (works with or without source)
                                            if capturing.load(Ordering::Relaxed) {
                                                stop_capture(&mut stream, &capturing);
                                            } else {
                                                // If we have a filter, use it; otherwise capture mic-only
                                                let mic_only = current_filter.is_none();
                                                start_capture(&mut stream, &current_filter, capture_size, &stream_config, &capture_state, &capturing, mic_only);
                                            }
                                        }
                                        2 => { // Config
                                            overlay.show_config = true;
                                            overlay.show_help = false;
                                        }
                                        3 => { // Quit
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
                                        start_capture(&mut stream, &current_filter, capture_size, &stream_config, &capture_state, &capturing, false);
                                    }
                                }
                                VirtualKeyCode::P => {
                                    if let Some(ref s) = stream {
                                        open_picker_for_stream(&pending_picker, s);
                                    } else {
                                        open_picker_no_stream(&pending_picker);
                                    }
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
                                    let new_val = !stream_config.captures_microphone();
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

                    // Try to get the latest IOSurface and create textures from it (zero-copy)
                    let mut capture_textures: Option<CaptureTextures> = None;
                    let mut tex_width = capture_size.0 as f32;
                    let mut tex_height = capture_size.1 as f32;
                    let mut pixel_format: u32 = 0;
                    
                    if capturing.load(Ordering::Relaxed) {
                        if let Ok(guard) = capture_state.latest_surface.try_lock() {
                            if let Some(ref surface) = *guard {
                                tex_width = surface.width() as f32;
                                tex_height = surface.height() as f32;
                                // Create Metal textures directly from IOSurface (zero-copy)
                                capture_textures = unsafe {
                                    create_textures_from_iosurface(&device, surface.as_ptr())
                                };
                                if let Some(ref ct) = capture_textures {
                                    pixel_format = ct.pixel_format;
                                }
                            }
                        }
                    }

                    // Build vertex buffer for this frame
                    vertex_builder.clear();

                    // Status bar background
                    vertex_builder.rect(0.0, 0.0, width, 32.0, [0.1, 0.1, 0.12, 0.9]);

                    // Status text - include audio sample counts and peaks for debugging
                    let fps = capture_state.frame_count.load(Ordering::Relaxed);
                    let audio_samples_cnt = capture_state.audio_waveform.lock().unwrap().sample_count();
                    let mic_samples_cnt = capture_state.mic_waveform.lock().unwrap().sample_count();
                    let audio_peak = capture_state.audio_waveform.lock().unwrap().peak(512);
                    let mic_peak = capture_state.mic_waveform.lock().unwrap().peak(512);
                    let status = if capture_textures.is_some() {
                        format!("LIVE {}x{} F:{} A:{}k P:{:.2} M:{}k P:{:.2}", 
                            tex_width as u32, tex_height as u32, fps, 
                            audio_samples_cnt / 1000, audio_peak,
                            mic_samples_cnt / 1000, mic_peak)
                    } else if capturing.load(Ordering::Relaxed) {
                        format!("Starting... {}", fps)
                    } else {
                        "H=Menu".to_string()
                    };
                    vertex_builder.text(&font, &status, 8.0, 8.0, 2.0, [0.2, 1.0, 0.3, 1.0]);

                    // Waveform bar at top - 100% width with both system audio and mic
                    if overlay.show_waveform && capturing.load(Ordering::Relaxed) {
                        let single_wave_h = 40.0;
                        let wave_spacing = 4.0;
                        let total_wave_h = single_wave_h * 2.0 + wave_spacing;
                        let bar_y = 36.0; // Below status bar
                        let meter_w = 24.0;
                        let padding = 8.0;
                        let label_w = 24.0; // Space for labels
                        
                        // Waveform background - full width
                        vertex_builder.rect(0.0, bar_y, width, total_wave_h + 12.0, [0.08, 0.08, 0.1, 0.9]);
                        
                        // Calculate waveform area (leave space for labels on left and meters on right)
                        let meters_space = meter_w * 2.0 + padding * 3.0;
                        let wave_w = width - meters_space - padding - label_w;
                        let wave_x = padding + label_w;
                        
                        // System audio waveform (top) - cyan/green
                        let audio_wave_y = bar_y + 4.0;
                        vertex_builder.text(&font, "SYS", padding, audio_wave_y + single_wave_h / 2.0 - 4.0, 1.0, [0.0, 0.9, 0.8, 0.7]);
                        let audio_samples = capture_state.audio_waveform.lock().unwrap().display_samples(512);
                        vertex_builder.waveform(
                            &audio_samples,
                            wave_x,
                            audio_wave_y,
                            wave_w,
                            single_wave_h,
                            [0.0, 0.9, 0.8, 0.9], // Cyan (system audio)
                        );

                        // Microphone waveform (bottom) - magenta/pink
                        let mic_wave_y = audio_wave_y + single_wave_h + wave_spacing;
                        vertex_builder.text(&font, "MIC", padding, mic_wave_y + single_wave_h / 2.0 - 4.0, 1.0, [1.0, 0.3, 0.7, 0.7]);
                        let mic_samples = capture_state.mic_waveform.lock().unwrap().display_samples(512);
                        vertex_builder.waveform(
                            &mic_samples,
                            wave_x,
                            mic_wave_y,
                            wave_w,
                            single_wave_h,
                            [1.0, 0.3, 0.7, 0.9], // Magenta (mic)
                        );

                        // Vertical meters on the right
                        let meters_x = width - meters_space + padding;
                        
                        // System audio vertical meter
                        let audio_level = capture_state.audio_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            audio_level,
                            meters_x,
                            audio_wave_y,
                            meter_w,
                            total_wave_h,
                            "S",
                            &font,
                        );

                        // Microphone vertical meter
                        let mic_level = capture_state.mic_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            mic_level,
                            meters_x + meter_w + padding,
                            audio_wave_y,
                            meter_w,
                            total_wave_h,
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
                        let source_str = format_picked_source(&picked_source);
                        vertex_builder.config_menu(
                            &font,
                            width,
                            height,
                            &stream_config,
                            mic_device_idx,
                            overlay.config_selection,
                            capturing.load(Ordering::Relaxed),
                            &source_str,
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
                        pixel_format,
                        _padding: [0.0; 2],
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
                    if let Some(ref textures) = capture_textures {
                        let is_ycbcr = textures.pixel_format == PIXEL_FORMAT_420V || textures.pixel_format == PIXEL_FORMAT_420F;
                        
                        if is_ycbcr && textures.plane1.is_some() {
                            // Use YCbCr pipeline for biplanar formats
                            encoder.set_render_pipeline_state(&ycbcr_pipeline);
                            encoder.set_vertex_buffer(0, Some(&uniforms_buffer), 0);
                            encoder.set_fragment_texture(0, Some(&textures.plane0));
                            encoder.set_fragment_texture(1, Some(textures.plane1.as_ref().unwrap()));
                            encoder.set_fragment_buffer(0, Some(&uniforms_buffer), 0);
                        } else {
                            // Use standard BGRA/RGB pipeline
                            encoder.set_render_pipeline_state(&fullscreen_pipeline);
                            encoder.set_vertex_buffer(0, Some(&uniforms_buffer), 0);
                            encoder.set_fragment_texture(0, Some(&textures.plane0));
                        }
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
