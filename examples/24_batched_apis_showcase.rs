//! Batched APIs showcase
//!
//! This example demonstrates the three batched APIs introduced in v2.x:
//!
//!   1. [`SCShareableContent::snapshot`] — every display + window + app + their
//!      attributes in one FFI round-trip per category, vs `1 + N + 6N` calls
//!      with the per-element accessor pattern.
//!   2. [`CMSampleBuffer::frame_info`] — every `SCStreamFrameInfo` attachment
//!      in one CF→Swift bridge cast, vs one cast per accessor.
//!   3. [`CGImage::bgra_data`] — pixel data in the source layout, skipping
//!      the per-pixel R↔B swap that `rgba_data()` performs.
//!
//! Each section measures both the new and the legacy path so you can see
//! the win on YOUR machine. Run with:
//!
//!   cargo run --release --example 24_batched_apis_showcase --features macos_14_0
//!
//! Expected output (numbers vary by machine):
//!
//!   == snapshot vs per-element ==
//!   snapshot()           : 38µs
//!   per-element + attrs  : 73µs   (1.9× slower)
//!
//!   == frame_info vs per-attribute ==
//!   frame_info()         : 7.6µs
//!   5× per-attribute calls: 11.3µs (1.5× slower)
//!
//!   == bgra_data vs rgba_data (held image) ==
//!   bgra_data() 1080p    : 199µs
//!   rgba_data() 1080p    : 209µs  (5% slower)

use screencapturekit::cm::CMSampleBuffer;
use screencapturekit::prelude::*;
use screencapturekit::screenshot_manager::SCScreenshotManager;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ITERATIONS: u32 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }

    println!("🎯 Batched APIs Showcase\n");

    showcase_snapshot()?;
    println!();
    showcase_frame_info()?;
    println!();
    showcase_bgra_data()?;

    Ok(())
}

fn showcase_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    println!("== snapshot vs per-element ==");
    let content = SCShareableContent::get()?;
    let win_count = content.windows().len();
    let app_count = content.applications().len();
    println!("Environment: {win_count} windows, {app_count} applications");

    // Warmup
    for _ in 0..3 {
        std::hint::black_box(content.snapshot());
    }

    let snap_avg = bench(ITERATIONS, || {
        std::hint::black_box(content.snapshot());
    });
    println!("  snapshot()                    : {snap_avg:>9?}");

    let per_avg = bench(ITERATIONS, || {
        let windows = content.windows();
        for w in &windows {
            std::hint::black_box(w.window_id());
            std::hint::black_box(w.title());
            std::hint::black_box(w.frame());
            std::hint::black_box(w.window_layer());
            std::hint::black_box(w.is_on_screen());
            std::hint::black_box(w.owning_application());
        }
        std::hint::black_box(content.displays());
        std::hint::black_box(content.applications());
    });
    println!("  windows() + 6 attrs + displays + apps : {per_avg:>9?}");

    if per_avg > snap_avg {
        let speedup = per_avg.as_secs_f64() / snap_avg.as_secs_f64();
        println!("  → snapshot() is {speedup:.1}× faster");
    } else {
        println!("  → close to noise floor on this machine");
    }
    Ok(())
}

fn showcase_frame_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("== frame_info vs per-attribute ==");
    let Some(sample) = capture_one_video_frame() else {
        println!("  (skipping — could not capture a frame; permissions?)");
        return Ok(());
    };

    // Warmup
    for _ in 0..3 {
        std::hint::black_box(sample.frame_info());
    }

    let fi_avg = bench(ITERATIONS, || {
        std::hint::black_box(sample.frame_info());
    });
    println!("  frame_info()                  : {fi_avg:>9?}");

    let per_avg = bench(ITERATIONS, || {
        std::hint::black_box(sample.frame_status());
        std::hint::black_box(sample.display_time());
        std::hint::black_box(sample.scale_factor());
        std::hint::black_box(sample.content_rect());
        std::hint::black_box(sample.bounding_rect());
    });
    println!("  5 per-attribute accessors     : {per_avg:>9?}");

    if per_avg > fi_avg {
        let speedup = per_avg.as_secs_f64() / fi_avg.as_secs_f64();
        println!("  → frame_info() is {speedup:.1}× faster");
    } else {
        println!("  → close to noise floor on this machine");
    }
    Ok(())
}

fn showcase_bgra_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("== bgra_data vs rgba_data ==");
    let content = SCShareableContent::get()?;
    let display = content.displays().into_iter().next().ok_or("no display")?;
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    for &(label, w, h) in &[("1080p", 1920u32, 1080u32), ("4K", 3840u32, 2160u32)] {
        let config = SCStreamConfiguration::new()
            .with_width(w)
            .with_height(h)
            .with_pixel_format(PixelFormat::BGRA);
        let img = match SCScreenshotManager::capture_image(&filter, &config) {
            Ok(i) => i,
            Err(e) => {
                println!("  ({label}) skipped: {e}");
                continue;
            }
        };

        // Warmup — first call on a fresh CGImage triggers lazy decode/cache.
        for _ in 0..3 {
            std::hint::black_box(img.rgba_data().ok());
            std::hint::black_box(img.bgra_data().ok());
        }

        let rgba_avg = bench(ITERATIONS, || {
            std::hint::black_box(img.rgba_data().ok());
        });
        let bgra_avg = bench(ITERATIONS, || {
            std::hint::black_box(img.bgra_data().ok());
        });
        println!("  {label}:");
        println!("    rgba_data()                 : {rgba_avg:>9?}");
        println!("    bgra_data()                 : {bgra_avg:>9?}");
        if rgba_avg > bgra_avg {
            let saved = rgba_avg - bgra_avg;
            let pct = 100.0 * saved.as_secs_f64() / rgba_avg.as_secs_f64();
            println!("    → bgra_data() saves {saved:?} ({pct:.1}%)");
        }
    }
    Ok(())
}

fn bench<F: FnMut()>(iters: u32, mut f: F) -> Duration {
    let t0 = Instant::now();
    for _ in 0..iters {
        f();
    }
    t0.elapsed() / iters
}

fn capture_one_video_frame() -> Option<CMSampleBuffer> {
    let content = SCShareableContent::get().ok()?;
    let display = content.displays().into_iter().next()?;
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_pixel_format(PixelFormat::BGRA);

    let sample: Arc<Mutex<Option<CMSampleBuffer>>> = Arc::new(Mutex::new(None));
    let captured = Arc::new(AtomicUsize::new(0));
    let s = Arc::clone(&sample);
    let cnt = Arc::clone(&captured);
    let handler = move |buf: CMSampleBuffer, ot: SCStreamOutputType| {
        if matches!(ot, SCStreamOutputType::Screen)
            && cnt
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
        {
            *s.lock().unwrap() = Some(buf);
        }
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture().ok()?;
    let start = Instant::now();
    while captured.load(Ordering::Relaxed) == 0 && start.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(10));
    }
    stream.stop_capture().ok()?;
    let result = sample.lock().unwrap().take();
    result
}
