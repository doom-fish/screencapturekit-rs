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
        SCContentSharingPickerMode::MultipleWindows,
        SCContentSharingPickerMode::SingleDisplay,
        SCContentSharingPickerMode::SingleApplication,
        SCContentSharingPickerMode::MultipleApplications,
    ]);

    println!("📋 Picker Configuration:");
    println!("   Allowed modes: SingleWindow, MultipleWindows, SingleDisplay, SingleApplication, MultipleApplications");
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
        "   MultipleWindows = {}",
        SCContentSharingPickerMode::MultipleWindows as i32
    );
    println!(
        "   SingleDisplay = {}",
        SCContentSharingPickerMode::SingleDisplay as i32
    );
    println!(
        "   SingleApplication = {}",
        SCContentSharingPickerMode::SingleApplication as i32
    );
    println!(
        "   MultipleApplications = {}",
        SCContentSharingPickerMode::MultipleApplications as i32
    );

    // Note: Actually showing the picker requires user interaction
    // and is typically done in a GUI application context.
    println!("\n⚠️  Note: The picker UI requires a GUI context to display.");
    println!("   In a real application, call SCContentSharingPicker::show(&config)");
    println!("   to present the system picker UI to the user.\n");

    // Demonstrate the result types
    println!("📦 Picker APIs:");
    println!("   Main API:");
    println!("     SCContentSharingPicker::pick(&config) -> SCPickerOutcome");
    println!("       - SCPickerOutcome::Picked(result)  // result.filter(), result.windows(), result.displays()");
    println!("       - SCPickerOutcome::Cancelled");
    println!("       - SCPickerOutcome::Error(message)");
    println!();
    println!("   Simple API:");
    println!("     SCContentSharingPicker::pick_filter(&config) -> SCPickerFilterOutcome");
    println!("       - SCPickerFilterOutcome::Filter(filter)");
    println!("       - SCPickerFilterOutcome::Cancelled");
    println!("       - SCPickerFilterOutcome::Error(message)");

    // Example of how to use the picker (commented out as it needs GUI)
    /*
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPicker, SCPickerOutcome,
    };
    
    let result = SCContentSharingPicker::pick(&config);
    match result {
        SCPickerOutcome::Picked(result) => {
            // Get filter + metadata
            let filter = result.filter();
            let (width, height) = result.pixel_size();
            
            // Access picked content for custom filters
            for window in result.windows() {
                println!("Selected window: {:?}", window.title());
            }
            for display in result.displays() {
                println!("Selected display: {:?}", display.display_id());
            }
        }
        SCPickerOutcome::Cancelled => {
            println!("User cancelled the picker");
        }
        SCPickerOutcome::Error(msg) => {
            eprintln!("Picker error: {}", msg);
        }
    }
    */

    println!("\n✅ Content sharing picker example completed!");
}
