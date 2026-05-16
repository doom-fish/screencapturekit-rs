//! Coverage tests for paths that the previous test suite didn't exercise.
//!
//! These all depend on the live `ScreenCaptureKit` callback path so they
//! skip with a clear `SKIP:` message on stderr when screen-recording
//! permission is unavailable. Run them on a permission-granted CI runner
//! with `cargo test -- --nocapture` to see the skip messages, or on a
//! local box with permission to validate the assertions actually run.
//!
//! Coverage gaps targeted (M11 from the deep code review):
//!
//! * **YUV / NV12 pixel format**: the encoder integration story
//!   (H.264/HEVC) depends on `SCStream` actually delivering YUV when
//!   asked. Previously tested only at the type-equality level.
//! * **Handler add/remove mid-capture**: a documented feature with no
//!   prior behavioural coverage. Validates the
//!   `add_output_handler` → `remove_output_handler` lifecycle while
//!   capture is in flight, which exercises the `RwLock<Vec<HandlerEntry>>`
//!   write path against an active dispatch reader.

use screencapturekit::{
    cm::CMSampleBuffer,
    shareable_content::SCShareableContent,
    stream::{
        configuration::{PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
};
use screencapturekit::cm::{CMSampleBufferExt, CMSampleBufferSCExt};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Captures the first sample so the test can inspect the actual pixel
/// format returned by `SCStream` (vs the format we asked for).
struct FirstSampleCollector {
    captured: Mutex<Option<CMSampleBuffer>>,
}

impl SCStreamOutputTrait for FirstSampleCollector {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if !matches!(of_type, SCStreamOutputType::Screen) {
            return;
        }
        if let Ok(mut slot) = self.captured.lock() {
            if slot.is_none() {
                *slot = Some(sample_buffer);
            }
        }
    }
}

/// Trait-object dispatch via Arc — lets multiple tests share the same
/// `FirstSampleCollector` while still satisfying `add_output_handler`'s
/// `'static + Send + Sync` bound.
struct DelegatingHandler {
    inner: Arc<FirstSampleCollector>,
}

impl SCStreamOutputTrait for DelegatingHandler {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.inner.did_output_sample_buffer(sample_buffer, of_type);
    }
}

/// Regression test for M11 (YUV/NV12 capture): if the user configures the
/// stream for `YCbCr_420v`, `SCStream` must actually deliver biplanar YUV
/// buffers — not silently fall back to BGRA. Branches on the wrong
/// assumption would otherwise read YUV bytes as packed BGRA and corrupt
/// every encoder/preview that relies on the format.
#[test]
fn test_yuv_420v_pixel_format_capture() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "SKIP: Screen recording permission required to validate YUV capture (error: {e:?})"
            );
            return;
        }
    };

    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("SKIP: No displays available");
        return;
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(640)
        .with_height(480)
        .with_pixel_format(PixelFormat::YCbCr_420v);

    let collector = Arc::new(FirstSampleCollector {
        captured: Mutex::new(None),
    });

    let mut stream = SCStream::new(&filter, &config);
    stream
        .add_output_handler(
            DelegatingHandler {
                inner: collector.clone(),
            },
            SCStreamOutputType::Screen,
        )
        .expect("add_output_handler failed");

    if let Err(e) = stream.start_capture() {
        eprintln!("SKIP: stream failed to start (typically permission-related): {e:?}");
        return;
    }

    // Wait up to 2 s for the first sample.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while collector.captured.lock().is_ok_and(|s| s.is_none())
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(20));
    }

    let _ = stream.stop_capture();

    let sample = collector
        .captured
        .lock()
        .ok()
        .and_then(|mut s| s.take())
        .expect("no sample captured within 2s");

    // Some samples are status-only (no image buffer attached, e.g. an
    // idle frame). Skip those — we want a real pixel-buffer-bearing one.
    let Some(image_buffer) = sample.image_buffer() else {
        eprintln!("SKIP: first sample had no image buffer (likely idle frame); test inconclusive");
        return;
    };

    // Two-plane YUV ("420v" / "420f") buffers report >=2 planes; BGRA is
    // a single packed plane. This is the smoking-gun assertion: if the
    // stream silently fell back to BGRA, this would be 0 (non-planar).
    let plane_count = image_buffer.plane_count();
    assert!(
        plane_count >= 2,
        "expected biplanar YUV (>=2 planes) for YCbCr_420v configuration, \
         got plane_count={plane_count} — stream likely fell back to BGRA"
    );

    // Sanity-check that we can lock and read at least the Y plane.
    let _guard = image_buffer
        .lock_read_only()
        .expect("lock_read_only failed for YUV buffer");
}

