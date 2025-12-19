//! Input handling and picker utilities

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use screencapturekit::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    SCPickedSource, SCPickerOutcome,
};
use screencapturekit::prelude::*;

use crate::capture::{CaptureHandler, CaptureState};

/// Result type for picker callbacks: (filter, width, height, source)
pub type PickerResult = Option<(SCContentFilter, u32, u32, SCPickedSource)>;

/// Format a picked source for display
pub fn format_picked_source(source: &SCPickedSource) -> String {
    match source {
        SCPickedSource::Window(name) => {
            format!("[W] {}", name.chars().take(20).collect::<String>())
        }
        SCPickedSource::Display(id) => format!("[D] Display {id}"),
        SCPickedSource::Application(name) => {
            format!("[A] {}", name.chars().take(20).collect::<String>())
        }
        SCPickedSource::Unknown => "None".to_string(),
    }
}

/// All picker modes we allow
const ALL_PICKER_MODES: &[SCContentSharingPickerMode] = &[
    SCContentSharingPickerMode::SingleWindow,
    SCContentSharingPickerMode::MultipleWindows,
    SCContentSharingPickerMode::SingleDisplay,
    SCContentSharingPickerMode::SingleApplication,
    SCContentSharingPickerMode::MultipleApplications,
];

/// Create picker configuration with all modes enabled
fn create_picker_config() -> SCContentSharingPickerConfiguration {
    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(ALL_PICKER_MODES);
    config
}

/// Open content picker without an existing stream
pub fn open_picker(pending_picker: &Arc<Mutex<PickerResult>>) {
    println!("📺 Opening content picker...");
    let config = create_picker_config();
    let pending = Arc::clone(pending_picker);

    SCContentSharingPicker::show(&config, move |outcome| {
        handle_picker_outcome(outcome, &pending);
    });
}

/// Open content picker for an existing stream
pub fn open_picker_for_stream(pending_picker: &Arc<Mutex<PickerResult>>, stream: &SCStream) {
    println!("📺 Opening content picker for stream...");
    let config = create_picker_config();
    let pending = Arc::clone(pending_picker);

    SCContentSharingPicker::show_for_stream(&config, stream, move |outcome| {
        handle_picker_outcome(outcome, &pending);
    });
}

fn handle_picker_outcome(outcome: SCPickerOutcome, pending: &Arc<Mutex<PickerResult>>) {
    match outcome {
        SCPickerOutcome::Picked(result) => {
            let (width, height) = result.pixel_size();
            let filter = result.filter();
            let source = result.source();

            if let Ok(mut pending) = pending.lock() {
                *pending = Some((filter, width, height, source));
            }
        }
        SCPickerOutcome::Cancelled => {
            println!("⚠️  Picker cancelled");
        }
        SCPickerOutcome::Error(e) => {
            eprintln!("❌ Picker error: {e}");
        }
    }
}

/// Start capture with the given filter and configuration
#[allow(clippy::option_option)]
pub fn start_capture(
    stream: &mut Option<SCStream>,
    current_filter: Option<&SCContentFilter>,
    capture_size: (u32, u32),
    stream_config: &SCStreamConfiguration,
    capture_state: &Arc<CaptureState>,
    capturing: &Arc<AtomicBool>,
    mic_only: bool,
) {
    // Get the filter to use
    let filter_to_use = if let Some(filter) = current_filter {
        filter.clone()
    } else if mic_only {
        // For mic-only capture, we still need a valid display filter
        println!("🎤 Starting mic-only capture (using main display)");
        match screencapturekit::shareable_content::SCShareableContent::get() {
            Ok(content) => {
                let displays = content.displays();
                if let Some(display) = displays.first() {
                    SCContentFilter::create()
                        .with_display(display)
                        .with_excluding_windows(&[])
                        .build()
                } else {
                    println!("❌ No displays available for mic-only capture");
                    return;
                }
            }
            Err(e) => {
                println!("❌ Failed to get shareable content: {e:?}");
                return;
            }
        }
    } else {
        println!("⚠️  No content selected. Open picker first.");
        return;
    };

    let (width, height) = capture_size;
    let mut sc_config = stream_config.clone();
    sc_config.set_width(width);
    sc_config.set_height(height);

    let handler = CaptureHandler {
        state: Arc::clone(capture_state),
    };

    let mut s = SCStream::new(&filter_to_use, &sc_config);
    if !mic_only {
        s.add_output_handler(handler.clone(), SCStreamOutputType::Screen);
        s.add_output_handler(handler.clone(), SCStreamOutputType::Audio);
    }
    s.add_output_handler(handler, SCStreamOutputType::Microphone);

    match s.start_capture() {
        Ok(()) => {
            capturing.store(true, Ordering::Relaxed);
            *stream = Some(s);
            println!("✅ Capture started");
        }
        Err(e) => {
            eprintln!("❌ Failed to start capture: {e:?}");
        }
    }
}

/// Stop the current capture
pub fn stop_capture(stream: &mut Option<SCStream>, capturing: &Arc<AtomicBool>) {
    println!("⏹️  Stopping capture...");
    if let Some(ref mut s) = stream {
        let _ = s.stop_capture();
    }
    *stream = None;
    capturing.store(false, Ordering::Relaxed);
    println!("✅ Capture stopped");
}
