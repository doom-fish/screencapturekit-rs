//! List Available Content
//!
//! Demonstrates listing all available shareable content.
//! This example shows:
//! - Getting displays
//! - Getting windows
//! - Getting applications
//! - Filtering content

use screencapturekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Available Shareable Content\n");

    // Get all shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let windows = content.windows();
    let applications = content.applications();

    // List displays
    println!("🖥️  Displays ({}):", displays.len());
    for display in displays {
        println!("  - ID: {}", display.display_id());
        println!("    Size: {}x{}", display.width(), display.height());
        println!("    Frame: {:?}", display.frame());
    }

    // List windows (first 10)
    println!("\n🪟 Windows (showing first 10 of {}):", windows.len());
    for window in windows.iter().take(10) {
        let app_name = window.owning_application()
            .as_ref()
            .map(|app| app.application_name())
            .unwrap_or_else(|| "Unknown".to_string());
        let title = window.title().as_deref().unwrap_or("Untitled").to_string();
        
        println!("  - {} - {}", app_name, title);
        println!("    Window ID: {}", window.window_id());
        println!("    Size: {}x{}", window.frame().width, window.frame().height);
        println!("    Layer: {}", window.window_layer());
        println!("    On screen: {}", window.is_on_screen());
    }

    // List applications
    println!("\n📱 Applications ({}):", applications.len());
    for app in applications {
        println!("  - {}", app.application_name());
        println!("    Bundle ID: {}", app.bundle_identifier());
        println!("    PID: {}", app.process_id());
    }

    // Filter windows by app
    println!("\n🔍 Filter Example - Safari Windows:");
    let safari_windows: Vec<_> = windows
        .iter()
        .filter(|w| {
            w.owning_application()
                .as_ref()
                .map(|app| app.application_name().contains("Safari"))
                .unwrap_or(false)
        })
        .collect();
    
    println!("Found {} Safari windows", safari_windows.len());
    for window in safari_windows {
        let title = window.title().as_deref().unwrap_or("Untitled").to_string();
        println!("  - {}", title);
    }

    // Custom filtering options
    println!("\n⚙️  Custom Filtering:");
    let filtered = SCShareableContent::with_options()
        .on_screen_windows_only(true)
        .exclude_desktop_windows(true)
        .get()?;
    
    println!("On-screen windows only: {}", filtered.windows().len());

    Ok(())
}
