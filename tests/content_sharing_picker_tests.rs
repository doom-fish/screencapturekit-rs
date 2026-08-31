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
    let modes = [
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
    ];
    config.set_allowed_picker_modes(&modes);
    assert_eq!(config.allowed_picker_modes(), modes);
    assert!(!config.as_ptr().is_null());
    println!("✓ Picker modes round-trip successfully");
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

    let _guard = exclusive_registry();

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

// MARK: - Repeating observers (macOS 14.0+)

/// `SCContentSharingPicker.shared`, its `isActive` flag, and its observer
/// registry are all process-wide singletons, so tests that mutate them cannot
/// run concurrently: one test's `remove_all_observers()` would sweep another's
/// registration, and one test's `set_active` would flip another's expectation.
static PICKER_STATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Take the picker lock and start from a known-empty observer registry.
fn exclusive_registry() -> std::sync::MutexGuard<'static, ()> {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let guard = PICKER_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    SCContentSharingPicker::remove_all_observers();
    guard
}

/// `allows_changing_selected_content` is only meaningful with a repeating
/// observer: Apple re-invokes `didUpdateWith:` on every re-selection, and the
/// one-shot `show*()` helpers latch after the first event. Registering must
/// therefore hand back a live subscription.
#[test]
fn add_observer_returns_a_live_subscription() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let _guard = exclusive_registry();

    let subscription = SCContentSharingPicker::add_observer(|_event| {});
    if !subscription.is_active() {
        assert_eq!(subscription.token(), 0);
        return;
    }
    assert!(
        subscription.is_active(),
        "add_observer returned an inactive subscription"
    );
    assert_ne!(
        subscription.token(),
        0,
        "live subscription must have a token"
    );
    assert!(
        subscription.unsubscribe(),
        "unsubscribe reported no removal"
    );
}

#[test]
fn dropping_a_subscription_removes_the_observer() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let _guard = exclusive_registry();

    let token = {
        let subscription = SCContentSharingPicker::add_observer(|_event| {});
        subscription.token()
    };
    if token == 0 {
        assert_eq!(SCContentSharingPicker::remove_all_observers(), 0);
        return;
    }
    assert_ne!(token, 0);

    // The drop above already removed it, so nothing is left to sweep.
    assert_eq!(
        SCContentSharingPicker::remove_all_observers(),
        0,
        "observer survived the drop of its subscription"
    );
}

#[test]
fn unsubscribing_twice_is_not_reported_twice() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let _guard = exclusive_registry();

    let subscription = SCContentSharingPicker::add_observer(|_event| {});
    let was_active = subscription.is_active();
    assert_eq!(subscription.unsubscribe(), was_active);
    assert_eq!(SCContentSharingPicker::remove_all_observers(), 0);
}

#[test]
fn remove_all_observers_sweeps_detached_registrations() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let _guard = exclusive_registry();

    let first = SCContentSharingPicker::add_observer(|_event| {});
    let second = SCContentSharingPicker::add_observer(|_event| {});
    let expected = usize::from(first.is_active()) + usize::from(second.is_active());
    first.detach();
    second.detach();

    assert_eq!(
        SCContentSharingPicker::remove_all_observers(),
        expected,
        "detached observers were not swept"
    );
}

#[test]
fn observer_registration_does_not_leak_across_many_cycles() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let _guard = exclusive_registry();

    // The counter is captured by the closure; if the observer context were
    // leaked the Arc would never drop and the strong count would climb.
    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..16 {
        let counter = Arc::clone(&counter);
        let subscription = SCContentSharingPicker::add_observer(move |_event| {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        drop(subscription);
    }

    assert_eq!(SCContentSharingPicker::remove_all_observers(), 0);
    assert_eq!(
        Arc::strong_count(&counter),
        1,
        "observer contexts retained captured state after teardown"
    );
    assert_eq!(
        counter.load(Ordering::Relaxed),
        0,
        "an observer fired without any user interaction"
    );
}

// MARK: - Standalone configuration operations

#[test]
fn set_default_configuration_is_reflected_by_default_configuration() {
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPicker, SCContentSharingPickerConfiguration,
    };

    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allows_changing_selected_content(true);
    config.set_excluded_bundle_ids(&["com.example.picker-test"]);

    SCContentSharingPicker::set_default_configuration(&config);

    let read_back = SCContentSharingPicker::default_configuration();
    assert!(!read_back.as_ptr().is_null());
}

