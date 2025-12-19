//! Benchmarks for screen capture operations
//!
//! Run with: `cargo bench`
//!
//! These benchmarks require screen recording permission and a display to capture.
//!
//! # Benchmark Categories
//!
//! 1. **API Overhead** - Measures the cost of creating objects (filter, config, etc.)
//! 2. **Frame Capture** - Measures throughput and latency of actual frame capture
//! 3. **Data Access** - Measures pixel buffer and `IOSurface` access patterns
//! 4. **Screenshots** - Measures single-frame capture performance (macOS 14.0+)

#![allow(clippy::cast_possible_truncation)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use screencapturekit::cm::CMTime;
use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes
fn cg_init() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

// =============================================================================
// API Overhead Benchmarks
// =============================================================================

fn bench_shareable_content_get(c: &mut Criterion) {
    cg_init();

    c.bench_function("api/SCShareableContent::get", |b| {
        b.iter(|| {
            let content = SCShareableContent::get();
            black_box(content)
        });
    });
}

fn bench_shareable_content_with_options(c: &mut Criterion) {
    cg_init();

    c.bench_function("api/SCShareableContent::create", |b| {
        b.iter(|| {
            let content = SCShareableContent::create()
                .with_exclude_desktop_windows(true)
                .with_on_screen_windows_only(true)
                .get();
            black_box(content)
        });
    });
}

fn bench_content_filter_creation(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    c.bench_function("api/SCContentFilter::create", |b| {
        b.iter(|| {
            let filter = SCContentFilter::create()
                .with_display(&display)
                .with_excluding_windows(&[])
                .build();
            black_box(filter)
        });
    });
}

fn bench_stream_configuration_creation(c: &mut Criterion) {
    c.bench_function("api/SCStreamConfiguration::new", |b| {
        b.iter(|| {
            let config = SCStreamConfiguration::new()
                .with_width(1920)
                .with_height(1080)
                .with_shows_cursor(true);
            black_box(config)
        });
    });
}

fn bench_stream_creation(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    c.bench_function("api/SCStream::new", |b| {
        b.iter(|| {
            let stream = SCStream::new(&filter, &config);
            black_box(stream)
        });
    });
}

// =============================================================================
// Frame Capture Benchmarks
// =============================================================================

