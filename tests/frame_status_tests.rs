//! Tests for `SCFrameStatus`

use screencapturekit::cm::SCFrameStatus;
use std::collections::HashSet;

#[test]
fn test_frame_status_from_raw() {
    assert_eq!(SCFrameStatus::from_raw(0), Some(SCFrameStatus::Complete));
    assert_eq!(SCFrameStatus::from_raw(1), Some(SCFrameStatus::Idle));
    assert_eq!(SCFrameStatus::from_raw(2), Some(SCFrameStatus::Blank));
    assert_eq!(SCFrameStatus::from_raw(3), Some(SCFrameStatus::Suspended));
    assert_eq!(SCFrameStatus::from_raw(4), Some(SCFrameStatus::Started));
    assert_eq!(SCFrameStatus::from_raw(5), Some(SCFrameStatus::Stopped));
    assert_eq!(SCFrameStatus::from_raw(6), None);
    assert_eq!(SCFrameStatus::from_raw(-1), None);
    assert_eq!(SCFrameStatus::from_raw(999), None);
}

#[test]
fn test_frame_status_has_content() {
    assert!(SCFrameStatus::Complete.has_content());
    assert!(!SCFrameStatus::Idle.has_content());
    assert!(!SCFrameStatus::Blank.has_content());
    assert!(!SCFrameStatus::Suspended.has_content());
    assert!(SCFrameStatus::Started.has_content());
    assert!(!SCFrameStatus::Stopped.has_content());
}

#[test]
fn test_frame_status_is_complete() {
    assert!(SCFrameStatus::Complete.is_complete());
    assert!(!SCFrameStatus::Idle.is_complete());
    assert!(!SCFrameStatus::Blank.is_complete());
    assert!(!SCFrameStatus::Suspended.is_complete());
    assert!(!SCFrameStatus::Started.is_complete());
    assert!(!SCFrameStatus::Stopped.is_complete());
}

#[test]
fn test_frame_status_display() {
    assert_eq!(format!("{}", SCFrameStatus::Complete), "Complete");
    assert_eq!(format!("{}", SCFrameStatus::Idle), "Idle");
    assert_eq!(format!("{}", SCFrameStatus::Blank), "Blank");
    assert_eq!(format!("{}", SCFrameStatus::Suspended), "Suspended");
    assert_eq!(format!("{}", SCFrameStatus::Started), "Started");
    assert_eq!(format!("{}", SCFrameStatus::Stopped), "Stopped");
}

#[test]
fn test_frame_status_debug() {
    let status = SCFrameStatus::Complete;
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Complete"));
}

#[test]
fn test_frame_status_equality() {
    assert_eq!(SCFrameStatus::Complete, SCFrameStatus::Complete);
    assert_ne!(SCFrameStatus::Complete, SCFrameStatus::Idle);
    assert_eq!(SCFrameStatus::Blank, SCFrameStatus::Blank);
}

#[test]
fn test_frame_status_hash() {
    let mut set = HashSet::new();
    set.insert(SCFrameStatus::Complete);
    set.insert(SCFrameStatus::Idle);
    set.insert(SCFrameStatus::Complete); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&SCFrameStatus::Complete));
    assert!(set.contains(&SCFrameStatus::Idle));
    assert!(!set.contains(&SCFrameStatus::Blank));
}

#[test]
fn test_frame_status_copy() {
    let status1 = SCFrameStatus::Complete;
    let status2 = status1; // Copy
    assert_eq!(status1, status2);

    // Both should still be usable
    assert!(status1.has_content());
    assert!(status2.has_content());
}

#[test]
fn test_frame_status_clone() {
    let status1 = SCFrameStatus::Idle;
    let status2 = status1;
    assert_eq!(status1, status2);
}

#[test]
fn test_frame_status_const() {
    const STATUS: SCFrameStatus = SCFrameStatus::Complete;
    assert_eq!(STATUS, SCFrameStatus::Complete);

    // Verify const methods work
    assert!(STATUS.has_content());
    assert!(STATUS.is_complete());
}

#[test]
fn test_frame_status_match() {
    let status = SCFrameStatus::Complete;

    let result = match status {
        SCFrameStatus::Complete => "complete",
        SCFrameStatus::Idle => "idle",
        SCFrameStatus::Blank => "blank",
        SCFrameStatus::Suspended => "suspended",
        SCFrameStatus::Started => "started",
        SCFrameStatus::Stopped => "stopped",
    };

    assert_eq!(result, "complete");
}

#[test]
fn test_frame_status_all_variants() {
    let all_statuses = vec![
        SCFrameStatus::Complete,
        SCFrameStatus::Idle,
        SCFrameStatus::Blank,
        SCFrameStatus::Suspended,
        SCFrameStatus::Started,
        SCFrameStatus::Stopped,
    ];

    // Ensure all can be created and are distinct
    let mut set = HashSet::new();
    for status in &all_statuses {
        set.insert(*status);
    }
    assert_eq!(set.len(), 6);
}

