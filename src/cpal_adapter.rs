//! cpal integration for zero-copy audio capture
//!
//! This module provides multiple ways to integrate `ScreenCaptureKit` audio with cpal:
//!
//! 1. **Zero-copy stream** - `ZeroCopyAudioStream` for lowest latency (recommended)
//! 2. **Buffered stream** - `SckAudioInputStream` with ring buffer for cpal output
//! 3. **Low-level adapters** - `AudioSamples`, `CpalAudioExt` for manual integration
//!
//! # Example: Zero-Copy (Recommended for processing)
//!
//! ```ignore
//! use screencapturekit::cpal_adapter::ZeroCopyAudioStream;
//! use screencapturekit::prelude::*;
//!
//! let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
//!
//! // Callback receives direct pointer to CMSampleBuffer data - no copies!
//! let stream = ZeroCopyAudioStream::start(&filter, |samples, info| {
//!     // WARNING: This runs on SCStream's thread - don't block!
//!     analyze_audio(samples);
//! })?;
//!
//! std::thread::sleep(std::time::Duration::from_secs(10));
//! drop(stream); // Stops capture
//! ```
//!
//! # Example: Buffered (for cpal output playback)
//!
//! ```ignore
//! use screencapturekit::cpal_adapter::{SckAudioInputStream, create_output_callback};
//! use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//!
//! let mut input = SckAudioInputStream::new(&filter)?;
//! let buffer = input.ring_buffer().clone();
//! input.start(|_, _| {})?;
//!
//! // Play captured audio through speakers
//! let host = cpal::default_host();
//! let device = host.default_output_device().unwrap();
//! let output = device.build_output_stream(
//!     &input.cpal_config(),
//!     create_output_callback(buffer),
//!     |_| {},
//!     None,
//! )?;
//! output.play()?;
//! ```
//!
//! # Performance Comparison
//!
//! | API | Copies | Latency | Use Case |
//! |-----|--------|---------|----------|
//! | `ZeroCopyAudioStream` | 0 | Lowest | Audio analysis, effects |
//! | `SckAudioInputStream` | 2 | Low | cpal output, buffering needed |
//! | `AudioSamples` (manual) | 0 | Lowest | Custom integration |

use crate::cm::{AudioBufferList, CMSampleBuffer};
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::stream::output_trait::SCStreamOutputTrait;
use crate::stream::output_type::SCStreamOutputType;
use crate::stream::sc_stream::SCStream;

use std::sync::{Arc, Condvar, Mutex};

// ============================================================================
// Low-level adapters
// ============================================================================

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

// ============================================================================
// Ring Buffer for audio transfer
// ============================================================================

/// Thread-safe ring buffer for transferring audio between SCK and cpal
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    available: usize,
}

impl AudioRingBuffer {
    /// Create a new ring buffer with the given capacity in samples
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            available: 0,
        }
    }

    /// Write samples to the buffer, returning number written
    pub fn write(&mut self, samples: &[f32]) -> usize {
        let to_write = samples.len().min(self.capacity - self.available);
        for &sample in &samples[..to_write] {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        self.available += to_write;
        to_write
    }

    /// Read samples from the buffer, returning number read
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let to_read = output.len().min(self.available);
        for sample in &mut output[..to_read] {
            *sample = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        // Fill rest with silence
        for sample in &mut output[to_read..] {
            *sample = 0.0;
        }
        self.available -= to_read;
        to_read
    }

    /// Get available samples count
    pub fn available(&self) -> usize {
        self.available
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.available == 0
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
        self.available = 0;
    }
}

// ============================================================================
// Shared state for SCK audio capture
// ============================================================================

/// Internal state for audio capture
pub struct SckAudioState {
    ring_buffer: AudioRingBuffer,
    format: Option<AudioFormat>,
    running: bool,
}

impl SckAudioState {
    /// Read samples from the internal ring buffer
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        self.ring_buffer.read(output)
    }

    /// Check if audio is available
    pub fn available(&self) -> usize {
        self.ring_buffer.available()
    }
}

/// Shared audio state wrapped in synchronization primitives
pub type SharedAudioState = Arc<(Mutex<SckAudioState>, Condvar)>;

