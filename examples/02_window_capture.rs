//! Window Capture
//!
//! Demonstrates capturing a specific window.
//! This example shows:
//! - Listing available windows
//! - Filtering windows by title
//! - Creating window-specific content filter
//!
//! Uses the batched [`SCShareableContent::snapshot`] API for the windows
//! listing — populates every window's title, frame, owning-app, etc. in a
//! single FFI round-trip instead of paying per-attribute FFI for each of
//! the ~200 windows on a typical desktop.

use screencapturekit::prelude::*;
use screencapturekit::shareable_content::ContentSnapshot;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Initialize CoreGraphics to prevent CGS_REQUIRE_INIT crashes
fn init_cg() {
    extern "C" {
        fn sc_initialize_core_graphics();
    }
    unsafe { sc_initialize_core_graphics() }
}

struct FrameHandler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _type: SCStreamOutputType) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_cg();
    println!("🪟 Window Capture\n");

    // 1. Get available content + a batched snapshot for fast iteration.
    let content = SCShareableContent::get()?;
    let ContentSnapshot {
        applications: app_snaps,
        windows: window_snaps,
        ..
    } = content
        .snapshot()
        .ok_or("Could not collect content snapshot")?;

    // 2. List windows (optional - for demonstration). Pure in-memory walk.
    println!("Available windows:");
    for (i, w) in window_snaps.iter().take(10).enumerate() {
        let app_name = w
            .owning_app_index
            .and_then(|idx| app_snaps.get(idx))
            .map_or("", |a| a.application_name.as_str());

        println!(
            "  {}. {} - {} ({}x{})",
            i + 1,
            app_name,
            w.title.as_deref().unwrap_or(""),
            w.frame.width,
            w.frame.height
        );
    }

    // 3. Find a window — prefer Safari, fallback to any visible titled
    // window. All these checks are pure data access on the snapshot rows.
    let safari_app_indices: Vec<usize> = app_snaps
        .iter()
        .enumerate()
        .filter(|(_, a)| a.application_name.contains("Safari"))
        .map(|(i, _)| i)
        .collect();

    let chosen_window_id = window_snaps
        .iter()
        .find(|w| {
            w.owning_app_index
                .is_some_and(|i| safari_app_indices.contains(&i))
        })
        .or_else(|| {
            window_snaps.iter().find(|w| {
                w.is_on_screen
                    && w.frame.width > 100.0
                    && w.frame.height > 100.0
                    && w.title.as_deref().is_some_and(|t| !t.is_empty())
            })
        })
        .or_else(|| window_snaps.iter().find(|w| w.is_on_screen))
        .map(|w| w.window_id)
        .ok_or("No suitable window found")?;

    // 4. Re-fetch the live SCWindow handle for the chosen ID — needed to
    // build a content filter (filters require live pointers, not snapshot
    // data).
    let live_windows = content.windows();
    let window = live_windows
        .iter()
        .find(|w| w.window_id() == chosen_window_id)
        .ok_or("chosen window vanished")?;

    let chosen_snap = window_snaps
        .iter()
        .find(|w| w.window_id == chosen_window_id)
        .expect("just selected");
    let chosen_app_name = chosen_snap
        .owning_app_index
        .and_then(|i| app_snaps.get(i))
        .map_or("", |a| a.application_name.as_str());
    println!(
        "\nCapturing: {} - {}\n",
        chosen_app_name,
        chosen_snap.title.as_deref().unwrap_or("")
    );

    // 5. Create window filter
    let filter = SCContentFilter::create().with_window(window).build();

    // 6. Configure stream
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);

    // 7. Start capture
    let count = Arc::new(AtomicUsize::new(0));
    let handler = FrameHandler {
        count: count.clone(),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;

    println!("✅ Captured {} frames", count.load(Ordering::Relaxed));
    Ok(())
}