#[test]
fn test_frame_status_logic() {
    // Test filtering logic commonly used in applications
    let statuses = [
        SCFrameStatus::Complete,
        SCFrameStatus::Idle,
        SCFrameStatus::Blank,
        SCFrameStatus::Started,
    ];

    let processable: Vec<_> = statuses.iter().filter(|s| s.has_content()).collect();

    assert_eq!(processable.len(), 2); // Complete and Started
    assert!(processable.contains(&&SCFrameStatus::Complete));
    assert!(processable.contains(&&SCFrameStatus::Started));
}

#[test]
fn test_frame_status_from_raw_roundtrip() {
    let statuses = vec![
        SCFrameStatus::Complete,
        SCFrameStatus::Idle,
        SCFrameStatus::Blank,
        SCFrameStatus::Suspended,
        SCFrameStatus::Started,
        SCFrameStatus::Stopped,
    ];

    for status in statuses {
        let raw = status as i32;
        let recovered = SCFrameStatus::from_raw(raw);
        assert_eq!(recovered, Some(status));
    }
}

#[test]
fn test_frame_status_ordering() {
    // Verify the raw values are as expected
    assert_eq!(SCFrameStatus::Complete as i32, 0);
    assert_eq!(SCFrameStatus::Idle as i32, 1);
    assert_eq!(SCFrameStatus::Blank as i32, 2);
    assert_eq!(SCFrameStatus::Suspended as i32, 3);
    assert_eq!(SCFrameStatus::Started as i32, 4);
    assert_eq!(SCFrameStatus::Stopped as i32, 5);
}

#[test]
fn test_frame_status_option_usage() {
    let status: Option<SCFrameStatus> = Some(SCFrameStatus::Complete);

    assert!(status.is_some());
    if let Some(s) = status {
        assert_eq!(s, SCFrameStatus::Complete);
    }

    let no_status: Option<SCFrameStatus> = None;
    assert!(no_status.is_none());
}

#[test]
fn test_frame_status_result_usage() {
    fn check_status(status: SCFrameStatus) -> Result<(), &'static str> {
        if status.has_content() {
            Ok(())
        } else {
            Err("No content")
        }
    }

    assert!(check_status(SCFrameStatus::Complete).is_ok());
    assert!(check_status(SCFrameStatus::Started).is_ok());
    assert!(check_status(SCFrameStatus::Idle).is_err());
    assert!(check_status(SCFrameStatus::Blank).is_err());
}

// MARK: - FrameInfo completeness

/// `FrameInfo` is documented as a faithful, complete snapshot of the
/// `SCStreamFrameInfo` attachment dictionary. `dirtyRects` used to be the one
/// key the batched reader silently dropped, forcing callers back onto the
/// single-key accessor and a second attachment fetch per frame.
#[test]
fn test_frame_info_covers_every_attachment_key() {
    use screencapturekit::cm::FrameInfo;

    let info = FrameInfo::default();
    assert!(info.frame_status.is_none());
    assert!(info.display_time.is_none());
    assert!(info.scale_factor.is_none());
    assert!(info.content_scale.is_none());
    assert!(info.content_rect.is_none());
    assert!(info.bounding_rect.is_none());
    assert!(info.screen_rect.is_none());
    assert!(info.presenter_overlay_content_rect.is_none());
    assert!(
        info.dirty_rects.is_none(),
        "dirty_rects must be part of the batched snapshot"
    );
}

/// `SCStreamFrameInfoStatus` arrives as an `NSNumber`, not as a bridged
/// `SCFrameStatus`. Every raw value the enum knows must survive the
/// integer round-trip the Swift decoder performs.
#[test]
fn test_frame_status_round_trips_through_raw_values() {
    let all = [
        SCFrameStatus::Complete,
        SCFrameStatus::Idle,
        SCFrameStatus::Blank,
        SCFrameStatus::Suspended,
        SCFrameStatus::Started,
        SCFrameStatus::Stopped,
    ];
    for status in all {
        let raw = status as i32;
        assert_eq!(
            SCFrameStatus::from_raw(raw),
            Some(status),
            "status {status:?} must survive the NSNumber round-trip"
        );
    }
}

#[test]
fn test_live_frame_status_attachment_decodes() {
    use screencapturekit::cm::CMSampleBufferSCExt;
    use screencapturekit::prelude::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    let Ok(content) = SCShareableContent::get() else {
        eprintln!("skip: screen-recording permission unavailable");
        return;
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
    let config = SCStreamConfiguration::new()
        .with_width(160)
        .with_height(120);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&observed);

    let mut stream = SCStream::new(&filter, &config);
    stream
        .add_output_handler(
            move |sample: CMSampleBuffer, _| {
                captured
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(sample.frame_status());
            },
            SCStreamOutputType::Screen,
        )
        .expect("failed to register screen output");

    if let Err(error) = stream.start_capture() {
        eprintln!("skip: stream failed to start: {error}");
        return;
    }
    std::thread::sleep(Duration::from_millis(800));
    let _ = stream.stop_capture();

    let statuses = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(!statuses.is_empty(), "capture produced no frames");
    assert!(
        statuses.iter().all(Option::is_some),
        "live SCStreamFrameInfoStatus values did not decode: {statuses:?}"
    );
}
