//! Content Sharing Picker Example
//!
//! Demonstrates the system UI for selecting content to share (macOS 14.0+).
//!
//! Run with: cargo run --example `11_content_picker` --features `macos_14_0`

#[cfg(not(feature = "macos_14_0"))]
fn main() {
    eprintln!("This example requires the 'macos_14_0' feature flag.");
    eprintln!("Run with: cargo run --example 11_content_picker --features macos_14_0");
}

#[cfg(feature = "macos_14_0")]
fn main() {
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    };

    println!("=== Content Sharing Picker Example (macOS 14.0+) ===\n");

    // Create picker configuration
    let mut config = SCContentSharingPickerConfiguration::new();

    // Set allowed picker modes
    config.set_allowed_picker_modes(&[
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
        SCContentSharingPickerMode::Multiple,
    ]);

    println!("📋 Picker Configuration:");
    println!("   Allowed modes: SingleWindow, SingleDisplay, Multiple");
    println!("   Pointer: {:?}", config.as_ptr());

    // Test configuration cloning
    let config_clone = config.clone();
    println!("\n📋 Configuration cloned:");
    println!("   Original ptr: {:?}", config.as_ptr());
    println!("   Clone ptr: {:?}", config_clone.as_ptr());

    // Show available picker modes
    println!("\n🎯 Available Picker Modes:");
    println!(
        "   SingleWindow = {}",
        SCContentSharingPickerMode::SingleWindow as i32
    );
    println!(
        "   Multiple = {}",
        SCContentSharingPickerMode::Multiple as i32
    );
    println!(
        "   SingleDisplay = {}",
        SCContentSharingPickerMode::SingleDisplay as i32
    );

    // Note: Actually showing the picker requires user interaction
    // and is typically done in a GUI application context.
    println!("\n⚠️  Note: The picker UI requires a GUI context to display.");
    println!("   In a real application, call SCContentSharingPicker::show(&config)");
    println!("   to present the system picker UI to the user.\n");

    // Demonstrate the result types
    println!("📦 Possible Result Types:");
    println!("   - SCContentSharingPickerResult::Display(display)");
    println!("   - SCContentSharingPickerResult::Window(window)");
    println!("   - SCContentSharingPickerResult::Application(app)");
    println!("   - SCContentSharingPickerResult::Cancelled");
    println!("   - SCContentSharingPickerResult::Error(message)");

    // Example of how to use the picker (commented out as it needs GUI)
    /*
    let result = SCContentSharingPicker::show(&config);
    match result {
        SCContentSharingPickerResult::Display(display) => {
            println!("Selected display: {:?}", display.display_id());
        }
        SCContentSharingPickerResult::Window(window) => {
            println!("Selected window: {:?}", window.title());
        }
        SCContentSharingPickerResult::Application(app) => {
            println!("Selected app: {}", app.application_name());
        }
        SCContentSharingPickerResult::Cancelled => {
            println!("User cancelled the picker");
        }
        SCContentSharingPickerResult::Error(msg) => {
            eprintln!("Picker error: {}", msg);
        }
    }
    */

    println!("\n✅ Content sharing picker example completed!");
}
