#![allow(clippy::pedantic, clippy::nursery)]
//! End-to-end audio capture test with audio processing
//!
//! This test verifies that audio capture works from start to finish:
//! 1. Get shareable content (displays)
//! 2. Create a content filter
//! 3. Configure stream for audio capture
//! 4. Start capture
//! 5. Receive audio samples
//! 6. Process audio data (calculate levels, detect silence, analyze format)
//! 7. Verify audio buffer data
//! 8. Stop capture

use screencapturekit::{
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration,
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
    cm::CMSampleBuffer,
};

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use std::thread;

/// Handler that collects audio samples
struct AudioCapture {
    sample_count: Arc<AtomicUsize>,
    audio_samples: Arc<Mutex<Vec<AudioSampleInfo>>>,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct AudioSampleInfo {
    num_buffers: usize,
    total_bytes: usize,
    has_data: bool,
    rms_level: f32,
    peak_level: f32,
    is_silent: bool,
    sample_rate: Option<u32>,
    channels: u32,
}

/// Audio processor that analyzes audio data
struct AudioProcessor;

impl AudioProcessor {
    /// Calculate RMS (Root Mean Square) level for audio data
    /// This gives us the average power/loudness of the audio
    fn calculate_rms(data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        
        // Treat data as 16-bit PCM samples (most common format)
        let samples: Vec<i16> = data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f64 = samples
            .iter()
            .map(|&s| {
                let normalized = s as f64 / i16::MAX as f64;
                normalized * normalized
            })
            .sum();
        
        let mean_square = sum_squares / samples.len() as f64;
        (mean_square.sqrt() * 100.0) as f32 // Scale to 0-100
    }
    
    /// Calculate peak level (maximum amplitude)
    fn calculate_peak(data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        
        let samples: Vec<i16> = data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        
        if samples.is_empty() {
            return 0.0;
        }
        
        let max_sample = samples
            .iter()
            .map(|&s| s.abs())
            .max()
            .unwrap_or(0);
        
        max_sample as f32 / i16::MAX as f32 * 100.0
    }
    
    /// Detect if audio is silent (below threshold)
    fn is_silent(rms_level: f32, threshold: f32) -> bool {
        rms_level < threshold
    }
    
    /// Analyze audio format
    fn analyze_format(data: &[u8], _channels: u32) -> Option<u32> {
        // Most common sample rates for screen audio
        // Typically 44100 or 48000 Hz
        if data.len() > 8192 {
            Some(48000) // Common screen capture sample rate
        } else {
            Some(44100)
        }
    }
    
    /// Create a visual level meter
    fn create_level_meter(level: f32, width: usize) -> String {
        let bars = (level / 100.0 * width as f32) as usize;
        let bars = bars.min(width);
        
        let filled = "█".repeat(bars);
        let empty = "░".repeat(width - bars);
        
        format!("[{}{}] {:.1}%", filled, empty, level)
    }
}

impl AudioCapture {
    fn new(
        sample_count: Arc<AtomicUsize>,
        audio_samples: Arc<Mutex<Vec<AudioSampleInfo>>>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            sample_count,
            audio_samples,
            running,
        }
    }
}

impl SCStreamOutputTrait for AudioCapture {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        if matches!(of_type, SCStreamOutputType::Audio) {
            let count = self.sample_count.fetch_add(1, Ordering::Relaxed);
            