/// Internal handler that receives audio from `SCStream`
struct SckAudioHandler {
    state: SharedAudioState,
}

impl SCStreamOutputTrait for SckAudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }

        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();

        // Detect format on first sample
        if state.format.is_none() {
            state.format = AudioFormat::from_sample_buffer(&sample);
        }

        // Copy audio to ring buffer
        if let Some(samples) = sample.audio_f32_samples() {
            let slice = samples.as_f32_slice();
            state.ring_buffer.write(slice);
            cvar.notify_all();
        }
    }
}

// ============================================================================
// SCK Audio Input Stream
// ============================================================================

/// Configuration for SCK audio input
#[derive(Clone)]
pub struct SckAudioConfig {
    filter: SCContentFilter,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
}

impl SckAudioConfig {
    /// Create a new config with the given content filter
    pub fn new(filter: &SCContentFilter) -> Self {
        Self {
            filter: filter.clone(),
            sample_rate: 48000,
            channels: 2,
            buffer_size: 48000 * 2, // 1 second buffer
        }
    }

    /// Set the sample rate
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Set the number of channels
    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }

    /// Set the buffer size in samples
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Convert to cpal `StreamConfig`
    pub fn to_cpal_config(&self) -> cpal::StreamConfig {
        cpal::StreamConfig {
            channels: self.channels,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        }
    }
}

/// Information about audio callback
#[derive(Debug, Clone, Copy)]
pub struct SckAudioCallbackInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
}

/// Errors from SCK audio operations
#[derive(Debug, Clone)]
pub enum SckAudioError {
    /// Stream creation failed
    StreamCreationFailed(String),
    /// Stream start failed
    StreamStartFailed(String),
    /// Stream already running
    AlreadyRunning,
    /// Stream not running
    NotRunning,
}

impl std::fmt::Display for SckAudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreamCreationFailed(e) => write!(f, "Stream creation failed: {e}"),
            Self::StreamStartFailed(e) => write!(f, "Stream start failed: {e}"),
            Self::AlreadyRunning => write!(f, "Stream already running"),
            Self::NotRunning => write!(f, "Stream not running"),
        }
    }
}

impl std::error::Error for SckAudioError {}

/// SCK audio input stream with callback support
///
/// This provides a cpal-style callback interface for SCK audio capture.
///
/// # Example
///
/// ```ignore
/// use screencapturekit::cpal_adapter::SckAudioInputStream;
/// use screencapturekit::prelude::*;
///
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
/// let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
///
/// let mut stream = SckAudioInputStream::new(&filter)?;
/// stream.start(|samples, info| {
///     println!("Got {} samples at {}Hz", samples.len(), info.sample_rate);
/// })?;
/// ```
pub struct SckAudioInputStream {
    config: SckAudioConfig,
    state: SharedAudioState,
    stream: Option<SCStream>,
    callback_thread: Option<std::thread::JoinHandle<()>>,
}

impl SckAudioInputStream {
    /// Create a new audio input stream with default settings
    pub fn new(filter: &SCContentFilter) -> Result<Self, SckAudioError> {
        Self::with_config(SckAudioConfig::new(filter))
    }

