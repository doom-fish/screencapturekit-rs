//! Benchmarks for screen capture operations
//!
//! Run with: `cargo bench`
//!
//! These benchmarks require screen recording permission and a display to capture.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes
fn cg_init() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

fn bench_shareable_content_get(c: &mut Criterion) {
    cg_init();

    c.bench_function("SCShareableContent::get", |b| {
        b.iter(|| {
            let content = SCShareableContent::get();
            black_box(content)
        });
    });
}

fn bench_shareable_content_with_options(c: &mut Criterion) {
    cg_init();

    c.bench_function("SCShareableContent::with_options", |b| {
        b.iter(|| {
            let content = SCShareableContent::with_options()
                .exclude_desktop_windows(true)
                .on_screen_windows_only(true)
                .get();
            black_box(content)
        });
    });
}

fn bench_content_filter_creation(c: &mut Criterion) {
    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");

    c.bench_function("SCContentFilter::builder (display)", |b| {
        b.iter(|| {
            let filter = SCContentFilter::builder()
                .display(&display)
                .exclude_windows(&[])
                .build();
            black_box(filter)
        });
    });
}

fn bench_stream_configuration_creation(c: &mut Criterion) {
    c.bench_function("SCStreamConfiguration::new", |b| {
        b.iter(|| {
            let config = SCStreamConfiguration::new()
                .with_width(1920)
                .with_height(1080)
                .with_shows_cursor(true);
            black_box(config)
        });
    });
}

#[cfg(feature = "macos_14_0")]
fn bench_screenshot_capture(c: &mut Criterion) {
    use screencapturekit::screenshot_manager::SCScreenshotManager;

    cg_init();

    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let display = content.displays().into_iter().next().expect("No display");
    let filter = SCContentFilter::builder()
        .display(&display)
        .exclude_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    c.bench_function("SCScreenshotManager::capture_image", |b| {
        b.iter(|| {
            let result = SCScreenshotManager::capture_image(&filter, &config);
            black_box(result)
        });
    });
}

#[cfg(not(feature = "macos_14_0"))]
fn bench_screenshot_capture(_c: &mut Criterion) {
    // Screenshot manager requires macOS 14.0+
}

criterion_group!(
    benches,
    bench_shareable_content_get,
    bench_shareable_content_with_options,
    bench_content_filter_creation,
    bench_stream_configuration_creation,
    bench_screenshot_capture,
);

criterion_main!(benches);
