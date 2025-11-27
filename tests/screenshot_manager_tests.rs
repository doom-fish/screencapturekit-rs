#![allow(clippy::pedantic, clippy::nursery)]
//! Screenshot manager tests (macOS 14.0+)

#![cfg(feature = "macos_14_0")]

use screencapturekit::screenshot_manager::SCScreenshotManager;

#[test]
fn test_screenshot_manager_type() {
    // Just verify the type exists and can be referenced
    let _manager = SCScreenshotManager;
}
