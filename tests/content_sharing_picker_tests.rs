//! Content sharing picker tests
//!
//! Tests for `SCContentSharingPickerConfiguration` (macOS 14.0+).

#![cfg(feature = "macos_14_0")]

use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

#[test]
fn test_picker_configuration_new() {
    let config = SCContentSharingPickerConfiguration::new();
    assert!(!config.as_ptr().is_null());
    println!("✓ Picker configuration created");
}

#[test]
fn test_picker_configuration_default() {
    let config = SCContentSharingPickerConfiguration::default();
    assert!(!config.as_ptr().is_null());
    println!("✓ Picker default configuration created");
}

/// Regression test for Gap 2 of the SDK gap analysis: the bridge
/// must expose the system's `defaultConfiguration` on
/// `SCContentSharingPicker.shared` so callers can build on Apple's
/// baseline rather than starting from `SCContentSharingPickerConfiguration()`.
///
/// We can't assert anything about the *content* of the default config
/// without screen-recording permission and a valid picker session, but
/// we can verify the constructor returns a non-null pointer and that
/// the value participates in the standard retain/release lifecycle
/// (Drop must not crash; Clone must produce a distinct heap-owned copy).
#[test]
fn test_picker_configuration_default_from_system() {
    let config = SCContentSharingPickerConfiguration::default_from_system();
    assert!(
        !config.as_ptr().is_null(),
        "default_from_system() returned a null configuration pointer"
    );

    // The returned config must be independently retain/releasable.
    let cloned = config.clone();
    assert!(!cloned.as_ptr().is_null());
    drop(cloned);

    // And it must be safe to mutate (i.e. it isn't pointing at a shared
    // singleton that other callers depend on).
    let mut config = config;
    config.set_excluded_bundle_ids(&["com.apple.dock"]);
    drop(config);
}

#[test]
fn test_picker_configuration_clone() {
    let config1 = SCContentSharingPickerConfiguration::new();
    let config2 = config1.clone();

    assert!(!config1.as_ptr().is_null());
    assert!(!config2.as_ptr().is_null());

    println!("✓ Picker configuration clone works");
}

#[test]
fn test_picker_configuration_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SCContentSharingPickerConfiguration>();
    assert_sync::<SCContentSharingPickerConfiguration>();

    println!("✓ SCContentSharingPickerConfiguration is Send + Sync");
}

#[test]
fn test_picker_configuration_lifecycle() {
    // Test creating and dropping multiple configurations
    for i in 0..3 {
        let config = SCContentSharingPickerConfiguration::new();
        assert!(!config.as_ptr().is_null());
        println!("✓ Configuration {i} created");
        drop(config);
        println!("✓ Configuration {i} dropped");
    }
}

#[test]
fn test_picker_configuration_modes() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerMode;

    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(&[
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
    ]);
    assert!(!config.as_ptr().is_null());
    println!("✓ Picker modes set successfully");
}

#[test]
fn test_picker_api_availability() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    // Verify the API types are available on macOS 14.0+
    let _picker_type = std::any::type_name::<SCContentSharingPicker>();
    let _config_type = std::any::type_name::<SCContentSharingPickerConfiguration>();

    println!("✓ Content sharing picker API available on macOS 14.0+");
}

// MARK: - Async Picker Tests

#[test]
#[cfg(feature = "async")]
fn test_async_picker_types_exist() {
    use screencapturekit::async_api::{
        AsyncPickerFilterFuture, AsyncPickerFuture, AsyncSCContentSharingPicker,
    };

    // Verify the async types exist
    let _picker_type = std::any::type_name::<AsyncSCContentSharingPicker>();
    let _future_type = std::any::type_name::<AsyncPickerFuture>();
    let _filter_future_type = std::any::type_name::<AsyncPickerFilterFuture>();

    println!("✓ Async picker types available");
}

#[test]
#[cfg(feature = "async")]
fn test_async_picker_future_is_future() {
    use screencapturekit::async_api::AsyncPickerFuture;
    use std::future::Future;

    fn assert_future<T: Future>() {}
    assert_future::<AsyncPickerFuture>();

    println!("✓ AsyncPickerFuture implements Future");
}

#[test]
#[cfg(feature = "async")]
fn test_async_picker_filter_future_is_future() {
    use screencapturekit::async_api::AsyncPickerFilterFuture;
    use std::future::Future;

    fn assert_future<T: Future>() {}
    assert_future::<AsyncPickerFilterFuture>();

    println!("✓ AsyncPickerFilterFuture implements Future");
}

/// Regression test for SDK-headers gap analysis Pass 2: the bridge
/// must expose Apple's `SCContentSharingPicker.isActive` getter and
/// setter. Apple requires `picker.isActive = true` before its UI can
/// appear. The crate's `show*()` trampolines set it implicitly, but
/// callers may want to query the flag (to avoid double-presenting) or
/// explicitly deactivate the picker between sessions.
///
/// This test verifies the round-trip works without screen-recording
/// permission (the `isActive` flag is a process-local state that
/// doesn't depend on TCC).
#[test]
fn test_picker_is_active_get_set_roundtrip() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    // Capture the initial state so we can restore it (the picker is a
    // process-wide singleton; another test or example might depend on
    // its current state).
    let original = SCContentSharingPicker::is_active();

    SCContentSharingPicker::set_active(true);
    assert!(
        SCContentSharingPicker::is_active(),
        "is_active() returned false immediately after set_active(true)"
    );

    SCContentSharingPicker::set_active(false);
    assert!(
        !SCContentSharingPicker::is_active(),
        "is_active() returned true immediately after set_active(false)"
    );

    // Restore.
    SCContentSharingPicker::set_active(original);
}
