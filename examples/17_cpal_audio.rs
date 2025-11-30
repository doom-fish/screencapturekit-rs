//! Example: cpal audio playback integration
//!
//! Demonstrates capturing system audio and playing it back through cpal.
//!
//! Run with: cargo run --example 17_cpal_audio --features cpal
//!
//! Note: Requires screen recording permission and audio output device.

use screencapturekit::cpal_adapter::{AudioFormat, CpalAudioExt};
use screencapturekit::prelude::*;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Ring buffer for audio samples
struct AudioRingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    capacity: usize,
}

impl AudioRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            write_pos: 0,
            read_pos: 0,
            capacity,
        }
    }

    fn write(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
    }

    fn read(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
    }
}

struct AudioHandler {
    ring_buffer: Arc<Mutex<AudioRingBuffer>>,
    format_detected: Arc<Mutex<Option<AudioFormat>>>,
}

impl SCStreamOutputTrait for AudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }

        // Detect audio format on first sample
        {
            let mut format = self.format_detected.lock().unwrap();
            if format.is_none() {
                if let Some(f) = AudioFormat::from_sample_buffer(&sample) {
                    println!(
                        "🔊 Audio format detected: {}Hz, {} channels, {} bits, float={}",
                        f.sample_rate, f.channels, f.bits_per_sample, f.is_float
                    );
                    *format = Some(f);
                }
            }
        }

        // Copy audio samples to ring buffer
        if let Some(samples) = sample.audio_f32_samples() {
            let slice = samples.as_f32_slice();
            if !slice.is_empty() {
                let mut rb = self.ring_buffer.lock().unwrap();
                rb.write(slice);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 cpal Audio Capture Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Get display to capture
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let display = displays.first().ok_or("No display found")?;

    println!(
        "📺 Capturing display: {}x{}",
        display.width(),
        display.height()
    );

    // Create content filter
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Configure stream with audio capture
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    // Setup ring buffer for audio transfer
    let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(48000 * 2 * 2))); // 2 seconds buffer
    let format_detected = Arc::new(Mutex::new(None));

    let handler = AudioHandler {
        ring_buffer: Arc::clone(&ring_buffer),
        format_detected: Arc::clone(&format_detected),
    };

    // Start capture
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Audio);
    stream.start_capture()?;

    println!("🎬 Capture started, waiting for audio format detection...");

    // Wait for audio format detection
    let audio_format = loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Some(format) = format_detected.lock().unwrap().clone() {
            break format;
        }
    };

    // Setup cpal output
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device found")?;

    println!("🔈 Output device: {}", device.name()?);

    let stream_config = audio_format.to_stream_config();
    let rb_clone = Arc::clone(&ring_buffer);

    let output_stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut rb = rb_clone.lock().unwrap();
            rb.read(data);
        },
        |err| eprintln!("Audio output error: {}", err),
        None,
    )?;

    output_stream.play()?;
    println!("▶️  Audio playback started");
    println!();
    println!("Playing captured system audio for 10 seconds...");
    println!("(Make sure to play some audio on your Mac!)");

    std::thread::sleep(std::time::Duration::from_secs(10));

    // Cleanup
    drop(output_stream);
    stream.stop_capture()?;

    println!();
    println!("✅ Done!");

    Ok(())
}
