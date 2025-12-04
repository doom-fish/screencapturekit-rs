//! GPU data structures and vertex buffer

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_arguments
)]

use std::mem::size_of;

use screencapturekit::output::metal::{MetalBuffer, MetalDevice, ResourceOptions};

use crate::font::BitmapFont;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    #[allow(clippy::many_single_char_names)]
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
    pub pixel_format: u32,
    pub _padding: [f32; 2],
}

pub struct VertexBufferBuilder {
    vertices: Vec<Vertex>,
}

impl VertexBufferBuilder {
    pub const fn new() -> Self {
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

    pub fn rect_outline(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        self.rect(x, y, w, thickness, color);
        self.rect(x, y + h - thickness, w, thickness, color);
        self.rect(x, y, thickness, h, color);
        self.rect(x + w - thickness, y, thickness, h, color);
    }

    pub fn text(
        &mut self,
        font: &BitmapFont,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: [f32; 4],
    ) {
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
        // Draw background with center line
        self.rect(x, y, w, h, [0.05, 0.05, 0.08, 0.7]);
        let center_y = y + h / 2.0;
        self.rect(x, center_y - 0.5, w, 1.0, [0.3, 0.3, 0.35, 0.5]); // Center line

        if samples.is_empty() {
            return;
        }
        let half_h = h / 2.0;
        let bar_w = 3.0;
        let gap = 1.0;
        let num_bars = (w / bar_w) as usize;
        if num_bars == 0 {
            return;
        }
        let step = samples.len() as f32 / num_bars as f32;

        // Calculate RMS for each bar segment for smoother display
        for i in 0..num_bars {
            let start_idx = (i as f32 * step) as usize;
            let end_idx = ((i + 1) as f32 * step) as usize;
            let segment = &samples[start_idx.min(samples.len() - 1)..end_idx.min(samples.len())];

            // Use peak value in segment (more responsive than RMS)
            let peak = segment
                .iter()
                .map(|s| s.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            // Aggressive amplification for visibility (audio is typically quiet)
            let amplified = (peak * 8.0).clamp(0.0, 1.0);
            let bar_h = amplified * half_h;

            // Always show at least a small bar if there's any signal
            if bar_h > 0.1 {
                let bar_x = (i as f32).mul_add(bar_w, x);
                // Draw symmetric bars from center
                self.rect(bar_x, center_y - bar_h, bar_w - gap, bar_h * 2.0, color);
            }
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
        self.rect(x, y, w, h, [0.1, 0.1, 0.1, 0.9]);
        // More sensitive dB conversion for quiet signals
        let db = if level > 0.0001 {
            20.0 * level.log10()
        } else {
            -80.0
        };
        // Map -60dB to 0dB range with more sensitivity at lower levels
        let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        // Apply a curve to make quiet signals more visible
        let curved = normalized.sqrt();
        let fill_h = curved * h;
        let green_end = h * 0.6;
        let yellow_end = h * 0.85;
        if fill_h > 0.0 {
            self.rect(
                x,
                y + h - fill_h.min(green_end),
                w,
                fill_h.min(green_end),
                [0.2, 0.9, 0.2, 1.0],
            );
        }
        if fill_h > green_end {
            self.rect(
                x,
                y + h - green_end - (fill_h - green_end).min(yellow_end - green_end),
                w,
                (fill_h - green_end).min(yellow_end - green_end),
                [0.9, 0.9, 0.2, 1.0],
            );
        }
        if fill_h > yellow_end {
            self.rect(
                x,
                y + h - yellow_end - (fill_h - yellow_end),
                w,
                fill_h - yellow_end,
                [0.9, 0.2, 0.2, 1.0],
            );
        }
        self.rect_outline(x, y, w, h, 1.0, [0.5, 0.5, 0.5, 0.8]);
        self.text(font, label, x, y + h + 4.0, 1.0, [0.8, 0.8, 0.8, 1.0]);
    }

    pub fn build(&self, device: &MetalDevice) -> MetalBuffer {
        let size = self.vertices.len() * size_of::<Vertex>();
        let buffer = device
            .create_buffer(size, ResourceOptions::STORAGE_MODE_MANAGED)
            .expect("Failed to create vertex buffer");
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.vertices.as_ptr(),
                buffer.contents().cast(),
                self.vertices.len(),
            );
        }
        buffer
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}
