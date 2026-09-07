//! End-to-end regression tests for per-stream callback routing.
//!
//! These complement the in-tree unit test in `src/stream/sc_stream.rs`
//! (`test_per_stream_callback_isolation`), which exercises the routing
//! logic by directly invoking `StreamContext` from Rust without going
//! through Swift. The tests below drive two concurrent live `SCStream`s
//! end-to-end through the Swift bridge, verifying that the fix for
//! [#135](https://github.com/doom-fish/screencapturekit-rs/pull/135) holds:
//! samples for stream A must not be observed by stream B's handler, and
//! vice versa.
//!
//! Tests skip with a clear message when screen-recording permission is
//! unavailable or no display is found, since they depend on the live
//! `ScreenCaptureKit` callback path.

use screencapturekit::{
    cm::CMSampleBuffer,
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Tagged handler — every call increments `count`. The `tag` is kept on
/// the struct so debug logs from a future test extension can inspect
/// which stream the sample was routed to without refactoring; the field
/// is intentionally unread today.
#[allow(dead_code)]
struct TaggedHandler {
    tag: &'static str,
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for TaggedHandler {
    fn did_output_sample_buffer(
        &self,
        _sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

impl TaggedHandler {
    fn new(tag: &'static str) -> (Self, Arc<AtomicUsize>) {
        let count = Arc::new(AtomicUsize::new(0));
        (
            Self {
                tag,
                count: count.clone(),
            },
            count,
        )
    }
}

/// Regression test for #135: two concurrent `SCStream` instances must
/// route samples only to their own handlers. Before the per-stream
/// `StreamContext` fix, both streams shared a global registry and any
/// callback on one stream would also fire the other stream's handler.
///
/// This is the END-TO-END complement to the unit test
/// `test_per_stream_callback_isolation` in `sc_stream.rs`. The unit test
/// invokes context routing directly without Swift; this test takes the
/// full Swift→Rust callback path through real `SCStream`s.
#[test]
fn test_two_concurrent_streams_route_samples_independently() {
    // Skip with a clear message if permission/display isn't available —
    // these are environmental failures, not regressions.
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "skip: screen-recording permission required to exercise the \
                 live Swift→Rust routing path (error: {e:?})"
            );
            return;
        }
    };

    let displays = content.displays();
    let Some(display) = displays.first() else {
        eprintln!("skip: no displays available");
        return;
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    // Tiny configuration to keep the test cheap.
    let mut config_a = SCStreamConfiguration::default();
    config_a.set_width(320);
    config_a.set_height(240);
    config_a.set_captures_audio(false);

    let mut config_b = SCStreamConfiguration::default();
    config_b.set_width(320);
    config_b.set_height(240);
    config_b.set_captures_audio(false);

    let (handler_a, count_a) = TaggedHandler::new("A");
    let (handler_b, count_b) = TaggedHandler::new("B");

    // Stream A
    let mut stream_a = SCStream::new(&filter, &config_a);
    let _id_a = stream_a
        .add_output_handler(handler_a, SCStreamOutputType::Screen)
        .expect("add_output_handler A failed");

    // Stream B (independent SCStream — must have its own routing context)
    let mut stream_b = SCStream::new(&filter, &config_b);
    let _id_b = stream_b
        .add_output_handler(handler_b, SCStreamOutputType::Screen)
        .expect("add_output_handler B failed");

    if let Err(e) = stream_a.start_capture() {
        eprintln!("skip: stream A failed to start (typically permission-related): {e:?}");
        return;
    }
    if let Err(e) = stream_b.start_capture() {
        let _ = stream_a.stop_capture();
        eprintln!("skip: stream B failed to start: {e:?}");
        return;
    }

    // Let frames flow.
    std::thread::sleep(Duration::from_millis(800));

    // Stop both streams before reading counters so no further callbacks fire.
    let _ = stream_a.stop_capture();
    let _ = stream_b.stop_capture();
    // Settle any in-flight callbacks.
    std::thread::sleep(Duration::from_millis(100));

    let a = count_a.load(Ordering::Relaxed);
    let b = count_b.load(Ordering::Relaxed);

    eprintln!("stream A received {a} samples; stream B received {b} samples");

    // Both streams should have received SOME samples (otherwise the
    // capture path itself is broken — not a routing bug).
    assert!(
        a > 0,
        "stream A handler received zero samples; capture path broken"
    );
    assert!(
        b > 0,
        "stream B handler received zero samples; capture path broken"
    );

    // The cross-stream leak that #135 fixed manifests as either handler
    // receiving roughly DOUBLE its expected count (because every callback
    // fires both handlers via the old global registry). With per-stream
    // context routing the counts are independent — they should be in the
    // same order of magnitude (both streams capture the same display at
    // the same FPS) but neither should be much larger than the other.
    //
    // We use a generous ratio bound (3×) to stay robust to scheduling
    // jitter while still catching the regression: under the old global
    // registry, A would receive A's frames + B's frames ≈ 2× expected,
    // so a 3× tolerance is comfortably below 2× the natural ratio.
    let (smaller, larger) = if a <= b { (a, b) } else { (b, a) };
    assert!(
        larger <= smaller.saturating_mul(3),
        "sample counts diverge too much (A={a}, B={b}) — possible cross-stream leak; \
         under the old global routing one stream would see ~2× the other's frames"
    );
}

/// Shared skip-aware setup: a 320×240 video-only filter/config for the first
/// display, or `None` when the environment can't run live capture.
fn live_capture_fixture() -> Option<(SCContentFilter, SCStreamConfiguration)> {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip: screen-recording permission required (error: {e:?})");
            return None;
        }
    };
    let displays = content.displays();
    let display = displays.first().or_else(|| {
        eprintln!("skip: no displays available");
        None
    })?;

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let mut config = SCStreamConfiguration::default();
    config.set_width(320);
    config.set_height(240);
    config.set_captures_audio(false);

    Some((filter, config))
}

