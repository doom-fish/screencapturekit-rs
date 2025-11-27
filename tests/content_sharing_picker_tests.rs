//! Content sharing picker tests
//!
//! Tests for SCContentSharingPicker and SCContentSharingPickerConfiguration (macOS 14.0+).

#![cfg(feature = "macos_14_0")]

use screencapturekit::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration,
};

#[test]
fn test_picker_configuration_new() {
    let config = SCContentSharingPickerConfiguration::new();
    println!("✓ Picker configuration created");
    drop(config);
}

#[test]
fn test_picker_configuration_clone() {
    let config1 = SCContentSharingPickerConfiguration::new();
    let config2 = config1.clone();
    
    drop(config1);
    drop(config2);
    
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
fn test_picker_shared_instance() {
    let picker = SCContentSharingPicker::new();
    println!("✓ Picker shared instance obtained");
    drop(picker);
}

#[test]
fn test_picker_active_state() {
    let picker = SCContentSharingPicker::new();
    let is_active = picker.is_active();
    
    println!("✓ Picker active state: {}", is_active);
}

#[test]
fn test_picker_default_configuration() {
    let config = SCContentSharingPickerConfiguration::default_configuration();
    println!("✓ Default picker configuration obtained");
    drop(config);
}

#[test]
fn test_picker_max_stream_count() {
    let picker = SCContentSharingPicker::new();
    let max_count = picker.max_stream_count();
    
    assert!(max_count >= 0, "Max stream count should be non-negative");
    println!("✓ Max stream count: {}", max_count);
}

#[test]
fn test_picker_set_max_stream_count() {
    let mut picker = SCContentSharingPicker::new();
    
    // Try setting different values
    for count in [1, 2, 5] {
        picker.set_max_stream_count(count);
        let retrieved = picker.max_stream_count();
        
        // macOS might clamp the value
        assert!(retrieved > 0, "Max stream count should be positive");
        println!("✓ Set max stream count to {}, got {}", count, retrieved);
    }
}

#[test]
fn test_picker_multiple_instances() {
    let picker1 = SCContentSharingPicker::new();
    let picker2 = SCContentSharingPicker::new();
    
    // Both should reference the same shared instance
    let active1 = picker1.is_active();
    let active2 = picker2.is_active();
    
    assert_eq!(active1, active2, "Shared instances should have same state");
    println!("✓ Multiple shared instances work correctly");
}

#[test]
fn test_picker_configuration_lifecycle() {
    // Test creating and dropping multiple configurations
    for i in 0..3 {
        let config = SCContentSharingPickerConfiguration::new();
        println!("✓ Configuration {} created", i);
        drop(config);
        println!("✓ Configuration {} dropped", i);
    }
}

#[test]
fn test_picker_zero_max_stream_count() {
    let mut picker = SCContentSharingPicker::new();
    
    picker.set_max_stream_count(0);
    let retrieved = picker.max_stream_count();
    
    // System might clamp to minimum of 1
    println!("✓ Zero max stream count handled: {}", retrieved);
}

#[test]
fn test_picker_large_max_stream_count() {
    let mut picker = SCContentSharingPicker::new();
    
    picker.set_max_stream_count(100);
    let retrieved = picker.max_stream_count();
    
    // System might clamp to a reasonable maximum
    println!("✓ Large max stream count handled: {}", retrieved);
}

#[test]
fn test_picker_api_availability() {
    // Verify the API is available on macOS 14.0+
    let _picker_type = std::any::type_name::<SCContentSharingPicker>();
    let _config_type = std::any::type_name::<SCContentSharingPickerConfiguration>();
    
    println!("✓ Content sharing picker API available on macOS 14.0+");
}

#[test]
fn test_picker_configuration_modes() {
    use screencapturekit::content_sharing_picker::SCContentSharingPickerMode;
    
    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(&[
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
    ]);
    // Just verify it doesn't crash
    assert!(!config.as_ptr().is_null());
}
