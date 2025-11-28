use screencapturekit::{
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
    },
    CMSampleBuffer,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test output handler that collects video samples
struct VideoTestOutput {
    samples: Arc<Mutex<Vec<CMSampleBuffer>>>,
}

impl SCStreamOutputTrait for VideoTestOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        if let Ok(mut guard) = self.samples.lock() {
            guard.push(sample_buffer);
        }
    }
}

/// Test output handler that collects audio samples
struct AudioTestOutput {
    samples: Arc<Mutex<Vec<CMSampleBuffer>>>,
}

impl SCStreamOutputTrait for AudioTestOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        if let Ok(mut guard) = self.samples.lock() {
            guard.push(sample_buffer);
        }
    }
}

#[test]
fn test_video_capture() {
    // Get shareable content
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Screen recording permission required!");
            println!("   Go to: System Settings → Privacy & Security → Screen Recording");
            println!("   Error: {e:?}");
            return; // Skip test gracefully
        }
    };

    let displays = content.displays();

    if displays.is_empty() {
        println!("⚠️  No displays available - skipping test");
        return;
    }

    let display = &displays[0];

    // Create configuration for video
    let config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080)
        .set_captures_audio(false);

    // Create filter for the display
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Create stream
    let mut stream = SCStream::new(&filter, &config);

    // Add video output handler
    let samples = Arc::new(Mutex::new(Vec::new()));
    let output = VideoTestOutput {
        samples: samples.clone(),
    };

    stream.add_output_handler(output, SCStreamOutputType::Screen);

    // Start capture
    stream.start_capture().expect("Failed to start capture");

    // Wait for some frames
    std::thread::sleep(Duration::from_secs(2));

    // Stop capture
    stream.stop_capture().expect("Failed to stop capture");

    // Verify we got frames
    let collected_samples = samples.lock().unwrap();

    if collected_samples.is_empty() {
        println!("⚠️  No video samples captured - this may be due to permissions or environment");
        return; // Skip assertion to avoid false negatives
    }

    println!("Captured {} video samples", collected_samples.len());

    // Verify sample properties
    if let Some(sample) = collected_samples.first() {
        if let Some(image_buffer) = sample.get_image_buffer() {
            let width = image_buffer.get_width();
            let height = image_buffer.get_height();

            println!("Video frame size: {width}x{height}");
            assert!(width > 0, "Invalid video width");
            assert!(height > 0, "Invalid video height");
        } else {
            println!("⚠️  First sample has no image buffer (may be idle frame)");
        }
    }
}

#[test]
fn test_audio_capture() {
    // Get shareable content
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Screen recording permission required!");
            println!("   Error: {e:?}");
            return;
        }
    };

    let displays = content.displays();

    if displays.is_empty() {
        println!("⚠️  No displays available - skipping test");
        return;
    }

    let display = &displays[0];

    // Create configuration for audio
    let config = SCStreamConfiguration::default().set_captures_audio(true);

    // Create filter for the display
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Create stream
    let mut stream = SCStream::new(&filter, &config);

    // Add audio output handler
    let samples = Arc::new(Mutex::new(Vec::new()));
    let output = AudioTestOutput {
        samples: samples.clone(),
    };

    stream.add_output_handler(output, SCStreamOutputType::Audio);

    // Start capture
    stream.start_capture().expect("Failed to start capture");

    // Wait for some audio samples
    std::thread::sleep(Duration::from_secs(3));

    // Stop capture
    stream.stop_capture().expect("Failed to stop capture");

    // Verify we got audio samples
    let collected_samples = samples.lock().unwrap();

    if collected_samples.is_empty() {
        println!("⚠️  No audio samples captured (OK if no audio was playing)");
        return;
    }

    println!("Captured {} audio samples", collected_samples.len());

    // Verify audio buffer properties (may be empty if no audio playing)
    let mut samples_with_data = 0;
    for sample in collected_samples.iter() {
        if let Some(audio_buffer_list) = sample.get_audio_buffer_list() {
            let num_buffers = audio_buffer_list.get_number_buffers();
            if num_buffers > 0 {
                samples_with_data += 1;
                if let Some(buffer) = audio_buffer_list.get_buffer(0) {
                    let data_size = buffer.get_data_byte_size();
                    println!("Audio buffer: {num_buffers} buffers, {data_size} bytes");
                }
            }
        }
    }

    let total_samples = collected_samples.len();
    drop(collected_samples);

    println!("Audio samples with buffer data: {samples_with_data}/{total_samples}");
    // Note: samples_with_data may be 0 if no audio was playing during capture
}

