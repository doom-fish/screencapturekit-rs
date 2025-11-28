//! Content Sharing Picker tests (macOS 14.0+)
//!
//! These tests verify the content sharing picker functionality

#![cfg(feature = "macos_14_0")]

use screencapturekit::content_sharing_picker::{
    SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
};

#[test]
fn test_picker_configuration_creation() {
    // Test basic picker configuration creation
    let config = SCContentSharingPickerConfiguration::new();
    assert!(
        !config.as_ptr().is_null(),
        "Configuration pointer should not be null"
    );
    println!("✅ Picker configuration created");
}

#[test]
fn test_picker_configuration_modes() {
    // Test setting different picker modes
    let mut config = SCContentSharingPickerConfiguration::new();

    let modes = vec![
        vec![SCContentSharingPickerMode::SingleWindow],
        vec![SCContentSharingPickerMode::Multiple],
        vec![SCContentSharingPickerMode::SingleDisplay],
        vec![
            SCContentSharingPickerMode::SingleWindow,
            SCContentSharingPickerMode::SingleDisplay,
        ],
        vec![
            SCContentSharingPickerMode::SingleWindow,
            SCContentSharingPickerMode::Multiple,
            SCContentSharingPickerMode::SingleDisplay,
        ],
    ];

    for mode_set in modes {
        config.set_allowed_picker_modes(&mode_set);
        println!("✅ Set picker modes: {mode_set:?}");
    }
}

#[test]
fn test_picker_mode_values() {
    // Test that picker mode enum values are correct
    assert_eq!(SCContentSharingPickerMode::SingleWindow as i32, 0);
    assert_eq!(SCContentSharingPickerMode::Multiple as i32, 1);
    assert_eq!(SCContentSharingPickerMode::SingleDisplay as i32, 2);
    println!("✅ Picker mode values correct");
}

#[test]
fn test_picker_configuration_reuse() {
    // Test reusing the same configuration
    let mut config = SCContentSharingPickerConfiguration::new();

    // Set modes multiple times
    config.set_allowed_picker_modes(&[SCContentSharingPickerMode::SingleWindow]);
    config.set_allowed_picker_modes(&[SCContentSharingPickerMode::SingleDisplay]);
    config.set_allowed_picker_modes(&[
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
    ]);

    println!("✅ Configuration can be reused");
}

#[test]
fn test_multiple_picker_configurations() {
    // Test creating multiple configurations
    let configs: Vec<_> = (0..5)
        .map(|_| SCContentSharingPickerConfiguration::new())
        .collect();

    // Verify all have valid pointers
    for (i, config) in configs.iter().enumerate() {
        assert!(
            !config.as_ptr().is_null(),
            "Config {i} should have valid pointer"
        );
    }

    println!("✅ Multiple configurations created");
}

#[test]
fn test_picker_configuration_drop() {
    // Test that configurations are properly cleaned up
    {
        let _config = SCContentSharingPickerConfiguration::new();
        println!("Configuration created in scope");
    }
    println!("✅ Configuration dropped successfully");
}

#[test]
fn test_empty_modes_array() {
    // Test setting empty modes array
    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(&[]);
    println!("✅ Empty modes array handled");
}

#[test]
fn test_picker_mode_equality() {
    // Test that picker modes can be compared
    let mode1 = SCContentSharingPickerMode::SingleWindow;
    let mode2 = SCContentSharingPickerMode::SingleWindow;
    let mode3 = SCContentSharingPickerMode::Multiple;

    assert_eq!(mode1, mode2, "Same modes should be equal");
    assert_ne!(mode1, mode3, "Different modes should not be equal");
    println!("✅ Picker mode equality works");
}

#[test]
fn test_picker_mode_hash() {
    // Test that picker modes can be used in hash sets
    use std::collections::HashSet;

    let mut modes = HashSet::new();
    modes.insert(SCContentSharingPickerMode::SingleWindow);
    modes.insert(SCContentSharingPickerMode::Multiple);
    modes.insert(SCContentSharingPickerMode::SingleWindow); // Duplicate

    assert_eq!(modes.len(), 2, "Should have 2 unique modes");
    println!("✅ Picker modes can be hashed");
}

#[test]
fn test_picker_mode_clone() {
    // Test that picker modes can be cloned
    let mode1 = SCContentSharingPickerMode::SingleWindow;
    let mode2 = mode1;

    assert_eq!(mode1, mode2, "Cloned modes should be equal");
    println!("✅ Picker modes can be cloned");
}

#[test]
fn test_picker_mode_debug() {
    // Test that picker modes have debug formatting
    let modes = vec![
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::Multiple,
        SCContentSharingPickerMode::SingleDisplay,
    ];

    for mode in modes {
        let debug_str = format!("{mode:?}");
        assert!(!debug_str.is_empty(), "Debug string should not be empty");
        println!("Mode: {debug_str}");
    }

    println!("✅ Picker modes have debug formatting");
}
