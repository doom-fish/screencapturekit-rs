//! Application-Based Capture
//!
//! Demonstrates capturing specific applications.
//! This example shows:
//! - Filtering by application
//! - Including/excluding specific apps
//! - Capturing multiple applications
//!
//! Run with: `cargo run --example 14_app_capture`

use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct FrameCounter {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameCounter {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if matches!(of_type, SCStreamOutputType::Screen) {
            let n = self.count.fetch_add(1, Ordering::Relaxed);
            if n % 30 == 0 {
                println!("📹 Frame {n}");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📱 Application-Based Capture\n");

    // 1. Get available content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    let apps = content.applications();
    let windows = content.windows();

    let display = displays.first().ok_or("No displays found")?;

    // 2. List running applications
    println!("📋 Running Applications:");
    for (i, app) in apps.iter().take(10).enumerate() {
        let window_count = windows
            .iter()
            .filter(|w| {
                w.owning_application()
                    .is_some_and(|a| a.process_id() == app.process_id())
            })
            .count();

        println!(
            "   {}. {} (PID: {}, Windows: {})",
            i + 1,
            app.application_name(),
            app.process_id(),
            window_count
        );
    }

    // 3. Find a specific application (example: Finder)
    let target_app = apps
        .iter()
        .find(|a| a.application_name().contains("Finder"))
        .or_else(|| apps.first());

    let Some(app) = target_app else {
        println!("⚠️  No applications found");
        return Ok(());
    };

    println!(
        "\n🎯 Target: {} ({})",
        app.application_name(),
        app.bundle_identifier()
    );

    // ========================================
    // Option A: Capture INCLUDING specific apps
    // ========================================
    println!("\n📦 Option A: Include specific application");

    let include_filter = SCContentFilter::with()
        .display(display)
        .include_applications(&[app], &[])
        .build();

    println!("   Filter created: include only {}", app.application_name());

    // ========================================
    // Option B: Capture EXCLUDING specific apps
    // ========================================
    println!("\n📦 Option B: Exclude specific application");

    let _exclude_filter = SCContentFilter::with()
        .display(display)
        .exclude_applications(&[app], &[])
        .build();

    println!("   Filter created: exclude {}", app.application_name());

    // ========================================
    // Option C: Capture multiple applications
    // ========================================
    println!("\n📦 Option C: Multiple applications");

    // Get first 3 apps with visible windows
    let multi_apps: Vec<_> = apps
        .iter()
        .filter(|a| {
            windows.iter().any(|w| {
                w.is_on_screen()
                    && w.owning_application()
                        .is_some_and(|oa| oa.process_id() == a.process_id())
            })
        })
        .take(3)
        .collect();

    if !multi_apps.is_empty() {
        let _multi_filter = SCContentFilter::with()
            .display(display)
            .include_applications(&multi_apps, &[])
            .build();

        println!("   Filter created for {} apps:", multi_apps.len());
        for a in &multi_apps {
            println!("     • {}", a.application_name());
        }
    }

    // ========================================
    // Demo: Capture with the include filter
    // ========================================
    println!("\n▶️  Starting capture (include filter)...");

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameCounter {
        count: count.clone(),
    };

    let mut stream = SCStream::new(&include_filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    stream.stop_capture()?;

    println!("\n⏹️  Capture stopped");
    println!(
        "✅ Captured {} frames of {}",
        count.load(Ordering::Relaxed),
        app.application_name()
    );

    Ok(())
}
