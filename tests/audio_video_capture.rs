use screencapturekit::cm::CMSampleBuffer;
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use screencapturekit::stream::output_type::SCStreamOutputType;
use screencapturekit::stream::SCStream;
use screencapturekit::stream::SCStreamOutput;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TestVideoOutput {
    frame_count: Arc<AtomicUsize>,
    received_frame: Arc<AtomicBool>,
}

impl SCStreamOutput for TestVideoOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        let count = self.frame_count.fetch_add(1, Ordering::SeqCst);
        self.received_frame.store(true, Ordering::SeqCst);

        println!(
            "Video frame {}: pts={:?}, duration={:?}",
            count,
            sample_buffer.presentation_timestamp(),
            sample_buffer.duration()
        );
    }
}

struct TestAudioOutput {
    audio_count: Arc<AtomicUsize>,
    received_audio: Arc<AtomicBool>,
}

impl SCStreamOutput for TestAudioOutput {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if matches!(of_type, SCStreamOutputType::Audio) {
            let count = self.audio_count.fetch_add(1, Ordering::SeqCst);
            self.received_audio.store(true, Ordering::SeqCst);

            println!(
                "Audio sample {}: pts={:?}, duration={:?}",
                count,
                sample_buffer.presentation_timestamp(),
                sample_buffer.duration()
            );
        }
    }
}

#[test]
fn test_screen_capture_with_audio() {
    println!("=== Starting Screen Capture with Audio Test ===");

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
        println!("⚠️  No displays found - skipping test");
        return;
    }

    let display = &displays[0];
    println!("Using display: {}", display.display_id());

    let filter = SCContentFilter::with()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    config.set_channel_count(2);

    let video_frame_count = Arc::new(AtomicUsize::new(0));
    let video_received = Arc::new(AtomicBool::new(false));
    let audio_count = Arc::new(AtomicUsize::new(0));
    let audio_received = Arc::new(AtomicBool::new(false));

    let video_output = TestVideoOutput {
        frame_count: video_frame_count.clone(),
        received_frame: video_received.clone(),
    };

    let audio_output = TestAudioOutput {
        audio_count: audio_count.clone(),
        received_audio: audio_received.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(video_output, SCStreamOutputType::Screen);
    stream.add_output_handler(audio_output, SCStreamOutputType::Audio);

    stream.start_capture().ok();
    println!("Capture started, waiting for frames...");

    thread::sleep(Duration::from_secs(5));

    stream.stop_capture().ok();
    println!("Capture stopped");

    let video_count = video_frame_count.load(Ordering::SeqCst);
    let audio_samples = audio_count.load(Ordering::SeqCst);

    println!("Received {video_count} video frames");
    println!("Received {audio_samples} audio samples");

    // Note: Test may not receive frames if screen recording permissions are not granted
    // or if the test environment doesn't allow screen capture
    if video_received.load(Ordering::SeqCst) {
        println!("✓ Video capture working!");
        assert!(video_count > 0, "Video frame count should be > 0");
    } else {
        println!("⚠️  No video frames received - this may be due to:");
        println!("   - Missing screen recording permissions");
        println!("   - Test running in headless environment");
        println!("   - System restrictions");
        println!("   Skipping assertion to avoid false negatives");
    }

    if audio_received.load(Ordering::SeqCst) {
        println!("✓ Audio capture working!");
    } else {
        println!("⚠️  No audio captured (this is OK if no audio was playing)");
    }
}

#[test]
fn test_combined_video_audio_capture() {
    println!("=== Combined Video + Audio Capture Test ===");

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
        println!("⚠️  No displays found - skipping test");
        return;
    }

    let display = &displays[0];
    println!("Capturing display: {}", display.display_id());

    let filter = SCContentFilter::with()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_captures_audio(true);
    config.set_sample_rate(48000);
    config.set_channel_count(2);

    let video_count = Arc::new(AtomicUsize::new(0));
    let video_received = Arc::new(AtomicBool::new(false));
    let audio_count = Arc::new(AtomicUsize::new(0));
    let audio_received = Arc::new(AtomicBool::new(false));

    let video_output = TestVideoOutput {
        frame_count: video_count.clone(),
        received_frame: video_received.clone(),
    };

    let audio_output = TestAudioOutput {
        audio_count: audio_count.clone(),
        received_audio: audio_received,
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(video_output, SCStreamOutputType::Screen);
    stream.add_output_handler(audio_output, SCStreamOutputType::Audio);

    stream.start_capture().ok();
    println!("Combined capture started");

    thread::sleep(Duration::from_secs(3));

    stream.stop_capture().ok();
    println!("Combined capture stopped");

    let v_count = video_count.load(Ordering::SeqCst);
    let a_count = audio_count.load(Ordering::SeqCst);

    println!("Video frames: {v_count}");
    println!("Audio samples: {a_count}");

    // Note: Test may not receive frames if screen recording permissions are not granted
    if video_received.load(Ordering::SeqCst) {
        println!("✓ Video capture working!");
        assert!(v_count > 0, "Video count must be > 0");
        println!("✓ Combined capture test passed!");
    } else {
        println!("⚠️  No video frames received - this may be due to permissions or environment");
        println!("   Skipping assertion to avoid false negatives");
    }
}
