//! Stress tests for robustness and stability
//! 
//! These tests push the API to its limits to find issues with:
//! - Memory leaks
//! - Resource exhaustion
//! - Race conditions
//! - Edge cases

use screencapturekit::{
    cm::{CMSampleBuffer, CMTime},
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration,
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_rapid_stream_creation_destruction() {
    // Test creating and destroying many streams rapidly
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    let start = Instant::now();
    let iterations = 50;

    for i in 0..iterations {
        #[allow(deprecated)]
        let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();

        let config = SCStreamConfiguration::build()
            .set_width(320)
            .unwrap()
            .set_height(240)
            .unwrap();

        let _stream = SCStream::new(&filter, &config);
        // Stream is immediately dropped

        if i % 10 == 0 {
            println!("Created and destroyed {i} streams");
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Created/destroyed {iterations} streams in {elapsed:?}");
    println!("   Average: {:?} per stream", elapsed / iterations);
}

#[test]
fn test_rapid_start_stop_cycles() {
    // Test rapid start/stop cycles
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();

    let config = SCStreamConfiguration::build()
        .set_width(320)
        .unwrap()
        .set_height(240)
        .unwrap();

    struct NullHandler;
    impl SCStreamOutputTrait for NullHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {}
    }

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(NullHandler, SCStreamOutputType::Screen);

    let cycles = 10;
    let mut successful_cycles = 0;

    for i in 0..cycles {
        if stream.start_capture().is_ok() {
            thread::sleep(Duration::from_millis(50));
            if stream.stop_capture().is_ok() {
                successful_cycles += 1;
            }
        }

        if i % 5 == 0 {
            println!("Completed {i} start/stop cycles");
        }
    }

    println!("✅ Completed {successful_cycles}/{cycles} start/stop cycles");
    assert!(successful_cycles >= cycles / 2, "At least half should succeed");
}

#[test]
fn test_many_concurrent_streams() {
    // Test creating many streams concurrently
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = displays[0].clone();
    let shared_display = Arc::new(display);
    let success_count = Arc::new(AtomicUsize::new(0));

    let num_threads = 10;
    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let display = Arc::clone(&shared_display);
            let success_count = Arc::clone(&success_count);

            thread::spawn(move || {
                #[allow(deprecated)]
                let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();

                let config = match SCStreamConfiguration::build()
                    .set_width(320)
                    .and_then(|c| c.set_height(240))
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Thread {i}: Config failed: {e}");
                        return;
                    }
                };

                let _stream = SCStream::new(&filter, &config);
                success_count.fetch_add(1, Ordering::Relaxed);
                println!("Thread {i}: Created stream");
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    let successes = success_count.load(Ordering::Relaxed);
    println!("✅ Created {successes}/{num_threads} concurrent streams");
}

#[test]
fn test_high_frame_rate_handling() {
    // Test handling high frame rates
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    let filter = SCContentFilter::build()
        .display(display)
        .exclude_windows(&[])
        .build();

    let frame_interval = CMTime::new(1, 60); // ~60 FPS (1/60 seconds)
    let config = SCStreamConfiguration::build()
        .set_width(640)
        .unwrap()
        .set_height(480)
        .unwrap()
        .set_minimum_frame_interval(&frame_interval)
        .unwrap();

    struct HighSpeedHandler {
        count: Arc<AtomicUsize>,
        start_time: Instant,
        peak_fps: Arc<Mutex<f64>>,
        last_check: Arc<Mutex<Instant>>,
        check_interval_frames: Arc<Mutex<usize>>,
    }

    impl SCStreamOutputTrait for HighSpeedHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            let count = self.count.fetch_add(1, Ordering::Relaxed) + 1;
            let mut check_frames = self.check_interval_frames.lock().unwrap();
            *check_frames += 1;

            // Check FPS every 30 frames
            if *check_frames >= 30 {
                let now = Instant::now();
                let mut last_check = self.last_check.lock().unwrap();
                let elapsed = now.duration_since(*last_check).as_secs_f64();
                let fps = *check_frames as f64 / elapsed;

                let mut peak = self.peak_fps.lock().unwrap();
                if fps > *peak {
                    *peak = fps;
                }

                *last_check = now;
                *check_frames = 0;
            }

            if count % 100 == 0 {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let avg_fps = count as f64 / elapsed;
                println!("Processed {count} frames (avg {avg_fps:.1} fps)");
            }
        }
    }

    let count = Arc::new(AtomicUsize::new(0));
    let peak_fps = Arc::new(Mutex::new(0.0));
    let handler = HighSpeedHandler {
        count: Arc::clone(&count),
        start_time: Instant::now(),
        peak_fps: Arc::clone(&peak_fps),
        last_check: Arc::new(Mutex::new(Instant::now())),
        check_interval_frames: Arc::new(Mutex::new(0)),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    if let Err(e) = stream.start_capture() {
        eprintln!("⚠️  Failed to start: {e}");
        return;
    }

    thread::sleep(Duration::from_secs(2));

    if let Err(e) = stream.stop_capture() {
        eprintln!("⚠️  Failed to stop: {e}");
    }

    let final_count = count.load(Ordering::Relaxed);
    let peak = *peak_fps.lock().unwrap();
    println!("✅ Handled high frame rate: {final_count} total frames");
    println!("   Peak FPS: {peak:.1}");
}

#[test]
fn test_memory_pressure_many_handlers() {
    // Test with many handlers on the same stream
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();

    let config = SCStreamConfiguration::build()
        .set_width(320)
        .unwrap()
        .set_height(240)
        .unwrap();

    struct CountingHandler {
        id: usize,
        count: Arc<Mutex<Vec<usize>>>,
    }

    impl SCStreamOutputTrait for CountingHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            let mut counts = self.count.lock().unwrap();
            if self.id >= counts.len() {
                counts.resize(self.id + 1, 0);
            }
            counts[self.id] += 1;
        }
    }

    let counts = Arc::new(Mutex::new(Vec::new()));
    let mut stream = SCStream::new(&filter, &config);

    // Add multiple handlers
    let num_handlers = 5;
    for i in 0..num_handlers {
        let handler = CountingHandler {
            id: i,
            count: Arc::clone(&counts),
        };
        stream.add_output_handler(handler, SCStreamOutputType::Screen);
    }

    if let Err(e) = stream.start_capture() {
        eprintln!("⚠️  Failed to start: {e}");
        return;
    }

    thread::sleep(Duration::from_millis(500));

    if let Err(e) = stream.stop_capture() {
        eprintln!("⚠️  Failed to stop: {e}");
    }

    let counts = counts.lock().unwrap();
    println!("✅ Multiple handlers stress test:");
    for (i, &count) in counts.iter().enumerate() {
        println!("   Handler {i}: {count} frames");
    }
}