/// Benchmark frame capture throughput at various resolutions
fn bench_frame_throughput(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    let resolutions: [(u32, u32, &str); 4] = [
        (640, 480, "480p"),
        (1280, 720, "720p"),
        (1920, 1080, "1080p"),
        (3840, 2160, "4K"),
    ];

    let mut group = c.benchmark_group("throughput");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    for (width, height, label) in resolutions {
        // Skip 4K if display is smaller
        if width > display.width() || height > display.height() {
            continue;
        }

        let pixels = u64::from(width * height);
        group.throughput(Throughput::Elements(pixels));

        group.bench_with_input(
            BenchmarkId::new("frames", label),
            &(width, height),
            |b, &(w, h)| {
                let filter = SCContentFilter::create()
                    .with_display(&display)
                    .with_excluding_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(w)
                    .with_height(h)
                    .with_pixel_format(PixelFormat::BGRA)
                    .with_minimum_frame_interval(&CMTime::new(1, 120)); // Request 120 FPS max

                let frame_count = Arc::new(AtomicUsize::new(0));
                let frame_count_clone = Arc::clone(&frame_count);

                let handler = move |_sample: CMSampleBuffer, output_type: SCStreamOutputType| {
                    if matches!(output_type, SCStreamOutputType::Screen) {
                        frame_count_clone.fetch_add(1, Ordering::Relaxed);
                    }
                };

                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        frame_count.store(0, Ordering::Relaxed);

                        let mut stream = SCStream::new(&filter, &config);
                        stream.add_output_handler(handler.clone(), SCStreamOutputType::Screen);

                        let start = Instant::now();
                        stream.start_capture().expect("Failed to start capture");

                        // Capture for a fixed duration
                        std::thread::sleep(Duration::from_millis(100));

                        stream.stop_capture().expect("Failed to stop capture");
                        total_duration += start.elapsed();

                        let frames = frame_count.load(Ordering::Relaxed);
                        black_box(frames);
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark frame latency (time from capture request to callback)
fn bench_frame_latency(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    let mut group = c.benchmark_group("latency");
    group.sample_size(20);

    group.bench_function("first_frame", |b| {
        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(1920)
            .with_height(1080)
            .with_pixel_format(PixelFormat::BGRA);

        b.iter_custom(|iters| {
            let mut total_latency = Duration::ZERO;

            for _ in 0..iters {
                let first_frame_time = Arc::new(AtomicU64::new(0));
                let first_frame_time_clone = Arc::clone(&first_frame_time);
                let start_time = Instant::now();

                let handler = move |_sample: CMSampleBuffer, output_type: SCStreamOutputType| {
                    if matches!(output_type, SCStreamOutputType::Screen) {
                        // Only record the first frame
                        first_frame_time_clone
                            .compare_exchange(
                                0,
                                start_time.elapsed().as_nanos() as u64,
                                Ordering::SeqCst,
                                Ordering::Relaxed,
                            )
                            .ok();
                    }
                };

                let mut stream = SCStream::new(&filter, &config);
                stream.add_output_handler(handler, SCStreamOutputType::Screen);

                stream.start_capture().expect("Failed to start capture");

                // Wait for first frame (with timeout)
                let timeout = Duration::from_secs(2);
                let poll_start = Instant::now();
                while first_frame_time.load(Ordering::Relaxed) == 0 {
                    if poll_start.elapsed() > timeout {
                        break;
                    }
                    std::thread::sleep(Duration::from_micros(100));
                }

                stream.stop_capture().expect("Failed to stop capture");

                let latency_nanos = first_frame_time.load(Ordering::Relaxed);
                if latency_nanos > 0 {
                    total_latency += Duration::from_nanos(latency_nanos);
                }
            }

            total_latency
        });
    });

    group.finish();
}

// =============================================================================
// Data Access Benchmarks
// =============================================================================

/// Benchmark pixel buffer locking and access patterns
fn bench_pixel_buffer_access(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    // Capture a single frame to benchmark access patterns
    let sample_buffer: Arc<std::sync::Mutex<Option<CMSampleBuffer>>> =
        Arc::new(std::sync::Mutex::new(None));
    let sample_buffer_clone = Arc::clone(&sample_buffer);
    let captured = Arc::new(AtomicUsize::new(0));
    let captured_clone = Arc::clone(&captured);

    let handler = move |sample: CMSampleBuffer, output_type: SCStreamOutputType| {
        if matches!(output_type, SCStreamOutputType::Screen)
            && captured_clone
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
        {
            *sample_buffer_clone.lock().unwrap() = Some(sample);
        }
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture().expect("Failed to start capture");

    // Wait for a frame
    let timeout = Duration::from_secs(2);
    let start = Instant::now();
    while captured.load(Ordering::Relaxed) == 0 && start.elapsed() < timeout {
        std::thread::sleep(Duration::from_millis(10));
    }

    stream.stop_capture().expect("Failed to stop capture");

    let sample = sample_buffer.lock().unwrap().take();
    let Some(sample) = sample else {
        eprintln!("Warning: Could not capture frame for pixel buffer benchmarks");
        return;
    };

    let Some(pixel_buffer) = sample.image_buffer() else {
        eprintln!("Warning: No pixel buffer in captured frame");
        return;
    };

    let mut group = c.benchmark_group("data_access");

    // Benchmark lock/unlock cycle
    group.bench_function("pixel_buffer/lock_unlock", |b| {
        b.iter(|| {
            let guard = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY);
            black_box(guard)
        });
    });

    // Benchmark reading first pixel
    group.bench_function("pixel_buffer/read_first_pixel", |b| {
        b.iter(|| {
            if let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) {
                let slice = guard.as_slice();
                if slice.len() >= 4 {
                    let pixel: [u8; 4] = [slice[0], slice[1], slice[2], slice[3]];
                    black_box(pixel);
                }
            }
        });
    });

    // Benchmark reading all pixels (simulate full-frame processing)
    group.bench_function("pixel_buffer/read_all_pixels", |b| {
        b.iter(|| {
            if let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) {
                let slice = guard.as_slice();
                // Simulate processing by computing a checksum
                let sum: u64 = slice.iter().step_by(1024).map(|&b| u64::from(b)).sum();
                black_box(sum);
            }
        });
    });

    // Benchmark IOSurface access if available
    if pixel_buffer.is_backed_by_io_surface() {
        if let Some(iosurface) = pixel_buffer.io_surface() {
            group.bench_function("iosurface/lock_unlock", |b| {
                use screencapturekit::cm::IOSurfaceLockOptions;
                b.iter(|| {
                    let guard = iosurface.lock(IOSurfaceLockOptions::READ_ONLY);
                    black_box(guard)
                });
            });

            group.bench_function("iosurface/get_properties", |b| {
                b.iter(|| {
                    let width = iosurface.width();
                    let height = iosurface.height();
                    let format = iosurface.pixel_format();
                    let bpr = iosurface.bytes_per_row();
                    black_box((width, height, format, bpr));
                });
            });
        }
    }

    group.finish();
}

// =============================================================================
// Screenshot Benchmarks (macOS 14.0+)
// =============================================================================

#[cfg(feature = "macos_14_0")]
fn bench_screenshot_capture(c: &mut Criterion) {
    use screencapturekit::screenshot_manager::SCScreenshotManager;

    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let resolutions: [(u32, u32, &str); 3] = [
        (640, 480, "480p"),
        (1920, 1080, "1080p"),
        (3840, 2160, "4K"),
    ];

    let mut group = c.benchmark_group("screenshot");
    group.sample_size(20);

    for (width, height, label) in resolutions {
        // Skip if larger than display
        if width > display.width() || height > display.height() {
            continue;
        }

        let config = SCStreamConfiguration::new()
            .with_width(width)
            .with_height(height);

        group.bench_with_input(BenchmarkId::new("capture_image", label), &(), |b, ()| {
            b.iter(|| {
                let result = SCScreenshotManager::capture_image(&filter, &config);
                black_box(result)
            });
        });

        group.bench_with_input(
            BenchmarkId::new("capture_sample_buffer", label),
            &(),
            |b, ()| {
                b.iter(|| {
                    let result = SCScreenshotManager::capture_sample_buffer(&filter, &config);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

#[cfg(not(feature = "macos_14_0"))]
fn bench_screenshot_capture(_c: &mut Criterion) {
    // Screenshot manager requires macOS 14.0+
}

// =============================================================================
// Stream Lifecycle Benchmarks
// =============================================================================

fn bench_stream_lifecycle(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    let mut group = c.benchmark_group("lifecycle");
    group.sample_size(20);

    group.bench_function("start_stop_cycle", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let stream = SCStream::new(&filter, &config);

                let start = Instant::now();
                stream.start_capture().expect("Failed to start");
                stream.stop_capture().expect("Failed to stop");
                total += start.elapsed();

                black_box(&stream);
            }

            total
        });
    });

    group.bench_function("start_capture_only", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let stream = SCStream::new(&filter, &config);

                let start = Instant::now();
                stream.start_capture().expect("Failed to start");
                total += start.elapsed();

                stream.stop_capture().expect("Failed to stop");
                black_box(&stream);
            }

            total
        });
    });

    group.finish();
}

// =============================================================================
// Configuration Update Benchmarks
// =============================================================================

fn bench_configuration_updates(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    let mut group = c.benchmark_group("updates");
    group.sample_size(20);

    group.bench_function("update_configuration", |b| {
        let stream = SCStream::new(&filter, &config);
        stream.start_capture().expect("Failed to start");

        b.iter(|| {
            let new_config = SCStreamConfiguration::new()
                .with_width(1280)
                .with_height(720);
            let result = stream.update_configuration(&new_config);
            black_box(result)
        });

        stream.stop_capture().expect("Failed to stop");
    });

    group.finish();
}

criterion_group!(
    benches,
    // API overhead
    bench_shareable_content_get,
    bench_shareable_content_with_options,
    bench_content_filter_creation,
    bench_stream_configuration_creation,
    bench_stream_creation,
    // Frame capture
    bench_frame_throughput,
    bench_frame_latency,
    // Data access
    bench_pixel_buffer_access,
    // Screenshots
    bench_screenshot_capture,
    // Lifecycle
    bench_stream_lifecycle,
    bench_configuration_updates,
);

criterion_main!(benches);
