//! Configuration Presets and Advanced Settings
//!
//! Demonstrates advanced stream configuration options:
//! - Configuration presets (macOS 15.0+)
//! - HDR capture settings
//! - Presenter overlay privacy settings
//! - Microphone capture configuration
//!
//! Run with: `cargo run --example 13_advanced_config --features macos_15_0`

#[cfg(not(feature = "macos_15_0"))]
fn main() {
    println!("⚠️  This example requires macOS 15.0+ features");
    println!("    Run with: cargo run --example 13_advanced_config --features macos_15_0");
}

#[cfg(feature = "macos_15_0")]
#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::prelude::*;
    use screencapturekit::stream::configuration::{
        SCCaptureDynamicRange, SCPresenterOverlayAlertSetting, SCStreamConfigurationPreset,
    };

    println!("⚙️  Advanced Configuration Options\n");

    // ========================================
    // 1. Configuration Presets (macOS 15.0+)
    // ========================================
    println!("📋 1. Configuration Presets");
    println!("   ─────────────────────────");

    // SDR capture preset
    let _sdr_config = SCStreamConfiguration::from_preset(
        SCStreamConfigurationPreset::CaptureHDRStreamLocalDisplay,
    );
    println!("   HDR Local Display preset created");

    // Canonical HDR preset
    let _hdr_config = SCStreamConfiguration::from_preset(
        SCStreamConfigurationPreset::CaptureHDRStreamCanonicalDisplay,
    );
    println!("   HDR Canonical Display preset created");

    // ========================================
    // 2. HDR / Dynamic Range Settings
    // ========================================
    println!("\n🌈 2. Dynamic Range Settings");
    println!("   ─────────────────────────");

    let mut config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    // Set dynamic range
    config.set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay);
    println!("   Dynamic range: HDR Local Display");

    let range = config.capture_dynamic_range();
    println!("   Current setting: {range:?}");

    // Available options:
    println!("\n   Available dynamic range options:");
    println!("   • SDR - Standard dynamic range");
    println!("   • HdrLocalDisplay - HDR for local display");
    println!("   • HdrCanonicalDisplay - HDR canonical (for sharing)");

    // ========================================
    // 3. Presenter Overlay Settings
    // ========================================
    println!("\n👤 3. Presenter Overlay Privacy");
    println!("   ─────────────────────────────");

    config.set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Always);
    println!("   Set to: Always show privacy alert");

    let setting = config.presenter_overlay_privacy_alert_setting();
    println!("   Current: {setting:?}");

    println!("\n   Available settings:");
    println!("   • System - Follow system preference");
    println!("   • Never - Never show overlay");
    println!("   • Always - Always show when presenter detected");

    // ========================================
    // 4. Microphone Capture
    // ========================================
    println!("\n🎤 4. Microphone Capture Configuration");
    println!("   ─────────────────────────────────────");

    // Enable microphone capture
    config.set_captures_microphone(true);
    println!("   Microphone capture: enabled");

    // List available audio devices
    let devices = screencapturekit::audio_devices::AudioInputDevice::list();
    if !devices.is_empty() {
        println!("\n   Available audio devices:");
        for (i, device) in devices.iter().take(5).enumerate() {
            println!("     {}. {} (ID: {})", i + 1, device.name, device.id);
        }

        // Set specific microphone device
        if let Some(mic) = devices.first() {
            config.set_microphone_capture_device_id(&mic.id);
            println!("\n   Selected microphone: {}", mic.name);
        }
    }

    // ========================================
    // 5. Mouse Click Indicators
    // ========================================
    println!("\n🖱️  5. Mouse Click Indicators");
    println!("   ────────────────────────────");

    config.set_shows_mouse_clicks(true);
    println!("   Mouse click indicators: enabled");
    println!("   (Shows visual feedback when user clicks)");

    // ========================================
    // 6. Child Windows
    // ========================================
    println!("\n🪟 6. Child Window Inclusion");
    println!("   ──────────────────────────");

    config.set_includes_child_windows(true);
    println!("   Include child windows: enabled");
    println!("   (Captures tooltips, menus, etc.)");

    // ========================================
    // Summary
    // ========================================
    println!("\n═══════════════════════════════════════════");
    println!("📊 Final Configuration Summary:");
    println!("   • Dimensions: {}x{}", config.width(), config.height());
    println!("   • Dynamic range: {:?}", config.capture_dynamic_range());
    println!(
        "   • Presenter overlay: {:?}",
        config.presenter_overlay_privacy_alert_setting()
    );
    println!("   • Microphone: {}", config.captures_microphone());
    println!("   • Mouse clicks: {}", config.shows_mouse_clicks());
    println!("   • Child windows: {}", config.includes_child_windows());

    println!("\n✅ Advanced configuration example complete!");
    Ok(())
}
