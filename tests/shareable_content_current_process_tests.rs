//! `SCShareableContent::current_process` availability behaviour (macOS 14.4+).

#![cfg(feature = "macos_14_4")]

use screencapturekit::error::SCError;
use screencapturekit::shareable_content::SCShareableContent;

/// The current-process query must never silently widen into the system-wide
/// content query: that needs screen-recording consent and returns every window
/// on the machine. When the API is unavailable it has to say so.
#[test]
fn current_process_reports_feature_not_available_when_unsupported() {
    if SCShareableContent::current_process_is_available() {
        return;
    }

    match SCShareableContent::current_process() {
        Err(SCError::FeatureNotAvailable {
            required_version, ..
        }) => assert_eq!(required_version, "14.4"),
        Err(other) => panic!("expected FeatureNotAvailable, got {other:?}"),
        Ok(_) => panic!("current_process succeeded despite reporting unavailable"),
    }
}

#[test]
fn current_process_succeeds_when_available() {
    if !SCShareableContent::current_process_is_available() {
        return;
    }

    let content = SCShareableContent::current_process()
        .expect("current_process failed on a system that reports it as available");
    // The current process owns no shareable displays in a test harness, but the
    // call must still return a usable container.
    let _ = content.displays().len();
}