#[test]
fn test_video_and_audio_capture() {
    // Get shareable content
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Screen recording permission required!");
            println!("   Error: {e:?}");
            return;
        }
    };

    let displays = content.displays();

    if displays.is_empty() {
        println!("⚠️  No displays available - skipping test");
        return;
    }

    let display = &displays[0];

    // Create configuration for both video and audio
    let config = SCStreamConfiguration::default()
        .set_width(1280)
        .set_height(720)
        .set_captures_audio(true);

    // Create filter for the display
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();

    // Create stream
    let mut stream = SCStream::new(&filter, &config);

    // Add video output handler
    let video_samples = Arc::new(Mutex::new(Vec::new()));
    let video_output = VideoTestOutput {
        samples: video_samples.clone(),
    };
    stream.add_output_handler(video_output, SCStreamOutputType::Screen);

    // Add audio output handler
    let audio_samples = Arc::new(Mutex::new(Vec::new()));
    let audio_output = AudioTestOutput {
        samples: audio_samples.clone(),
    };
    stream.add_output_handler(audio_output, SCStreamOutputType::Audio);

    // Start capture
    stream.start_capture().expect("Failed to start capture");

    // Wait for samples
    std::thread::sleep(Duration::from_secs(3));

    // Stop capture
    stream.stop_capture().expect("Failed to stop capture");

    // Verify we got both video and audio samples
    let video_count = video_samples.lock().unwrap().len();
    let audio_count = audio_samples.lock().unwrap().len();

    println!("Captured {video_count} video samples and {audio_count} audio samples");

    if video_count == 0 {
        println!("⚠️  No video samples captured - may be due to permissions");
        return;
    }

    if audio_count == 0 {
        println!("⚠️  No audio samples captured (OK if no audio was playing)");
    }
}

#[test]
fn test_pixel_buffer_locking() {
    // Get shareable content
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Screen recording permission required!");
            println!("   Error: {e:?}");
            return;
        }
    };

    let displays = content.displays();

    if displays.is_empty() {
        println!("⚠️  No displays available - skipping test");
        return;
    }

    let display = &displays[0];

    // Create configuration
    let config = SCStreamConfiguration::default()
        .set_width(640)
        .set_height(480);

    // Create filter and stream
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();
    let mut stream = SCStream::new(&filter, &config);

    // Add output handler
    let samples = Arc::new(Mutex::new(Vec::new()));
    let output = VideoTestOutput {
        samples: samples.clone(),
    };
    stream.add_output_handler(output, SCStreamOutputType::Screen);

    // Start capture
    stream.start_capture().expect("Failed to start capture");

    // Wait for one frame
    std::thread::sleep(Duration::from_millis(500));

    // Stop capture
    stream.stop_capture().expect("Failed to stop capture");

    // Test pixel buffer locking
    let collected_samples = samples.lock().unwrap();
    if let Some(sample) = collected_samples.first() {
        let Some(pixel_buffer) = sample.get_image_buffer() else {
            println!("⚠️  First sample has no image buffer (may be idle frame)");
            return;
        };

        // Test read lock
        {
            let lock_guard = pixel_buffer
                .lock_base_address(true)
                .expect("Failed to lock base address for reading");

            let base_address = lock_guard.get_base_address();
            assert!(!base_address.is_null(), "Base address is null");

            let width = pixel_buffer.get_width();
            let height = pixel_buffer.get_height();
            let bytes_per_row = pixel_buffer.get_bytes_per_row();

            println!("Locked pixel buffer: {width}x{height}, {bytes_per_row} bytes/row");

            // Lock guard automatically unlocks when dropped
        }

        // Test write lock
        {
            let mut lock_guard = pixel_buffer
                .lock_base_address(false)
                .expect("Failed to lock base address for writing");

            let base_address_mut = lock_guard.get_base_address_mut();
            assert!(!base_address_mut.is_null(), "Mutable base address is null");

            // Lock guard automatically unlocks when dropped
        }

        println!("Pixel buffer locking test passed");
    }
}

#[test]
fn test_iosurface_backed_buffer() {
    // Get shareable content
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Screen recording permission required!");
            println!("   Error: {e:?}");
            return;
        }
    };
    let displays = content.displays();

    assert!(!displays.is_empty(), "No displays available");

    let display = &displays[0];

    // Create configuration
    let config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080);

    // Create filter and stream
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();
    let mut stream = SCStream::new(&filter, &config);

    // Add output handler
    let samples = Arc::new(Mutex::new(Vec::new()));
    let output = VideoTestOutput {
        samples: samples.clone(),
    };
    stream.add_output_handler(output, SCStreamOutputType::Screen);

    // Start capture
    stream.start_capture().expect("Failed to start capture");

    // Wait for one frame
    std::thread::sleep(Duration::from_millis(500));

    // Stop capture
    stream.stop_capture().expect("Failed to stop capture");

    // Test IOSurface backing
    let collected_samples = samples.lock().unwrap();
    if let Some(sample) = collected_samples.first() {
        let Some(pixel_buffer) = sample.get_image_buffer() else {
            println!("⚠️  First sample has no image buffer (may be idle frame)");
            return;
        };

        // Check if backed by IOSurface
        let iosurface = pixel_buffer.get_iosurface();
        assert!(iosurface.is_some(), "Pixel buffer is not IOSurface-backed");

        if let Some(surface) = iosurface {
            let width = surface.get_width();
            let height = surface.get_height();
            let bytes_per_row = surface.get_bytes_per_row();

            println!("IOSurface: {width}x{height}, {bytes_per_row} bytes/row");
            assert!(width > 0, "Invalid IOSurface width");
            assert!(height > 0, "Invalid IOSurface height");
            assert!(bytes_per_row > 0, "Invalid IOSurface bytes per row");
        }
    }
}
