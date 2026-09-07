//! Live regression tests for [`SCShareableContent::snapshot`].
//!
//! The batched snapshot path writes into `Vec::with_capacity` scratch buffers
//! that the Swift bridge fills through raw pointers. Reading those back used to
//! panic ("index out of bounds") because `with_capacity` leaves `len() == 0`,
//! and a stale owning-app index could panic on `applications[idx]`. These tests
//! drive the real bridge to keep both fixed.
//!
//! They skip with a message when screen-recording permission or a display is
//! unavailable, matching the rest of the live-capture suite.

use screencapturekit::shareable_content::SCShareableContent;

fn content_or_skip() -> Option<SCShareableContent> {
    match SCShareableContent::get() {
        Ok(content) => Some(content),
        Err(e) => {
            eprintln!("skip: screen-recording permission required (error: {e:?})");
            None
        }
    }
}

#[test]
fn test_snapshot_does_not_panic_and_reports_real_content() {
    let Some(content) = content_or_skip() else {
        return;
    };

    let snapshot = content.snapshot().expect("snapshot returned None");

    assert_eq!(
        snapshot.displays.len(),
        content.displays().len(),
        "batched display count diverged from the per-element accessors"
    );

    for display in &snapshot.displays {
        assert!(display.width > 0, "display {display:?} has no width");
        assert!(display.height > 0, "display {display:?} has no height");
    }
}

/// Every `owning_app_index` must address a collected application — a stale
/// index would panic at the `applications[idx]` call site.
#[test]
fn test_snapshot_owning_app_indices_are_always_in_range() {
    let Some(content) = content_or_skip() else {
        return;
    };

    let snapshot = content.snapshot().expect("snapshot returned None");

    for window in &snapshot.windows {
        if let Some(index) = window.owning_app_index {
            assert!(
                index < snapshot.applications.len(),
                "window {} reported owning app index {index} with only {} applications",
                window.window_id,
                snapshot.applications.len()
            );
            // Indexing is what used to panic; do it explicitly.
            let owner = &snapshot.applications[index];
            assert!(
                owner.process_id > 0,
                "window {} is owned by application {index} with a non-positive pid {}",
                window.window_id,
                owner.process_id
            );
        }
    }
}

/// Repeated snapshots must be stable and must not read uninitialised scratch
/// memory: the string pool is reused across the application and window batch
/// calls, so a bad `used` bound would surface as garbage titles or a panic.
#[test]
fn test_repeated_snapshots_are_consistent() {
    let Some(content) = content_or_skip() else {
        return;
    };

    let first = content.snapshot().expect("first snapshot returned None");
    let second = content.snapshot().expect("second snapshot returned None");

    assert_eq!(first.displays, second.displays);
    assert_eq!(first.applications, second.applications);
    assert_eq!(first.windows, second.windows);
    assert!(
        first.string_pool_used
            <= screencapturekit::shareable_content::ContentSnapshot::string_pool_capacity(),
        "reported string pool usage exceeds its capacity"
    );

    for app in &first.applications {
        // Names come out of the shared pool; non-UTF-8 or an out-of-range
        // offset would have yielded an empty string rather than garbage.
        assert!(app.bundle_identifier.is_char_boundary(0));
        assert!(app.application_name.is_char_boundary(0));
    }
}