    /// Create with custom configuration
    pub fn with_config(config: SckAudioConfig) -> Result<Self, SckAudioError> {
        let state: SharedAudioState = Arc::new((
            Mutex::new(SckAudioState {
                ring_buffer: AudioRingBuffer::new(config.buffer_size),
                format: None,
                running: false,
            }),
            Condvar::new(),
        ));

        Ok(Self {
            config,
            state,
            stream: None,
            callback_thread: None,
        })
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Get the number of channels
    pub fn channels(&self) -> u16 {
        self.config.channels
    }

    /// Check if the stream is running
    pub fn is_running(&self) -> bool {
        let (lock, _) = &*self.state;
        lock.lock().unwrap().running
    }

    /// Get a cpal-compatible stream config
    pub fn cpal_config(&self) -> cpal::StreamConfig {
        self.config.to_cpal_config()
    }

    /// Start capturing with a callback
    ///
    /// The callback receives audio samples as f32 slices.
    pub fn start<F>(&mut self, mut callback: F) -> Result<(), SckAudioError>
    where
        F: FnMut(&[f32], SckAudioCallbackInfo) + Send + 'static,
    {
        if self.is_running() {
            return Err(SckAudioError::AlreadyRunning);
        }

        // Create SCStream configuration
        let stream_config = SCStreamConfiguration::new()
            .with_width(1)
            .with_height(1)
            .with_captures_audio(true)
            .with_sample_rate(self.config.sample_rate as i32)
            .with_channel_count(self.config.channels as i32);

        // Create handler
        let handler = SckAudioHandler {
            state: Arc::clone(&self.state),
        };

        // Create and start stream
        let mut stream = SCStream::new(&self.config.filter, &stream_config);
        stream.add_output_handler(handler, SCStreamOutputType::Audio);

        stream
            .start_capture()
            .map_err(|e| SckAudioError::StreamStartFailed(e.to_string()))?;

        // Mark as running
        {
            let (lock, _) = &*self.state;
            lock.lock().unwrap().running = true;
        }

        // Spawn callback thread
        let state_clone = Arc::clone(&self.state);
        let channels = self.config.channels;
        let sample_rate = self.config.sample_rate;
        let buffer_size = 1024 * channels as usize;

        let handle = std::thread::spawn(move || {
            let mut buffer = vec![0.0f32; buffer_size];
            let info = SckAudioCallbackInfo {
                sample_rate,
                channels,
            };

            loop {
                {
                    let (lock, cvar) = &*state_clone;
                    let mut state = lock.lock().unwrap();

                    if !state.running {
                        break;
                    }

                    // Wait for data
                    while state.ring_buffer.is_empty() && state.running {
                        state = cvar.wait(state).unwrap();
                    }

                    if !state.running {
                        break;
                    }

                    // Read available data
                    let read = state.ring_buffer.read(&mut buffer);
                    if read > 0 {
                        drop(state); // Release lock before callback
                        callback(&buffer[..read], info);
                    }
                }
            }
        });

        self.stream = Some(stream);
        self.callback_thread = Some(handle);

        Ok(())
    }

    /// Stop capturing
    pub fn stop(&mut self) -> Result<(), SckAudioError> {
        if !self.is_running() {
            return Err(SckAudioError::NotRunning);
        }

        // Signal stop
        {
            let (lock, cvar) = &*self.state;
            let mut state = lock.lock().unwrap();
            state.running = false;
            cvar.notify_all();
        }

        // Wait for callback thread
        if let Some(handle) = self.callback_thread.take() {
            let _ = handle.join();
        }

        // Stop SCStream
        if let Some(ref mut stream) = self.stream {
            let _ = stream.stop_capture();
        }
        self.stream = None;

        Ok(())
    }

    /// Get access to the ring buffer for manual reading
    ///
    /// This is useful for integration with cpal output streams.
    pub fn ring_buffer(&self) -> &SharedAudioState {
        &self.state
    }
}

impl Drop for SckAudioInputStream {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ============================================================================
// Helper for cpal output integration
// ============================================================================

/// Create a cpal output callback that reads from an `SckAudioInputStream`
///
/// # Example
///
/// ```ignore
/// use screencapturekit::cpal_adapter::{SckAudioInputStream, create_output_callback};
/// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
///
/// let mut input = SckAudioInputStream::new(&filter)?;
/// let buffer = input.ring_buffer().clone();
///
/// input.start(|_, _| {})?; // Start capture (callback not used when bridging)
///
/// let host = cpal::default_host();
/// let device = host.default_output_device().unwrap();
/// let config = input.cpal_config();
///
/// let output = device.build_output_stream(
///     &config,
///     create_output_callback(buffer),
///     |err| eprintln!("Error: {}", err),
///     None,
/// )?;
/// output.play()?;
/// ```
pub fn create_output_callback(
    state: SharedAudioState,
) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let (lock, _) = &*state;
        if let Ok(mut state) = lock.lock() {
            state.ring_buffer.read(data);
        } else {
            // Fill with silence on lock failure
            for sample in data.iter_mut() {
                *sample = 0.0;
            }
        }
    }
}

