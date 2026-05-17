//! Application-Based Capture
//!
//! Demonstrates capturing specific applications.
//! This example shows:
//! - Filtering by application
//! - Including/excluding specific apps
//! - Capturing multiple applications
//!
//! Run with: `cargo run --example 14_app_capture --features macos_14_0`
//!
//! ## Performance note
//!
//! The "list app + count its windows" pattern at the top is O(apps × windows)
//! — with the per-element accessor pattern that's `apps × windows × 2` FFI
//! calls (`.owning_application()` + `.process_id()` per window). On a
//! 36-app / 220-window system that's ~16k FFI calls just to print the
//! header. We use [`SCShareableContent::snapshot`] instead, which fetches
//! every owning-app index in a single batched FFI round-trip and lets us
//! count windows per app in pure Rust (`Counter` over snapshot rows).

use screencapturekit::prelude::*;
use screencapturekit::shareable_content::ContentSnapshot;
use std::collections::HashMap;
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

    // 1. Get available content + batched snapshot.
    let content = SCShareableContent::get()?;
    let ContentSnapshot {
        displays: display_snaps,
        applications: app_snaps,
        windows: window_snaps,
    } = content
        .snapshot()
        .ok_or("Could not collect content snapshot")?;

    if display_snaps.is_empty() {
        return Err("No displays found".into());
    }

    // 2. List running applications + window counts (purely in-memory now).
    let mut windows_per_app: HashMap<usize, usize> = HashMap::new();
    for w in &window_snaps {
        if let Some(idx) = w.owning_app_index {
            *windows_per_app.entry(idx).or_insert(0) += 1;
        }
    }

    println!("📋 Running Applications:");
    for (i, app) in app_snaps.iter().take(10).enumerate() {
        let window_count = windows_per_app.get(&i).copied().unwrap_or(0);
        println!(
            "   {}. {} (PID: {}, Windows: {})",
            i + 1,
            app.application_name,
            app.process_id,
            window_count
        );
    }

    // 3. Find a specific application (example: Finder).
    let target_idx = app_snaps
        .iter()
        .position(|a| a.application_name.contains("Finder"))
        .or(if app_snaps.is_empty() { None } else { Some(0) });

    let Some(target_idx) = target_idx else {
        println!("⚠️  No applications found");
        return Ok(());
    };
    let app_snap = &app_snaps[target_idx];

    println!(
        "\n🎯 Target: {} ({})",
        app_snap.application_name, app_snap.bundle_identifier
    );

    // 4. To actually build a content filter we need the live SCRunningApplication
    // handle (the snapshot only carries plain data). Re-fetch the live list
    // and find by PID — this is one FFI per app, but we only do it once.
    let live_apps = content.applications();
    let live_displays = content.displays();
    let display = live_displays.first().ok_or("display vanished")?;

    let live_app = live_apps
        .iter()
        .find(|a| a.process_id() == app_snap.process_id)
        .ok_or("live app vanished between snapshot and lookup")?;

    // ========================================
    // Option A: Capture INCLUDING specific apps
    // ========================================
    println!("\n📦 Option A: Include specific application");

    let include_filter = SCContentFilter::create()
        .with_display(display)
        .with_including_applications(&[live_app], &[])
        .build();

    println!(
        "   Filter created: include only {}",
        app_snap.application_name
    );

    // ========================================
    // Option B: Capture EXCLUDING specific apps
    // ========================================
    println!("\n📦 Option B: Exclude specific application");

    let _exclude_filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_applications(&[live_app], &[])
        .build();

    println!("   Filter created: exclude {}", app_snap.application_name);

    // ========================================
    // Option C: Capture multiple applications
    // ========================================
    println!("\n📦 Option C: Multiple applications");

    // Find apps with visible windows — pure in-memory walk over snapshot rows.
    let visible_app_indices: Vec<usize> = (0..app_snaps.len())
        .filter(|i| {
            window_snaps
                .iter()
                .any(|w| w.is_on_screen && w.owning_app_index == Some(*i))
        })
        .take(3)
        .collect();

    // For the actual filter we still need the live SCRunningApplication
    // handles — match by PID once.
    let multi_apps: Vec<&_> = visible_app_indices
        .iter()
        .filter_map(|i| {
            let pid = app_snaps[*i].process_id;
            live_apps.iter().find(|a| a.process_id() == pid)
        })
        .collect();

    if !multi_apps.is_empty() {
        let _multi_filter = SCContentFilter::create()
            .with_display(display)
            .with_including_applications(&multi_apps, &[])
            .build();

        println!("   Filter created for {} apps:", multi_apps.len());
        for &i in &visible_app_indices {
            println!("     • {}", app_snaps[i].application_name);
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
        app_snap.application_name
    );

    Ok(())
}