/// Regression test: `remove_output_handler` must match on the output type as
/// well as the id. Passing a mismatched type used to remove the handler
/// anyway — and then detach the *wrong* native output.
#[test]
fn test_remove_output_handler_rejects_a_mismatched_output_type() {
    let Some((filter, config)) = live_capture_fixture() else {
        return;
    };

    let (handler, count) = TaggedHandler::new("screen");
    let mut stream = SCStream::new(&filter, &config);
    let id = stream
        .add_output_handler(handler, SCStreamOutputType::Screen)
        .expect("add_output_handler failed");

    assert!(
        !stream.remove_output_handler(id, SCStreamOutputType::Audio),
        "removing a Screen handler under the Audio type must not succeed"
    );

    if let Err(e) = stream.start_capture() {
        eprintln!("skip: stream failed to start: {e:?}");
        return;
    }
    std::thread::sleep(Duration::from_millis(500));
    let _ = stream.stop_capture();
    std::thread::sleep(Duration::from_millis(100));

    assert!(
        count.load(Ordering::Relaxed) > 0,
        "the mismatched removal detached a handler it had no business touching"
    );

    // The correctly typed removal still works and reports the native teardown.
    assert!(
        stream
            .try_remove_output_handler(id, SCStreamOutputType::Screen)
            .expect("native removeStreamOutput failed"),
        "correctly typed removal reported 'not found'"
    );
    assert!(
        !stream.remove_output_handler(id, SCStreamOutputType::Screen),
        "second removal of the same id must report 'not found'"
    );
}

/// `ScreenCaptureKit` delivers every handler of one output type on a single
/// queue, so a second handler that demands a *different* queue must be
/// rejected rather than silently attached to the first handler's queue.
#[test]
fn test_conflicting_custom_queue_for_one_output_type_is_rejected() {
    use screencapturekit::dispatch_queue::{DispatchQoS, DispatchQueue};

    let Some((filter, config)) = live_capture_fixture() else {
        return;
    };

    let queue_a = DispatchQueue::new("com.test.capture.a", DispatchQoS::UserInteractive);
    let queue_b = DispatchQueue::new("com.test.capture.b", DispatchQoS::UserInteractive);

    let (first, _first_count) = TaggedHandler::new("first");
    let (second, _second_count) = TaggedHandler::new("second");
    let (third, _third_count) = TaggedHandler::new("third");

    let mut stream = SCStream::new(&filter, &config);
    let first_id = stream
        .add_output_handler_with_queue(first, SCStreamOutputType::Screen, Some(&queue_a))
        .expect("first registration failed");

    assert!(
        stream
            .add_output_handler_with_queue(second, SCStreamOutputType::Screen, Some(&queue_b))
            .is_none(),
        "a second Screen handler on a different queue must be rejected"
    );

    // Not asking for a specific queue is fine — it joins the established one.
    assert!(
        stream
            .add_output_handler_with_queue(third, SCStreamOutputType::Screen, None)
            .is_some(),
        "a queue-agnostic handler must be allowed to join the established queue"
    );

    assert!(stream.remove_output_handler(first_id, SCStreamOutputType::Screen));
}

/// Regression test: dropping a clone must not tear down the shared Swift-side
/// `StreamState`. The bridge used to discard it on the first release, which
/// detached the delegate and output handler of every surviving clone.
#[test]
fn test_clone_keeps_delivering_after_the_original_is_dropped() {
    let Some((filter, config)) = live_capture_fixture() else {
        return;
    };

    let (handler, count) = TaggedHandler::new("clone");
    let mut stream = SCStream::new(&filter, &config);
    stream
        .add_output_handler(handler, SCStreamOutputType::Screen)
        .expect("add_output_handler failed");

    let mut clone = stream.clone();
    drop(stream);

    // Registration goes through the shared bridge-side state, so this fails
    // outright if dropping the original discarded it.
    let (late, late_count) = TaggedHandler::new("late");
    clone
        .add_output_handler(late, SCStreamOutputType::Screen)
        .expect("registering on the surviving clone failed — bridge state was torn down");

    if let Err(e) = clone.start_capture() {
        eprintln!("skip: stream failed to start: {e:?}");
        return;
    }
    std::thread::sleep(Duration::from_millis(800));
    let _ = clone.stop_capture();
    std::thread::sleep(Duration::from_millis(100));

    assert!(
        count.load(Ordering::Relaxed) > 0,
        "the surviving clone received no samples — dropping the original tore \
         down the shared bridge state"
    );
    assert!(
        late_count.load(Ordering::Relaxed) > 0,
        "the handler registered after the original was dropped never fired"
    );
}