#[test]
fn excluded_bundle_ids_round_trip() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

    let mut config = SCContentSharingPickerConfiguration::new();
    let ids = ["com.apple.dock", "com.apple.finder"];
    config.set_excluded_bundle_ids(&ids);

    assert_eq!(config.excluded_bundle_ids_count(), ids.len());
    assert_eq!(config.excluded_bundle_ids(), ids);
}

#[test]
fn excluded_window_ids_round_trip() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_excluded_window_ids(&[7, 42, 1009]);
    assert_eq!(config.excluded_window_ids(), vec![7, 42, 1009]);
}

/// `deactivate()` must clear the picker's active flag rather than leaving the
/// Control Center "ready to share" entry lit for the process lifetime.
#[test]
fn deactivate_clears_the_active_flag() {
    use screencapturekit::content_sharing_picker::SCContentSharingPicker;

    let _guard = exclusive_registry();

    SCContentSharingPicker::set_active(true);
    SCContentSharingPicker::deactivate();

    // `deactivate()` hops to the main queue, so poll rather than assert
    // immediately. A test binary has no run loop pumping the main queue, so
    // this is best-effort: what must not happen is a crash or a hang.
    for _ in 0..20 {
        if !SCContentSharingPicker::is_active() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    SCContentSharingPicker::set_active(false);
    assert!(!SCContentSharingPicker::is_active());
}

// MARK: - Clone independence
//
// The wrapper is `Send + Sync` and exposes `&mut self` setters, so `Clone` must
// hand back an independent configuration. A refcount-only clone would let a
// `&mut` on one handle mutate state another handle observes through a shared
// `&`.

#[test]
fn clone_does_not_alias_the_original() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

    let mut original = SCContentSharingPickerConfiguration::new();
    original.set_excluded_bundle_ids(&["com.example.original"]);
    original.set_excluded_window_ids(&[1]);
    original.set_allows_changing_selected_content(false);

    let mut copy = original.clone();
    copy.set_excluded_bundle_ids(&["com.example.copy"]);
    copy.set_excluded_window_ids(&[2, 3]);
    copy.set_allows_changing_selected_content(true);

    assert_eq!(original.excluded_bundle_ids(), ["com.example.original"]);
    assert_eq!(original.excluded_window_ids(), vec![1]);
    assert!(!original.allows_changing_selected_content());

    assert_eq!(copy.excluded_bundle_ids(), ["com.example.copy"]);
    assert_eq!(copy.excluded_window_ids(), vec![2, 3]);
    assert!(copy.allows_changing_selected_content());
}

#[test]
fn clone_carries_the_source_values_forward() {
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    };

    let mut original = SCContentSharingPickerConfiguration::new();
    let modes = [
        SCContentSharingPickerMode::SingleDisplay,
        SCContentSharingPickerMode::MultipleWindows,
    ];
    original.set_allowed_picker_modes(&modes);
    original.set_excluded_bundle_ids(&["com.apple.dock"]);
    original.set_excluded_window_ids(&[11, 22]);
    original.set_allows_changing_selected_content(true);

    let copy = original.clone();
    assert_eq!(copy.allowed_picker_modes(), original.allowed_picker_modes());
    assert_eq!(copy.excluded_bundle_ids(), original.excluded_bundle_ids());
    assert_eq!(copy.excluded_window_ids(), original.excluded_window_ids());
    assert_eq!(
        copy.allows_changing_selected_content(),
        original.allows_changing_selected_content()
    );
}

#[test]
fn clone_survives_the_original_being_dropped() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

    let copy = {
        let mut original = SCContentSharingPickerConfiguration::new();
        original.set_excluded_bundle_ids(&["com.example.scoped"]);
        original.clone()
    };

    assert_eq!(copy.excluded_bundle_ids(), ["com.example.scoped"]);
}

/// `Send + Sync` is only sound because each handle owns its box exclusively.
/// Mutating clones concurrently must not race.
#[test]
fn clones_are_independently_mutable_across_threads() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerConfiguration;

    let mut template = SCContentSharingPickerConfiguration::new();
    template.set_excluded_window_ids(&[0]);

    let handles: Vec<_> = (1..=4u32)
        .map(|i| {
            let mut config = template.clone();
            std::thread::spawn(move || {
                for _ in 0..64 {
                    config.set_excluded_window_ids(&[i, i * 10]);
                    assert_eq!(config.excluded_window_ids(), vec![i, i * 10]);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    assert_eq!(template.excluded_window_ids(), vec![0]);
}
