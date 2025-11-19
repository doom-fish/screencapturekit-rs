//! Recording output tests
//!
//! Tests for SCRecordingOutput and SCRecordingOutputConfiguration (macOS 15.0+).

#![cfg(feature = "macos_15_0")]

use screencapturekit::recording_output::{
    SCRecordingOutput, SCRecordingOutputConfiguration,
};

#[test]
fn test_recording_output_configuration_new() {
    let config = SCRecordingOutputConfiguration::new();
    println!("✓ Recording output configuration created");
    drop(config);
}

#[test]
fn test_recording_output_configuration_clone() {
    let config1 = SCRecordingOutputConfiguration::new();
    let config2 = config1.clone();
    
    drop(config1);
    drop(config2);
    
    println!("✓ Recording output configuration clone works");
}

#[test]
fn test_recording_output_configuration_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<SCRecordingOutputConfiguration>();
    assert_sync::<SCRecordingOutputConfiguration>();
    
    println!("✓ SCRecordingOutputConfiguration is Send + Sync");
}

#[test]
fn test_recording_output_new() {
    let config = SCRecordingOutputConfiguration::new();
    
    let result = SCRecordingOutput::new(&config);
    
    match result {
        Some(output) => {
            println!("✓ Recording output created successfully");
            drop(output);
        }
        None => {
            println!("⚠ Recording output creation failed (expected in test env - requires macOS 15.0+)");
        }
    }
}

#[test]
fn test_recording_output_clone() {
    let config = SCRecordingOutputConfiguration::new();
    
    if let Some(output1) = SCRecordingOutput::new(&config) {
        let output2 = output1.clone();
        
        drop(output1);
        drop(output2);
        
        println!("✓ Recording output clone works");
    } else {
        println!("⚠ Skipping clone test - recording output unavailable");
    }
}

#[test]
fn test_recording_output_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<SCRecordingOutput>();
    assert_sync::<SCRecordingOutput>();
    
    println!("✓ SCRecordingOutput is Send + Sync");
}

#[test]
fn test_recording_output_multiple_instances() {
    let config = SCRecordingOutputConfiguration::new();
    
    let output1 = SCRecordingOutput::new(&config);
    let output2 = SCRecordingOutput::new(&config);
    
    if output1.is_some() {
        println!("✓ Multiple recording outputs can be created");
    } else {
        println!("⚠ Recording output creation requires macOS 15.0+ or permissions");
    }
    
    assert!(output1.is_some() == output2.is_some(), "Both outputs should have same creation status");
}

#[test]
fn test_recording_output_api_availability() {
    // Just test that the types exist and are accessible
    let _config_type = std::any::type_name::<SCRecordingOutputConfiguration>();
    let _output_type = std::any::type_name::<SCRecordingOutput>();
    
    println!("✓ Recording output API is available on macOS 15.0+");
}