// ============================================================================
// Zero-Copy Audio Stream
// ============================================================================

/// Zero-copy audio handler that calls callback directly from SCStream thread
struct ZeroCopyAudioHandler<F>
where
    F: FnMut(&[f32], SckAudioCallbackInfo) + Send,
{
    callback: std::sync::Mutex<F>,
    sample_rate: u32,
    channels: u16,
}

impl<F> SCStreamOutputTrait for ZeroCopyAudioHandler<F>
where
    F: FnMut(&[f32], SckAudioCallbackInfo) + Send,
{
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }

        // Zero-copy: get slice directly from CMSampleBuffer
        if let Some(audio_samples) = sample.audio_f32_samples() {
            let slice = audio_samples.as_f32_slice();
            if !slice.is_empty() {
                let info = SckAudioCallbackInfo {
                    sample_rate: self.sample_rate,
                    channels: self.channels,
                };
                if let Ok(mut callback) = self.callback.lock() {
                    callback(slice, info);
                }
            }
        }
    }
}

/// Zero-copy audio input stream
///
/// Unlike `SckAudioInputStream`, this calls your callback directly from the
/// SCStream capture thread with a reference to the audio data - no copies.
///
/// **Trade-offs:**
/// - ✅ Zero-copy: data is read directly from `CMSampleBuffer`
/// - ✅ Lower latency: no intermediate buffering
/// - ⚠️ Callback runs on SCStream's thread (don't block!)
/// - ⚠️ Cannot be used with `create_output_callback` (no ring buffer)
///
/// # Example
///
/// ```ignore
/// use screencapturekit::cpal_adapter::ZeroCopyAudioStream;
/// use screencapturekit::prelude::*;
///
/// let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
///
/// let stream = ZeroCopyAudioStream::start(&filter, |samples, info| {
///     // This runs on SCStream's thread - don't block!
///     // `samples` points directly into CMSampleBuffer memory
///     process_audio(samples);
/// })?;
///
/// // Stream runs until dropped
/// std::thread::sleep(std::time::Duration::from_secs(10));
/// drop(stream); // Stops capture
/// ```
pub struct ZeroCopyAudioStream {
    stream: SCStream,
}

impl ZeroCopyAudioStream {
    /// Start zero-copy audio capture
    ///
    /// The callback receives a direct reference to audio data in the `CMSampleBuffer`.
    /// **Important:** The callback runs on SCStream's internal thread - avoid blocking!
    pub fn start<F>(filter: &SCContentFilter, callback: F) -> Result<Self, SckAudioError>
    where
        F: FnMut(&[f32], SckAudioCallbackInfo) + Send + 'static,
    {
        Self::start_with_config(filter, 48000, 2, callback)
    }

    /// Start with custom sample rate and channels
    pub fn start_with_config<F>(
        filter: &SCContentFilter,
        sample_rate: u32,
        channels: u16,
        callback: F,
    ) -> Result<Self, SckAudioError>
    where
        F: FnMut(&[f32], SckAudioCallbackInfo) + Send + 'static,
    {
        let stream_config = SCStreamConfiguration::new()
            .with_width(1)
            .with_height(1)
            .with_captures_audio(true)
            .with_sample_rate(sample_rate as i32)
            .with_channel_count(channels as i32);

        let handler = ZeroCopyAudioHandler {
            callback: std::sync::Mutex::new(callback),
            sample_rate,
            channels,
        };

        let mut stream = SCStream::new(filter, &stream_config);
        stream.add_output_handler(handler, SCStreamOutputType::Audio);

        stream
            .start_capture()
            .map_err(|e| SckAudioError::StreamStartFailed(e.to_string()))?;

        Ok(Self { stream })
    }

    /// Stop capture explicitly (also happens on drop)
    pub fn stop(self) -> Result<(), SckAudioError> {
        // Drop will handle cleanup
        Ok(())
    }
}

impl Drop for ZeroCopyAudioStream {
    fn drop(&mut self) {
        let _ = self.stream.stop_capture();
    }
}
