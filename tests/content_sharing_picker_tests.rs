//! Content sharing picker tests
//!
//! Tests for SCContentSharingPickerConfiguration (macOS 14.0+).

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
        println!("✓ Configuration {} created", i);
        drop(config);
        println!("✓ Configuration {} dropped", i);
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