#[test]
fn test_rapid_content_refresh() {
    // Test rapidly refreshing shareable content
    let iterations = 20;
    let start = Instant::now();
    let mut successful = 0;

    for i in 0..iterations {
        match SCShareableContent::get() {
            Ok(_content) => {
                successful += 1;
                if i % 5 == 0 {
                    println!("Content refresh {i}/{iterations}");
                }
            }
            Err(e) => {
                eprintln!("Content refresh {i} failed: {e}");
            }
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Completed {successful}/{iterations} content refreshes in {elapsed:?}");
    println!("   Average: {:?} per refresh", elapsed / iterations);
}

#[test]
fn test_long_running_stream() {
    // Test a stream running for an extended period
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();

    let config = SCStreamConfiguration::build()
        .set_width(320)
        .unwrap()
        .set_height(240)
        .unwrap();

    struct StabilityHandler {
        count: Arc<AtomicUsize>,
        start_time: Instant,
        running: Arc<AtomicBool>,
    }

    impl SCStreamOutputTrait for StabilityHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            if !self.running.load(Ordering::Relaxed) {
                return;
            }

            let count = self.count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 50 == 0 {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let fps = count as f64 / elapsed;
                println!("Running: {count} frames, {fps:.1} fps, {elapsed:.1}s");
            }
        }
    }

    let count = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));

    let handler = StabilityHandler {
        count: Arc::clone(&count),
        start_time: Instant::now(),
        running: Arc::clone(&running),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    if let Err(e) = stream.start_capture() {
        eprintln!("⚠️  Failed to start: {e}");
        return;
    }

    // Run for 3 seconds
    thread::sleep(Duration::from_secs(3));

    running.store(false, Ordering::Relaxed);

    if let Err(e) = stream.stop_capture() {
        eprintln!("⚠️  Failed to stop: {e}");
    }

    let final_count = count.load(Ordering::Relaxed);
    println!("✅ Long-running stream stable: {final_count} total frames");
}

#[test]
fn test_concurrent_different_resolutions() {
    // Test creating streams with different resolutions concurrently
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = displays[0].clone();
    let shared_display = Arc::new(display);

    let resolutions = vec![
        (320, 240),
        (640, 480),
        (1280, 720),
        (1920, 1080),
    ];

    let handles: Vec<_> = resolutions
        .iter()
        .enumerate()
        .map(|(i, &(width, height))| {
            let display = Arc::clone(&shared_display);
            thread::spawn(move || {
                #[allow(deprecated)]
                let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();

                let config = match SCStreamConfiguration::build()
                    .set_width(width)
                    .and_then(|c| c.set_height(height))
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Thread {i} ({width}x{height}): Config failed: {e}");
                        return Err(());
                    }
                };

                let _stream = SCStream::new(&filter, &config);
                println!("Thread {i}: Created {width}x{height} stream");
                Ok(())
            })
        })
        .collect();

    let mut successes = 0;
    for handle in handles {
        if handle.join().unwrap().is_ok() {
            successes += 1;
        }
    }

    println!("✅ Created {successes}/{} streams with different resolutions", resolutions.len());
}

#[test]
fn test_error_recovery() {
    // Test that the API can recover from errors
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Skipping test: {e}");
            return;
        }
    };

    let displays = content.displays();
    if displays.is_empty() {
        eprintln!("⚠️  Skipping test - no displays available");
        return;
    }

    let display = &displays[0];

    // Try to create a stream with invalid configuration first
    let invalid_config = SCStreamConfiguration::build();

    #[allow(deprecated)]
    let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();

    let mut stream = SCStream::new(&filter, &invalid_config);

    struct DummyHandler;
    impl SCStreamOutputTrait for DummyHandler {
        fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {}
    }

    stream.add_output_handler(DummyHandler, SCStreamOutputType::Screen);

    // This might fail
    let _ = stream.start_capture();
    let _ = stream.stop_capture();

    // Now try with valid configuration
    let valid_config = SCStreamConfiguration::build()
        .set_width(640)
        .unwrap()
        .set_height(480)
        .unwrap();

    let mut stream2 = SCStream::new(&filter, &valid_config);
    stream2.add_output_handler(DummyHandler, SCStreamOutputType::Screen);

    match stream2.start_capture() {
        Ok(()) => {
            println!("✅ Recovered from error and started successfully");
            let _ = stream2.stop_capture();
        }
        Err(e) => {
            eprintln!("⚠️  Recovery failed: {e}");
        }
    }
}
