//! Targeted micro-benchmarks for the hotspots identified in the perf review.
//!
//! Run with: `cargo bench --bench hotspots --features macos_14_0`
//!
//! Each benchmark probes a specific issue called out in the review so we can
//! quantify the win before/after a fix.

#![allow(clippy::cast_possible_truncation)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use screencapturekit::cm::CMSampleBuffer;
use screencapturekit::prelude::*;
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn cg_init() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

// ============================================================================
// Hotspot #5: per-element FFI for shareable_content.windows()
// Status quo: 1 count call + N get_at calls, then per-window N×attr calls.
// ============================================================================

fn bench_shareable_content_enumeration(c: &mut Criterion) {
    cg_init();
    let content = SCShareableContent::get().expect("perms?");
    let n_windows = content.windows().len();
    let n_displays = content.displays().len();
    let n_apps = content.applications().len();
    eprintln!("Test environment: {n_windows} windows, {n_displays} displays, {n_apps} apps");

    let mut group = c.benchmark_group("shareable_content");

    // Just the list-of-pointers fetch (1 + N FFI calls).
    group.bench_function("windows_list_only", |b| {
        b.iter(|| {
            let w = content.windows();
            black_box(w);
        });
    });

    group.bench_function("displays_list_only", |b| {
        b.iter(|| {
            let d = content.displays();
            black_box(d);
        });
    });

    group.bench_function("applications_list_only", |b| {
        b.iter(|| {
            let a = content.applications();
            black_box(a);
        });
    });

    // Realistic enumeration: fetch list + read every attribute every consumer
    // typically wants (title, frame, owning_app, layer, on_screen) — current
    // code does ~6 FFI calls per window on top of the list fetch.
    group.bench_function("windows_full_attrs", |b| {
        b.iter(|| {
            let windows = content.windows();
            for w in &windows {
                black_box(w.window_id());
                black_box(w.title());
                black_box(w.frame());
                black_box(w.window_layer());
                black_box(w.is_on_screen());
                black_box(w.owning_application());
            }
        });
    });

    // Hotspot #6: Debug impl on SCShareableContent runs three full enumerations.
    group.bench_function("debug_format", |b| {
        b.iter(|| {
            let s = format!("{content:?}");
            black_box(s);
        });
    });

    // Hotspot #2 (post-fix): batched snapshot — populates everything in one
    // FFI round-trip per category instead of 1+N+6N.
    group.bench_function("snapshot_full", |b| {
        b.iter(|| {
            let snap = content.snapshot();
            black_box(snap);
        });
    });

    group.finish();
}

// ============================================================================
// Hotspot #7: SCContentFilterBuilder per-element clone()/retain() cost.
// ============================================================================

