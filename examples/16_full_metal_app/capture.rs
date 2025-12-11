//! Screen capture handler
//!
//! This module demonstrates both basic and advanced `IOSurface` inspection APIs.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use screencapturekit::cm::IOSurface;
use screencapturekit::metal::{pixel_format, IOSurfaceInfo};
use screencapturekit::prelude::*;

use crate::waveform::WaveformBuffer;

pub struct CaptureState {
    pub frame_count: AtomicUsize,
    pub audio_waveform: Mutex<WaveformBuffer>,
    pub mic_waveform: Mutex<WaveformBuffer>,
    pub latest_surface: Mutex<Option<IOSurface>>,
    /// Cached surface info for display (updated on first frame or format change)
    pub surface_info: Mutex<Option<IOSurfaceInfo>>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            frame_count: AtomicUsize::new(0),
            audio_waveform: Mutex::new(WaveformBuffer::new(4096)),
            mic_waveform: Mutex::new(WaveformBuffer::new(4096)),
            latest_surface: Mutex::new(None),
            surface_info: Mutex::new(None),
        }
    }

    /// Get a formatted string describing the current surface format
    pub fn format_info(&self) -> Option<String> {
        let info = self.surface_info.lock().ok()?.clone()?;
        let format_str = if pixel_format::is_ycbcr_biplanar(info.pixel_format) {
            let range = if pixel_format::is_full_range(info.pixel_format) {
                "full"
            } else {
                "video"
            };
            format!("YCbCr 4:2:0 ({range} range)")
        } else if info.pixel_format.equals(pixel_format::BGRA) {
            "BGRA 8-bit".to_string()
        } else if info.pixel_format.equals(pixel_format::L10R) {
            "BGR10A2 10-bit".to_string()
        } else {
            format!("{}", info.pixel_format)
        };

        Some(format!(
            "{}x{} {} ({} planes, {} bytes/row)",
            info.width, info.height, format_str, info.plane_count, info.bytes_per_row
        ))
    }

    /// Get audio stats for status display
    pub fn audio_stats(&self) -> (u64, f32) {
        let waveform = self.audio_waveform.lock().unwrap();
        (waveform.sample_count(), waveform.peak(512))
    }

    /// Get microphone stats for status display
    pub fn mic_stats(&self) -> (u64, f32) {
        let waveform = self.mic_waveform.lock().unwrap();
        (waveform.sample_count(), waveform.peak(512))
    }

    /// Get display samples for waveform visualization
    pub fn audio_display_samples(&self, count: usize) -> Vec<f32> {
        self.audio_waveform.lock().unwrap().display_samples(count)
    }

    /// Get display samples for mic waveform visualization
    pub fn mic_display_samples(&self, count: usize) -> Vec<f32> {
        self.mic_waveform.lock().unwrap().display_samples(count)
    }

    /// Get RMS level for VU meter
    pub fn audio_rms(&self, count: usize) -> f32 {
        self.audio_waveform.lock().unwrap().rms(count)
    }

    /// Get RMS level for mic VU meter
    pub fn mic_rms(&self, count: usize) -> f32 {
        self.mic_waveform.lock().unwrap().rms(count)
    }
}

pub struct CaptureHandler {
    pub state: Arc<CaptureState>,
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
                if let Some(pixel_buffer) = sample.image_buffer() {
                    if pixel_buffer.is_backed_by_io_surface() {
                        if let Some(surface) = pixel_buffer.io_surface() {
                            // Update surface info on first frame or format change
                            self.update_surface_info(&surface);
                            *self.state.latest_surface.lock().unwrap() = Some(surface);
                        }
                    }
                }
            }
            SCStreamOutputType::Audio | SCStreamOutputType::Microphone => {
                // Get audio samples from audio_buffer_list
                if let Some(audio_buffer_list) = sample.audio_buffer_list() {
                    for buffer in &audio_buffer_list {
                        let data = buffer.data();
                        if data.is_empty() {
                            continue;
                        }
                        let audio_samples: Vec<f32> = data
                            .chunks_exact(4)
                            .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
                            .collect();

                        if !audio_samples.is_empty() {
                            let waveform = if matches!(output_type, SCStreamOutputType::Audio) {
                                &self.state.audio_waveform
                            } else {
                                &self.state.mic_waveform
                            };
                            waveform.lock().unwrap().push(&audio_samples);
                        }
                    }
                }
            }
        }
    }
}

impl CaptureHandler {
    /// Update cached surface info if format changed
    fn update_surface_info(&self, surface: &IOSurface) {
        let new_info = surface.info();

        // Check if we need to update (first frame or format change)
        let needs_update = {
            let current = self.state.surface_info.lock().unwrap();
            match current.as_ref() {
                None => true,
                Some(info) => {
                    info.width != new_info.width
                        || info.height != new_info.height
                        || !info.pixel_format.equals(new_info.pixel_format)
                }
            }
        };

        if needs_update {
            // Log detailed surface info using the introspection APIs
            println!("📊 IOSurface Info:");
            println!("   Size: {}x{}", new_info.width, new_info.height);
            println!("   Format: {}", new_info.pixel_format);
            println!("   Bytes/row: {}", new_info.bytes_per_row);
            println!("   Planes: {}", new_info.plane_count);

            // Show plane details for multi-planar formats
            for plane in &new_info.planes {
                println!(
                    "   Plane {}: {}x{}, {} bytes/row",
                    plane.index, plane.width, plane.height, plane.bytes_per_row
                );
            }

            // Show texture params that would be used for Metal
            let tex_params = surface.texture_params();
            println!("   Metal textures needed: {}", tex_params.len());
            for (i, param) in tex_params.iter().enumerate() {
                println!(
                    "   Texture {}: {}x{} {:?}",
                    i, param.width, param.height, param.format
                );
            }

            *self.state.surface_info.lock().unwrap() = Some(new_info);
        }
    }
}
