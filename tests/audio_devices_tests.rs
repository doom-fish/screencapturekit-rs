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
