//! cpal integration for zero-copy audio playback
//!
//! This module provides adapters to use captured audio with the cpal audio library.
//!
//! # Example
//!
//! ```ignore
//! use screencapturekit::cpal_adapter::{AudioSamples, CpalAudioExt};
//! use screencapturekit::cm::CMSampleBuffer;
//!
//! fn process_audio(sample: &CMSampleBuffer) {
//!     // Get f32 samples from the captured audio
//!     if let Some(samples) = sample.audio_f32_samples() {
//!         for sample in samples.iter() {
//!             // Process or send to cpal output stream
//!         }
//!     }
//! }
//! ```

use crate::cm::{AudioBufferList, CMSampleBuffer};

/// Audio samples extracted from a `CMSampleBuffer`
///
/// Owns the underlying `AudioBufferList` and provides access to samples
/// in various formats compatible with cpal.
pub struct AudioSamples {
    buffer_list: AudioBufferList,
}

impl AudioSamples {
    /// Create from a `CMSampleBuffer`
    ///
    /// Returns `None` if the sample buffer doesn't contain audio data.
    pub fn new(sample: &CMSampleBuffer) -> Option<Self> {
        let buffer_list = sample.audio_buffer_list()?;
        Some(Self { buffer_list })
    }

    /// Get the number of audio channels
    pub fn channels(&self) -> usize {
        self.buffer_list
            .get(0)
            .map(|b| b.number_channels as usize)
            .unwrap_or(0)
    }

    /// Get raw bytes of audio data
    pub fn as_bytes(&self) -> &[u8] {
        self.buffer_list.get(0).map(|b| b.data()).unwrap_or(&[])
    }

    /// Get audio samples as f32 slice (zero-copy if data is already f32)
    ///
    /// # Safety
    /// Assumes the audio data is in native-endian f32 format.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn as_f32_slice(&self) -> &[f32] {
        let bytes = self.as_bytes();
        if bytes.len() < 4 {
            return &[];
        }
        // Safety: macOS audio buffers are properly aligned for the sample type
        unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<f32>(), bytes.len() / 4) }
    }

    /// Get audio samples as i16 slice (zero-copy if data is already i16)
    ///
    /// # Safety
    /// Assumes the audio data is in native-endian i16 format.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn as_i16_slice(&self) -> &[i16] {
        let bytes = self.as_bytes();
        if bytes.len() < 2 {
            return &[];
        }
        // Safety: macOS audio buffers are properly aligned for the sample type
        unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<i16>(), bytes.len() / 2) }
    }

    /// Iterator over f32 samples
    pub fn iter_f32(&self) -> impl Iterator<Item = f32> + '_ {
        self.as_f32_slice().iter().copied()
    }

    /// Iterator over i16 samples
    pub fn iter_i16(&self) -> impl Iterator<Item = i16> + '_ {
        self.as_i16_slice().iter().copied()
    }

    /// Get the number of f32 samples
    pub fn len_f32(&self) -> usize {
        self.as_bytes().len() / 4
    }

    /// Get the number of i16 samples  
    pub fn len_i16(&self) -> usize {
        self.as_bytes().len() / 2
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

/// Extension trait for `CMSampleBuffer` to provide cpal-compatible audio access
pub trait CpalAudioExt {
    /// Get audio samples as f32 (for cpal output)
    fn audio_f32_samples(&self) -> Option<AudioSamples>;

    /// Copy f32 audio samples into a cpal-compatible buffer
    ///
    /// Returns the number of samples copied.
    fn copy_f32_to_buffer(&self, buffer: &mut [f32]) -> usize;

    /// Copy i16 audio samples into a cpal-compatible buffer
    ///
    /// Returns the number of samples copied.
    fn copy_i16_to_buffer(&self, buffer: &mut [i16]) -> usize;
}

impl CpalAudioExt for CMSampleBuffer {
    fn audio_f32_samples(&self) -> Option<AudioSamples> {
        AudioSamples::new(self)
    }

    fn copy_f32_to_buffer(&self, buffer: &mut [f32]) -> usize {
        let Some(samples) = AudioSamples::new(self) else {
            return 0;
        };
        let src = samples.as_f32_slice();
        let len = buffer.len().min(src.len());
        buffer[..len].copy_from_slice(&src[..len]);
        len
    }

    fn copy_i16_to_buffer(&self, buffer: &mut [i16]) -> usize {
        let Some(samples) = AudioSamples::new(self) else {
            return 0;
        };
        let src = samples.as_i16_slice();
        let len = buffer.len().min(src.len());
        buffer[..len].copy_from_slice(&src[..len]);
        len
    }
}

/// Audio format information for cpal stream configuration
#[derive(Debug, Clone, Copy)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Whether samples are float format
    pub is_float: bool,
}

impl AudioFormat {
    /// Extract audio format from a `CMSampleBuffer`
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_sample_buffer(sample: &CMSampleBuffer) -> Option<Self> {
        let format_desc = sample.format_description()?;
        if !format_desc.is_audio() {
            return None;
        }

        Some(Self {
            sample_rate: format_desc.audio_sample_rate()? as u32,
            channels: format_desc.audio_channel_count()? as u16,
            bits_per_sample: format_desc.audio_bits_per_channel()? as u16,
            is_float: format_desc.audio_is_float(),
        })
    }

    /// Convert to cpal `StreamConfig`
    pub fn to_stream_config(&self) -> cpal::StreamConfig {
        cpal::StreamConfig {
            channels: self.channels,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        }
    }

    /// Get the cpal `SampleFormat` based on the audio format
    pub fn sample_format(&self) -> cpal::SampleFormat {
        if self.is_float {
            cpal::SampleFormat::F32
        } else if self.bits_per_sample == 8 {
            cpal::SampleFormat::I8
        } else if self.bits_per_sample == 32 {
            cpal::SampleFormat::I32
        } else {
            cpal::SampleFormat::I16
        }
    }
}