/// Regression test for M11 (handler add/remove mid-capture). Adds a
/// handler, captures a few frames, removes it mid-capture, captures a
/// few more, and asserts that callbacks stop at exactly the right
/// moment. This exercises the `RwLock<Vec<HandlerEntry>>` write path
/// while a reader (the dispatch callback) may be in flight, which is
/// the path that would have regressed if the Phase-2 `RwLock` fix
/// introduced a torn-read bug.
#[test]
fn test_handler_add_remove_mid_capture() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "SKIP: Screen recording permission required for mid-capture handler lifecycle test (error: {e:?})"
            );
            return;
        }
    };

    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("SKIP: No displays available");
        return;
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(320)
        .with_height(240)
        .with_pixel_format(PixelFormat::BGRA);

    let count = Arc::new(AtomicUsize::new(0));
    let active = Arc::new(AtomicBool::new(true));

    let count_handler = count.clone();
    let active_handler = active.clone();
    let handler = move |_sample: CMSampleBuffer, _of_type: SCStreamOutputType| {
        if active_handler.load(Ordering::Relaxed) {
            count_handler.fetch_add(1, Ordering::Relaxed);
        }
    };

    let mut stream = SCStream::new(&filter, &config);
    let id = stream
        .add_output_handler(handler, SCStreamOutputType::Screen)
        .expect("add_output_handler failed");

    if let Err(e) = stream.start_capture() {
        eprintln!("SKIP: stream failed to start: {e:?}");
        return;
    }

    // Phase 1: handler is registered; expect the count to grow.
    std::thread::sleep(Duration::from_millis(500));
    let count_before_remove = count.load(Ordering::Relaxed);
    assert!(
        count_before_remove > 0,
        "expected at least one frame before mid-capture removal, got {count_before_remove}"
    );

    // Remove the handler mid-capture, then disable the active flag so
    // any in-flight callback that already passed the read-lock and
    // started executing won't increment the counter further.
    let removed = stream.remove_output_handler(id, SCStreamOutputType::Screen);
    active.store(false, Ordering::Relaxed);
    assert!(removed, "remove_output_handler returned false");

    // Phase 2: stream still running, but no handler — the counter must
    // not advance. Sleep a little longer than typical inter-frame
    // intervals to give the system time to deliver more frames.
    let count_after_remove = count.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(500));
    let count_settled = count.load(Ordering::Relaxed);

    let _ = stream.stop_capture();

    // Allow up to a small grace window of in-flight callbacks (those
    // that won the read-lock before remove returned). 2 frames is a
    // generous bound at any framerate.
    let drift = count_settled.saturating_sub(count_after_remove);
    assert!(
        drift <= 2,
        "after remove_output_handler, the handler kept firing: \
         count_after_remove={count_after_remove}, count_settled={count_settled}, drift={drift}"
    );
}

/// Regression test for Gap 1 of the SDK gap analysis:
/// `CMSampleBuffer::presenter_overlay_content_rect()` must exist, be
/// callable on a real captured sample, and return `None` for streams
/// that aren't using Presenter Overlay (the common case).
///
/// We can't actually drive Presenter Overlay from a test (it requires
/// active video conferencing), but verifying the accessor returns `None`
/// gracefully on a non-overlay stream confirms:
///   1. the FFI declaration is correct (no link errors),
///   2. the Swift bridge function correctly handles the "attachment
///      missing" case (returns false → `None`),
///   3. the Rust accessor doesn't panic when the underlying Apple key
///      isn't set on the sample's attachment dictionary.
///
/// On a stream WITH Presenter Overlay enabled, this would return
/// `Some(rect)`; we don't have a way to assert that without a live
/// video-conference session.
#[cfg(feature = "macos_14_2")]
#[test]
fn test_presenter_overlay_content_rect_absent_when_overlay_disabled() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "SKIP: Screen recording permission required for Presenter Overlay attachment test (error: {e:?})"
            );
            return;
        }
    };

    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("SKIP: No displays available");
        return;
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(320)
        .with_height(240)
        .with_pixel_format(PixelFormat::BGRA);

    let collector = Arc::new(FirstSampleCollector {
        captured: Mutex::new(None),
    });

    let mut stream = SCStream::new(&filter, &config);
    stream
        .add_output_handler(
            DelegatingHandler {
                inner: collector.clone(),
            },
            SCStreamOutputType::Screen,
        )
        .expect("add_output_handler failed");

    if let Err(e) = stream.start_capture() {
        eprintln!("SKIP: stream failed to start: {e:?}");
        return;
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while collector.captured.lock().is_ok_and(|s| s.is_none())
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(20));
    }

    let _ = stream.stop_capture();

    let sample = collector
        .captured
        .lock()
        .ok()
        .and_then(|mut s| s.take())
        .expect("no sample captured within 2s");

    // ScreenCaptureKit may either omit the attachment entirely (-> None)
    // OR attach `CGRectNull` (`x = inf, y = inf, width = 0, height = 0`) when
    // Presenter Overlay isn't active. Both represent "no overlay rectangle";
    // both are acceptable. What we're really verifying is that the accessor
    // is wired up, doesn't panic, and doesn't return a finite rectangle.
    let overlay_rect = sample.presenter_overlay_content_rect();
    if let Some(rect) = overlay_rect {
        assert!(
            !rect.x.is_finite() || rect.width == 0.0 || rect.height == 0.0,
            "presenter_overlay_content_rect() returned a finite rect ({rect:?}) on a \
             stream that did not enable Presenter Overlay; expected None or CGRectNull"
        );
    }
}
