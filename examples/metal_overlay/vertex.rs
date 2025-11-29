//! GPU data structures and vertex buffer

use std::mem::size_of;

use metal::*;

use crate::font::BitmapFont;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            position: [x, y],
            color: [r, g, b, a],
        }
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
    pub fn new() -> Self {
        Self { vertices: vec![] }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

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

    pub fn waveform(&mut self, samples: &[f32], x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let center_y = y + h / 2.0;
        let half_h = h / 2.0;
        
        // Draw center line with neon glow
        let glow_color = [color[0] * 0.3, color[1] * 0.3, color[2] * 0.3, 0.4];
        self.rect(x, center_y - 2.0, w, 4.0, glow_color);
        self.rect(x, center_y - 0.5, w, 1.0, [color[0] * 0.6, color[1] * 0.6, color[2] * 0.6, 0.7]);
        
        if samples.is_empty() {
            return;
        }
        
        let step = samples.len() as f32 / w;
        let bar_w = 2.0;
        for i in 0..(w as usize / bar_w as usize) {
            let sample_idx = ((i as f32 * bar_w) * step) as usize;
            let sample = samples.get(sample_idx).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
            // Minimum 2px height so bars are always visible
            let bar_h = (sample.abs() * half_h).max(2.0);
            let bar_y = if sample >= 0.0 { center_y - bar_h } else { center_y };
            
            // Draw glow behind bar
            let glow = [color[0] * 0.4, color[1] * 0.4, color[2] * 0.4, 0.3];
            self.rect(x + i as f32 * bar_w - 0.5, bar_y - 1.0, bar_w + 1.0, bar_h + 2.0, glow);
            // Main bar
            self.rect(x + i as f32 * bar_w, bar_y, bar_w - 1.0, bar_h, color);
        }
    }

    pub fn vu_meter_vertical(
        &mut self,
        level: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        font: &BitmapFont,
    ) {
        // Dark purple background
        self.rect(x, y, w, h, [0.08, 0.03, 0.12, 0.95]);
        
        let db = if level > 0.0 { 20.0 * level.log10() } else { -60.0 };
        let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let fill_h = normalized * h;
        let cyan_end = h * 0.6;
        let pink_end = h * 0.85;
        
        // Cyan (low levels)
        if fill_h > 0.0 {
            self.rect(x, y + h - fill_h.min(cyan_end), w, fill_h.min(cyan_end), [0.0, 0.9, 0.8, 1.0]);
        }
        // Purple/magenta (mid levels)
        if fill_h > cyan_end {
            self.rect(
                x,
                y + h - cyan_end - (fill_h - cyan_end).min(pink_end - cyan_end),
                w,
                (fill_h - cyan_end).min(pink_end - cyan_end),
                [0.7, 0.3, 0.9, 1.0],
            );
        }
        // Hot pink (high levels)
        if fill_h > pink_end {
            self.rect(x, y + h - pink_end - (fill_h - pink_end), w, fill_h - pink_end, [1.0, 0.2, 0.5, 1.0]);
        }
        
        // Neon border
        self.rect_outline(x, y, w, h, 1.0, [0.4, 0.2, 0.5, 0.8]);
        self.text(font, label, x + 2.0, y + h + 2.0, 1.0, [0.6, 0.5, 0.7, 1.0]);
    }

    pub fn build(&self, device: &Device) -> Buffer {
        device.new_buffer_with_data(
            self.vertices.as_ptr().cast(),
            (self.vertices.len() * size_of::<Vertex>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        )
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

impl Default for VertexBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}
