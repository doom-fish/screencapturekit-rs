//! Tests for audio input device enumeration

use screencapturekit::audio_devices::AudioInputDevice;

#[test]
fn test_list_audio_devices() {
    // Should not panic
    let devices = AudioInputDevice::list();
    // On most Macs, there should be at least one built-in microphone
    println!("Found {} audio input devices", devices.len());
    for device in &devices {
        println!(
            "  {} - {} (default: {})",
            device.id, device.name, device.is_default
        );
    }
}

#[test]
fn test_default_device() {
    // Should not panic
    if let Some(device) = AudioInputDevice::default_device() {
        println!("Default device: {} - {}", device.id, device.name);
        assert!(device.is_default);
    } else {
        println!("No default audio input device");
    }
}

/// The device list and the default device must come from the same discovery
/// pass. Reading the count from one query and the fields from later queries
/// let a hot-plug produce a default device that wasn't in the list at all.
#[test]
fn default_device_is_present_in_the_listed_devices() {
    let (devices, default_index) = AudioInputDevice::list_with_default();

    if let Some(index) = default_index {
        let default = devices
            .get(index)
            .expect("list_with_default returned an index outside the device list");
        assert!(
            default.is_default,
            "device at the reported default index does not carry is_default"
        );
    }

    assert!(
        devices.iter().filter(|d| d.is_default).count() <= 1,
        "more than one device claims to be the system default"
    );
}

#[test]
fn list_entries_have_non_empty_id_and_name() {
    for device in AudioInputDevice::list() {
        assert!(!device.id.is_empty(), "device id must not be empty");
        assert!(!device.name.is_empty(), "device name must not be empty");
    }
}

/// Repeated snapshots must not leak or corrupt the Swift-owned list.
#[test]
fn repeated_snapshots_are_stable() {
    let first = AudioInputDevice::list();
    for _ in 0..8 {
        let again = AudioInputDevice::list();
        assert_eq!(
            first.len(),
            again.len(),
            "device count changed across back-to-back snapshots"
        );
    }
}