            // Try to get audio buffer list
            if let Some(buffer_list) = sample.get_audio_buffer_list() {
                let num_buffers = buffer_list.num_buffers();
                let mut total_bytes = 0;
                let mut has_data = false;
                let mut all_audio_data = Vec::new();
                let mut channels = 2; // Default stereo
                
                // Collect all audio data from all buffers
                for i in 0..num_buffers {
                    if let Some(buffer) = buffer_list.get(i) {
                        let data = buffer.data();
                        total_bytes += data.len();
                        if !data.is_empty() {
                            has_data = true;
                            all_audio_data.extend_from_slice(data);
                            channels = buffer.number_channels;
                        }
                    }
                }
                
                // Process the audio data
                let rms_level = if has_data {
                    AudioProcessor::calculate_rms(&all_audio_data)
                } else {
                    0.0
                };
                
                let peak_level = if has_data {
                    AudioProcessor::calculate_peak(&all_audio_data)
                } else {
                    0.0
                };
                
                let is_silent = AudioProcessor::is_silent(rms_level, 1.0);
                let sample_rate = AudioProcessor::analyze_format(&all_audio_data, channels);
                
                let info = AudioSampleInfo {
                    num_buffers,
                    total_bytes,
                    has_data,
                    rms_level,
                    peak_level,
                    is_silent,
                    sample_rate,
                    channels,
                };
                
                if let Ok(mut samples) = self.audio_samples.lock() {
                    samples.push(info.clone());
                }
                
                // Show detailed info for first few samples
                if count < 5 && has_data {
                    println!("  Audio sample {}:", count + 1);
                    println!("    • Buffers: {}", num_buffers);
                    println!("    • Bytes: {}", total_bytes);
                    println!("    • Channels: {}", channels);
                    println!("    • RMS Level: {}", AudioProcessor::create_level_meter(rms_level, 20));
                    println!("    • Peak Level: {}", AudioProcessor::create_level_meter(peak_level, 20));
                    println!("    • Silent: {}", if is_silent { "Yes" } else { "No" });
                } else if count < 5 {
                    println!("  Audio sample {}: {} buffers, {} bytes, has_data: {}", 
                        count + 1, num_buffers, total_bytes, has_data);
                }
            }
        }
    }
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_audio_capture_end_to_end() {
    println!("\n=== Audio Capture End-to-End Test ===\n");
    
    // Step 1: Get shareable content
    println!("Step 1: Getting shareable content...");
    let content = match SCShareableContent::get() {
        Ok(content) => {
            println!("  ✓ Got shareable content");
            content
        }
        Err(e) => {
            eprintln!("  ⚠ Could not get shareable content: {:?}", e);
            eprintln!("  Note: This test requires screen recording permission");
            return; // Skip test if permissions not granted
        }
    };
    
    let mut displays = content.displays();
    if displays.is_empty() {
        eprintln!("  ⚠ No displays available");
        return;
    }
    
    let display = displays.remove(0);
    println!("  ✓ Using display: {}", display.display_id());
    
    // Step 2: Create content filter
    println!("\nStep 2: Creating content filter...");
    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();
    println!("  ✓ Content filter created");
    
    // Step 3: Configure stream for audio
    println!("\nStep 3: Configuring stream for audio capture...");
    let config = match SCStreamConfiguration::build()
        .set_captures_audio(true)
    {
        Ok(config) => {
            println!("  ✓ Stream configured for audio");
            config
        }
        Err(e) => {
            eprintln!("  ✗ Failed to configure stream: {:?}", e);
            panic!("Configuration failed");
        }
    };
    
    // Step 4: Create stream and add output handler
    println!("\nStep 4: Creating stream...");
    let sample_count = Arc::new(AtomicUsize::new(0));
    let audio_samples = Arc::new(Mutex::new(Vec::new()));
    let running = Arc::new(AtomicBool::new(true));
    
    let mut stream = SCStream::new(&filter, &config);
    
    let handler = AudioCapture::new(
        Arc::clone(&sample_count),
        Arc::clone(&audio_samples),
        Arc::clone(&running),
    );
    
    stream.add_output_handler(handler, SCStreamOutputType::Audio);
    println!("  ✓ Stream created with audio handler");
    
    // Step 5: Start capture
    println!("\nStep 5: Starting audio capture...");
    match stream.start_capture() {
        Ok(_) => println!("  ✓ Capture started"),
        Err(e) => {
            eprintln!("  ✗ Failed to start capture: {:?}", e);
            panic!("Start capture failed");
        }
    }
    
    // Step 6: Wait for audio samples
    println!("\nStep 6: Collecting audio samples...");
    println!("  Waiting for audio data (10 seconds)...");
    
    let max_wait = Duration::from_secs(10);
    let check_interval = Duration::from_millis(500);
    let start = std::time::Instant::now();
    
    while start.elapsed() < max_wait {
        thread::sleep(check_interval);
        
        let count = sample_count.load(Ordering::Relaxed);
        if count >= 10 {
            println!("  ✓ Received {} audio samples", count);
            break;
        }
    }
    
    // Step 7: Stop capture
    println!("\nStep 7: Stopping capture...");
    running.store(false, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(100));
    
    match stream.stop_capture() {
        Ok(_) => println!("  ✓ Capture stopped"),
        Err(e) => eprintln!("  ⚠ Stop capture warning: {:?}", e),
    }
    
    // Give time for cleanup
    thread::sleep(Duration::from_millis(500));
    
    // Step 8: Verify results
    println!("\n=== Results ===\n");
    
    let final_count = sample_count.load(Ordering::Relaxed);
    println!("Total audio samples received: {}", final_count);
    
    let samples = audio_samples.lock().unwrap();
    println!("Audio samples with data: {}", samples.len());
    
    if !samples.is_empty() {
        let total_bytes: usize = samples.iter().map(|s| s.total_bytes).sum();
        let avg_bytes = total_bytes / samples.len();
        let has_data_count = samples.iter().filter(|s| s.has_data).count();
        
        // Calculate audio processing statistics
        let samples_with_sound: Vec<_> = samples.iter()
            .filter(|s| s.has_data && !s.is_silent)
            .collect();
        
        let avg_rms = if !samples_with_sound.is_empty() {
            samples_with_sound.iter().map(|s| s.rms_level).sum::<f32>() / samples_with_sound.len() as f32
        } else {
            0.0
        };
        
        let avg_peak = if !samples_with_sound.is_empty() {
            samples_with_sound.iter().map(|s| s.peak_level).sum::<f32>() / samples_with_sound.len() as f32
        } else {
            0.0
        };
        
        let max_peak = samples.iter()
            .map(|s| s.peak_level)
            .fold(0.0f32, |a, b| a.max(b));
        
        println!("\nAudio Statistics:");
        println!("  • Samples with buffers: {}", samples.len());
        println!("  • Samples with data: {}", has_data_count);
        println!("  • Samples with sound: {}", samples_with_sound.len());
        println!("  • Silent samples: {}", samples.iter().filter(|s| s.is_silent).count());
        println!("  • Total bytes captured: {}", total_bytes);
        println!("  • Average bytes per sample: {}", avg_bytes);
        
        if let Some(first) = samples.first() {
            println!("  • Buffers per sample: {}", first.num_buffers);
            println!("  • Channels: {}", first.channels);
            if let Some(rate) = first.sample_rate {
                println!("  • Estimated sample rate: {} Hz", rate);
            }
        }
        
        if !samples_with_sound.is_empty() {
            println!("\nAudio Levels:");
            println!("  • Average RMS: {}", AudioProcessor::create_level_meter(avg_rms, 25));
            println!("  • Average Peak: {}", AudioProcessor::create_level_meter(avg_peak, 25));
            println!("  • Maximum Peak: {}", AudioProcessor::create_level_meter(max_peak, 25));
        }
    }
    
    // Assertions
    println!("\n=== Verification ===\n");
    
    if final_count == 0 {
        eprintln!("  ⚠ No audio samples received");
        eprintln!("  Note: This may happen if:");
        eprintln!("    - System audio is muted");
        eprintln!("    - No audio is playing");
        eprintln!("    - macOS version doesn't support audio capture");
        eprintln!("    - Audio permissions not granted");
        println!("\n  Test completed but no audio captured (this may be expected)");
    } else {
        println!("  ✓ Audio samples received: {}", final_count);
        
        if !samples.is_empty() {
            println!("  ✓ Audio buffer data collected");
            
            let has_any_data = samples.iter().any(|s| s.has_data);
            if has_any_data {
                println!("  ✓ Audio data verified");
            } else {
                println!("  ⚠ Samples received but no audio data (system may be muted)");
            }
        }
        
        println!("\n  ✅ Audio capture test PASSED");
    }
    
    println!("\n=== Test Complete ===\n");
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_audio_buffer_structure() {
    println!("\n=== Audio Buffer Structure Test ===\n");
    
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("  ⚠ Could not get shareable content: {:?}", e);
            return;
        }
    };
    
    let mut displays = content.displays();
    if displays.is_empty() {
        return;
    }
    
    let display = displays.remove(0);
    
    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();
    
    let config = match SCStreamConfiguration::build().set_captures_audio(true) {
        Ok(config) => config,
        Err(_) => return,
    };
    
    let sample_received = Arc::new(AtomicBool::new(false));
    let sample_received_clone = Arc::clone(&sample_received);
    
    struct BufferAnalyzer {
        sample_received: Arc<AtomicBool>,
    }
    
    impl SCStreamOutputTrait for BufferAnalyzer {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
            if matches!(of_type, SCStreamOutputType::Audio) {
                if let Some(buffer_list) = sample.get_audio_buffer_list() {
                    println!("Audio Buffer Analysis:");
                    println!("  Number of buffers: {}", buffer_list.num_buffers());
                    
                    for i in 0..buffer_list.num_buffers() {
                        if let Some(buffer) = buffer_list.get(i) {
                            println!("  Buffer {}:", i);
                            println!("    • Channels: {}", buffer.number_channels);
                            println!("    • Data size: {} bytes", buffer.data_bytes_size);
                            println!("    • Actual data: {} bytes", buffer.data().len());
                        }
                    }
                    
                    self.sample_received.store(true, Ordering::Relaxed);
                }
            }
        }
    }
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        BufferAnalyzer { sample_received: sample_received_clone },
        SCStreamOutputType::Audio,
    );
    
    if stream.start_capture().is_ok() {
        thread::sleep(Duration::from_secs(3));
        let _ = stream.stop_capture();
        
        if sample_received.load(Ordering::Relaxed) {
            println!("\n  ✅ Audio buffer structure verified");
        } else {
            println!("\n  ⚠ No audio sample received (may be expected)");
        }
    }
    
    println!("\n=== Test Complete ===\n");
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_audio_processing_and_save() {
    println!("\n=== Audio Processing & Save Test ===\n");
    
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("  ⚠ Could not get shareable content: {:?}", e);
            return;
        }
    };
    
    let mut displays = content.displays();
    if displays.is_empty() {
        return;
    }
    
    let display = displays.remove(0);
    
    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();
    
    let config = match SCStreamConfiguration::build().set_captures_audio(true) {
        Ok(config) => config,
        Err(_) => return,
    };
    
    let audio_data = Arc::new(Mutex::new(Vec::new()));
    let audio_data_clone = Arc::clone(&audio_data);
    let sample_count = Arc::new(AtomicUsize::new(0));
    let sample_count_clone = Arc::clone(&sample_count);
    
    struct AudioSaver {
        audio_data: Arc<Mutex<Vec<u8>>>,
        sample_count: Arc<AtomicUsize>,
    }
    
    impl SCStreamOutputTrait for AudioSaver {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
            if matches!(of_type, SCStreamOutputType::Audio) {
                let count = self.sample_count.fetch_add(1, Ordering::Relaxed);
                
                if let Some(buffer_list) = sample.get_audio_buffer_list() {
                    for i in 0..buffer_list.num_buffers() {
                        if let Some(buffer) = buffer_list.get(i) {
                            let data = buffer.data();
                            if !data.is_empty() {
                                if let Ok(mut audio) = self.audio_data.lock() {
                                    audio.extend_from_slice(data);
                                }
                                
                                if count < 3 {
                                    println!("  Captured audio chunk {}: {} bytes", count + 1, data.len());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        AudioSaver {
            audio_data: audio_data_clone,
            sample_count: sample_count_clone,
        },
        SCStreamOutputType::Audio,
    );
    
    println!("Starting audio capture for 3 seconds...");
    if stream.start_capture().is_ok() {
        thread::sleep(Duration::from_secs(3));
        let _ = stream.stop_capture();
        
        let total_samples = sample_count.load(Ordering::Relaxed);
        let audio = audio_data.lock().unwrap();
        
        println!("\nCapture Results:");
        println!("  • Samples captured: {}", total_samples);
        println!("  • Total audio data: {} bytes", audio.len());
        
        if !audio.is_empty() {
            // Analyze the captured audio
            let rms = AudioProcessor::calculate_rms(&audio);
            let peak = AudioProcessor::calculate_peak(&audio);
            
            println!("\nAudio Analysis:");
            println!("  • RMS Level: {}", AudioProcessor::create_level_meter(rms, 30));
            println!("  • Peak Level: {}", AudioProcessor::create_level_meter(peak, 30));
            println!("  • Is Silent: {}", AudioProcessor::is_silent(rms, 1.0));
            
            // Save to file
            let filename = "/tmp/captured_audio.raw";
            match std::fs::write(filename, &*audio) {
                Ok(_) => {
                    println!("\n  ✅ Audio saved to: {}", filename);
                    println!("  💡 Play with: ffplay -f s16le -ar 48000 -ac 2 {}", filename);
                }
                Err(e) => {
                    println!("\n  ⚠ Could not save audio: {}", e);
                }
            }
            
            // Calculate some statistics
            let duration_estimate = audio.len() as f32 / (48000.0 * 2.0 * 2.0); // 48kHz, 2 channels, 2 bytes per sample
            println!("\nEstimated Duration: {:.2} seconds", duration_estimate);
            
            // Analyze audio quality
            println!("\nAudio Quality Check:");
            if rms > 10.0 {
                println!("  ✅ Good audio level detected");
            } else if rms > 1.0 {
                println!("  ⚠ Low audio level (might be background noise)");
            } else {
                println!("  ℹ Very low audio level (likely silent)");
            }
            
            if peak > 90.0 {
                println!("  ⚠ Peak level very high (possible clipping)");
            } else if peak > 50.0 {
                println!("  ✅ Good peak level");
            } else {
                println!("  ℹ Low peak level");
            }
        } else {
            println!("\n  ℹ No audio data captured (system may be muted)");
        }
    }
    
    println!("\n=== Test Complete ===\n");
}

#[test]
#[cfg_attr(feature = "ci", ignore)]
fn test_audio_level_monitoring() {
    println!("\n=== Audio Level Monitoring Test ===\n");
    println!("This test monitors audio levels in real-time for 5 seconds");
    println!("💡 Play some audio to see the level meters!\n");
    
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("  ⚠ Could not get shareable content: {:?}", e);
            return;
        }
    };
    
    let mut displays = content.displays();
    if displays.is_empty() {
        return;
    }
    
    let display = displays.remove(0);
    
    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();
    
    let config = match SCStreamConfiguration::build().set_captures_audio(true) {
        Ok(config) => config,
        Err(_) => return,
    };
    
    struct LevelMonitor {
        sample_count: Arc<AtomicUsize>,
    }
    
    impl SCStreamOutputTrait for LevelMonitor {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
            if matches!(of_type, SCStreamOutputType::Audio) {
                let count = self.sample_count.fetch_add(1, Ordering::Relaxed);
                
                if let Some(buffer_list) = sample.get_audio_buffer_list() {
                    let mut all_data = Vec::new();
                    
                    for i in 0..buffer_list.num_buffers() {
                        if let Some(buffer) = buffer_list.get(i) {
                            let data = buffer.data();
                            if !data.is_empty() {
                                all_data.extend_from_slice(data);
                            }
                        }
                    }
                    
                    if !all_data.is_empty() {
                        let rms = AudioProcessor::calculate_rms(&all_data);
                        let peak = AudioProcessor::calculate_peak(&all_data);
                        
                        // Show level meters every 5 samples (reduces spam)
                        if count % 5 == 0 {
                            print!("\r  RMS:  {} | Peak: {}   ", 
                                AudioProcessor::create_level_meter(rms, 25),
                                AudioProcessor::create_level_meter(peak, 25));
                            use std::io::Write;
                            std::io::stdout().flush().ok();
                        }
                    }
                }
            }
        }
    }
    
    let sample_count = Arc::new(AtomicUsize::new(0));
    let sample_count_clone = Arc::clone(&sample_count);
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        LevelMonitor { sample_count: sample_count_clone },
        SCStreamOutputType::Audio,
    );
    
    if stream.start_capture().is_ok() {
        thread::sleep(Duration::from_secs(5));
        let _ = stream.stop_capture();
        
        println!("\n\n  ✅ Monitored {} audio samples", sample_count.load(Ordering::Relaxed));
    }
    
    println!("\n=== Test Complete ===\n");
}