fn bench_content_filter_with_excludes(c: &mut Criterion) {
    cg_init();
    let content = SCShareableContent::get().expect("perms?");
    let displays = content.displays();
    let display = displays.first().expect("no display");
    let windows = content.windows();
    // Take refs so we mimic the public API shape `&[&SCWindow]`.
    let win_refs: Vec<&_> = windows.iter().collect();

    let mut group = c.benchmark_group("content_filter");

    for &n in &[0usize, 10, 50, 200] {
        let n_actual = n.min(win_refs.len());
        let slice = &win_refs[..n_actual];
        group.bench_with_input(
            BenchmarkId::new("excluding_windows", n_actual),
            slice,
            |b, slice| {
                b.iter(|| {
                    let f = SCContentFilter::create()
                        .with_display(display)
                        .with_excluding_windows(slice)
                        .build();
                    black_box(f);
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Capture a single sample buffer for the per-frame hotspots.
// ============================================================================

fn capture_single_frame() -> Option<CMSampleBuffer> {
    cg_init();
    let content = SCShareableContent::get().ok()?;
    let display = content.displays().into_iter().next()?;

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
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

// ============================================================================
// Hotspot #1: Frame attachment lookups re-fetch the attachment array.
// Each of frame_status / display_time / scale_factor / content_rect /
// bounding_rect / screen_rect / dirty_rects calls
// CMSampleBufferGetSampleAttachmentsArray + Swift bridging cast independently.
// ============================================================================

fn bench_frame_attachments(c: &mut Criterion) {
    let Some(sample) = capture_single_frame() else {
        eprintln!("Skipping frame_attachments — no frame captured.");
        return;
    };

    let mut group = c.benchmark_group("frame_attachments");

    group.bench_function("frame_status_only", |b| {
        b.iter(|| black_box(sample.frame_status()));
    });

    group.bench_function("display_time_only", |b| {
        b.iter(|| black_box(sample.display_time()));
    });

    group.bench_function("content_rect_only", |b| {
        b.iter(|| black_box(sample.content_rect()));
    });

    // Realistic per-frame consumer: read every attachment.
    group.bench_function("read_all_5_attachments", |b| {
        b.iter(|| {
            black_box(sample.frame_status());
            black_box(sample.display_time());
            black_box(sample.scale_factor());
            black_box(sample.content_rect());
            black_box(sample.bounding_rect());
        });
    });

    // Hotspot #1 (post-fix): batched FrameInfo — single FFI fetch.
    group.bench_function("read_all_via_frame_info", |b| {
        b.iter(|| black_box(sample.frame_info()));
    });

    group.finish();
}

// ============================================================================
// Hotspot #11: sample_timing_info_array makes N FFI calls (1 per sample).
// For a screen frame numSamples is 1, but for audio it's the frame count.
// ============================================================================

fn bench_sample_timing(c: &mut Criterion) {
    let Some(sample) = capture_single_frame() else {
        return;
    };
    let mut group = c.benchmark_group("sample_timing");
    group.bench_function("single_info", |b| {
        b.iter(|| black_box(sample.sample_timing_info(0).ok()));
    });
    group.bench_function("info_array", |b| {
        b.iter(|| black_box(sample.sample_timing_info_array().ok()));
    });
    group.finish();
}

// ============================================================================
// Hotspot #4: CGImage::rgba_data() does 3 full RGBA copies.
// Isolate that one call on a real screenshot.
// ============================================================================

#[cfg(feature = "macos_14_0")]
fn bench_screenshot_rgba(c: &mut Criterion) {
    use screencapturekit::screenshot_manager::SCScreenshotManager;
    cg_init();
    let content = SCShareableContent::get().expect("perms?");
    let display = content.displays().into_iter().next().expect("no display");
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let mut group = c.benchmark_group("screenshot_rgba");
    group.sample_size(20); // each sample = 1 capture + 1 rgba_data

    for &(label, w, h) in &[("1080p", 1920u32, 1080u32), ("4k", 3840u32, 2160u32)] {
        let config = SCStreamConfiguration::new()
            .with_width(w)
            .with_height(h)
            .with_pixel_format(PixelFormat::BGRA);
        // Verify we can capture before benching.
        let Ok(_) = SCScreenshotManager::capture_image(&filter, &config) else {
            eprintln!("Skip {label}: cannot capture");
            continue;
        };

        group.bench_with_input(
            BenchmarkId::new("capture_only", label),
            &(&filter, &config),
            |b, (f, cfg)| {
                b.iter(|| {
                    let img = SCScreenshotManager::capture_image(f, cfg).unwrap();
                    black_box(img);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("capture_plus_rgba", label),
            &(&filter, &config),
            |b, (f, cfg)| {
                b.iter(|| {
                    let img = SCScreenshotManager::capture_image(f, cfg).unwrap();
                    let data = img.rgba_data().unwrap();
                    black_box(data);
                });
            },
        );
    }

    group.finish();
}

#[cfg(not(feature = "macos_14_0"))]
fn bench_screenshot_rgba(_c: &mut Criterion) {}

// ============================================================================
// Hotspot #10: copy_data_bytes pre-zeroes the Vec.
// Hotspot #9: cursor() always copies even when contiguous.
// We can drive this on the pixel buffer's plane row to compare lock-as_slice
// (zero-copy) vs synthesised copy paths.
// ============================================================================

fn bench_pixel_buffer_paths(c: &mut Criterion) {
    let Some(sample) = capture_single_frame() else {
        return;
    };
    let Some(pb) = sample.image_buffer() else {
        return;
    };

    let mut group = c.benchmark_group("pixel_buffer_paths");
    use screencapturekit::cv::CVPixelBufferLockFlags;

    // Zero-copy path: lock + as_slice.
    group.bench_function("lock_as_slice_zero_copy", |b| {
        b.iter(|| {
            let g = pb.lock(CVPixelBufferLockFlags::READ_ONLY).unwrap();
            black_box(g.as_slice().len());
        });
    });

    // Cost of a 1080p RGBA copy to demonstrate the savings if we eliminated
    // any unnecessary copy of similar-size buffers.
    let bytes = 1920 * 1080 * 4;
    group.bench_function("memset_then_copy_1080p_rgba", |b| {
        let g = pb.lock(CVPixelBufferLockFlags::READ_ONLY).unwrap();
        let src = g.as_slice();
        let n = bytes.min(src.len());
        b.iter(|| {
            // emulates `vec![0u8; n]` + memcpy (the current copy_data_bytes
            // shape) at a representative size.
            let mut v = vec![0u8; n];
            v.copy_from_slice(&src[..n]);
            black_box(v);
        });
    });
    group.bench_function("alloc_uninit_then_copy_1080p_rgba", |b| {
        let g = pb.lock(CVPixelBufferLockFlags::READ_ONLY).unwrap();
        let src = g.as_slice();
        let n = bytes.min(src.len());
        b.iter(|| {
            // emulates the proposed Vec::with_capacity + set_len after copy
            // (no memset).
            let mut v: Vec<u8> = Vec::with_capacity(n);
            unsafe {
                std::ptr::copy_nonoverlapping(src.as_ptr(), v.as_mut_ptr(), n);
                v.set_len(n);
            }
            black_box(v);
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(1));
    targets =
        bench_shareable_content_enumeration,
        bench_content_filter_with_excludes,
        bench_frame_attachments,
        bench_sample_timing,
        bench_screenshot_rgba,
        bench_pixel_buffer_paths,
}
criterion_main!(benches);
