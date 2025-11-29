//! Screen capture handler

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use screencapturekit::output::{CVPixelBufferIOSurface, IOSurface};
use screencapturekit::prelude::*;

use crate::waveform::WaveformBuffer;

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
