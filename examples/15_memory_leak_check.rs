//! Memory Leak Detection Example
//!
//! This example demonstrates how to check for memory leaks using macOS's `leaks` tool.
//! It creates and destroys streams multiple times, then uses the `leaks` command to
//! verify no memory is leaked.
//!
//! # Tested API Surface
//! - `SCShareableContent`: displays, windows, applications
//! - `SCContentFilter`: all filter types (display, window, app inclusion/exclusion)
//! - `SCStreamConfiguration`: video, audio, microphone settings
//! - `SCStream`: start/stop, output handlers for all types
//! - `SCDisplay`, `SCWindow`, `SCRunningApplication`: property access
//!
//! # Usage
//! ```sh
//! cargo run --example 15_memory_leak_check
//! ```
//!
//! # Note
//! This uses the macOS `leaks` command which requires running as a standalone process.
//! Some Apple framework leaks in `ScreenCaptureKit` itself are expected and ignored.

use screencapturekit::{
    cg::CGRect,
    cm::CMSampleBuffer,
    shareable_content::{SCRunningApplication, SCShareableContent, SCWindow},
    stream::{
        configuration::{pixel_format::PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
};
use std::{
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

/// Handler that tracks sample counts for all output types
#[allow(clippy::struct_field_names)]
struct LeakTestHandler {
    screen_samples: AtomicUsize,
    audio_samples: AtomicUsize,
    #[cfg(feature = "macos_15_0")]
    mic_samples: AtomicUsize,
}

impl LeakTestHandler {
    const fn new() -> Self {
        Self {
            screen_samples: AtomicUsize::new(0),
            audio_samples: AtomicUsize::new(0),
            #[cfg(feature = "macos_15_0")]
            mic_samples: AtomicUsize::new(0),
        }
    }

    fn report(&self) {
        let screen = self.screen_samples.load(Ordering::Relaxed);
        let audio = self.audio_samples.load(Ordering::Relaxed);
        #[cfg(feature = "macos_15_0")]
        let mic = self.mic_samples.load(Ordering::Relaxed);

        #[cfg(feature = "macos_15_0")]
        println!("    Samples: screen={screen}, audio={audio}, mic={mic}");
        #[cfg(not(feature = "macos_15_0"))]
        println!("    Samples: screen={screen}, audio={audio}");
    }
}

impl SCStreamOutputTrait for LeakTestHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        // Access sample properties to ensure they're valid
        let _timestamp = sample.presentation_timestamp();
        let _duration = sample.duration();

        match of_type {
            SCStreamOutputType::Screen => {
                self.screen_samples.fetch_add(1, Ordering::Relaxed);
            }
            SCStreamOutputType::Audio => {
                self.audio_samples.fetch_add(1, Ordering::Relaxed);
            }
            #[cfg(feature = "macos_15_0")]
            SCStreamOutputType::Microphone => {
                self.mic_samples.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Wrapper to share handler across output types
struct SharedHandler(Arc<LeakTestHandler>);

impl SCStreamOutputTrait for SharedHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.0.did_output_sample_buffer(sample, of_type);
    }
}

fn main() {
    println!("🔍 Memory Leak Detection Test");
    println!("==============================\n");

    let iterations = 3;
    let capture_duration = Duration::from_millis(500);

    println!("Configuration:");
    println!("  • Iterations: {iterations}");
    println!("  • Capture duration per iteration: {capture_duration:?}");
    println!();

    // Test shareable content queries (exercises SCDisplay, SCWindow, SCRunningApplication)
    println!("📋 Testing SCShareableContent queries...");
    test_shareable_content_queries();

    // Test different filter types
    println!("\n📹 Testing different filter configurations...\n");

    for i in 1..=iterations {
        println!("--- Iteration {i}/{iterations} ---\n");

        // Test 1: Display with excluded windows (audio + video)
        println!("  1️⃣  Display filter (exclude windows, audio enabled)");
        test_capture_with_filter(FilterType::DisplayExcludeWindows, &capture_duration);

        // Test 2: Display with included windows
        println!("  2️⃣  Display filter (include windows)");
        test_capture_with_filter(FilterType::DisplayIncludeWindows, &capture_duration);

        // Test 3: Display with excluded apps
        println!("  3️⃣  Display filter (exclude apps)");
        test_capture_with_filter(FilterType::DisplayExcludeApps, &capture_duration);

        // Test 4: Display with included apps
        println!("  4️⃣  Display filter (include apps)");
        test_capture_with_filter(FilterType::DisplayIncludeApps, &capture_duration);

        // Test 5: Single window capture
        println!("  5️⃣  Single window filter");
        test_capture_with_filter(FilterType::SingleWindow, &capture_duration);

        // Test 6: Full config with microphone (macOS 15+)
        #[cfg(feature = "macos_15_0")]
        {
            println!("  6️⃣  Full config (audio + microphone)");
            test_capture_with_filter(FilterType::FullConfigWithMic, &capture_duration);
        }

        println!();
    }

    // Test configuration variations
    println!("⚙️  Testing configuration variations...");
    test_configuration_variations();

    println!("\n🧪 Running leak analysis...\n");

    // Run the macOS leaks command
    let result = check_for_leaks();

    match result {
        LeakResult::NoLeaks => {
            println!("✅ No memory leaks detected!");
        }
        LeakResult::AppleFrameworkLeaksOnly => {
            println!("✅ No leaks in our code (Apple framework leaks detected but ignored)");
        }
        LeakResult::LeaksDetected(details) => {
            println!("❌ Memory leaks detected in our code!");
            println!("\nDetails:\n{details}");
            std::process::exit(1);
        }
        LeakResult::Error(msg) => {
            println!("⚠️  Could not run leak check: {msg}");
            std::process::exit(2);
        }
    }
}

#[derive(Clone, Copy)]
enum FilterType {
    DisplayExcludeWindows,
    DisplayIncludeWindows,
    DisplayExcludeApps,
    DisplayIncludeApps,
    SingleWindow,
    #[cfg(feature = "macos_15_0")]
    FullConfigWithMic,
}

/// Test querying shareable content - exercises property access on all types
fn test_shareable_content_queries() {
    let content = SCShareableContent::get().expect("Failed to get shareable content");

    // Test displays
    let displays = content.displays();
    println!("  Found {} display(s)", displays.len());
    for display in &displays {
        let _id = display.display_id();
        let _width = display.width();
        let _height = display.height();
        let _frame = display.frame();
    }

    // Test windows
    let windows = content.windows();
    println!("  Found {} window(s)", windows.len());
    for window in windows.iter().take(10) {
        let _id = window.window_id();
        let _title = window.title();
        let _frame = window.frame();
        let _on_screen = window.is_on_screen();
        let _layer = window.window_layer();
        let _app = window.owning_application();
    }

    // Test applications
    let apps = content.applications();
    println!("  Found {} application(s)", apps.len());
    for app in apps.iter().take(10) {
        let _name = app.application_name();
        let _bundle_id = app.bundle_identifier();
        let _pid = app.process_id();
    }
}

/// Helper to collect window refs
fn collect_window_refs(windows: &[SCWindow], count: usize) -> Vec<&SCWindow> {
    windows.iter().take(count).collect()
}

/// Helper to collect app refs
fn collect_app_refs(apps: &[SCRunningApplication], count: usize) -> Vec<&SCRunningApplication> {
    apps.iter().take(count).collect()
}

/// Test different filter configurations
fn test_capture_with_filter(filter_type: FilterType, duration: &Duration) {
    let content = SCShareableContent::get().expect("Failed to get shareable content");
    let displays = content.displays();
    let display = displays.first().expect("No display found");
    let windows = content.windows();
    let apps = content.applications();

    let handler = Arc::new(LeakTestHandler::new());

    let filter = match filter_type {
        FilterType::DisplayExcludeWindows => {
            let exclude = collect_window_refs(&windows, 5);
            SCContentFilter::builder()
                .display(display)
                .exclude_windows(&exclude)
                .build()
        }
        FilterType::DisplayIncludeWindows => {
            let include = collect_window_refs(&windows, 3);
            SCContentFilter::builder()
                .display(display)
                .include_windows(&include)
                .build()
        }
        FilterType::DisplayExcludeApps => {
            let exclude_apps = collect_app_refs(&apps, 2);
            let except_windows = collect_window_refs(&windows, 1);
            SCContentFilter::builder()
                .display(display)
                .exclude_applications(&exclude_apps, &except_windows)
                .build()
        }
        FilterType::DisplayIncludeApps => {
            let include_apps = collect_app_refs(&apps, 3);
            let except_windows = collect_window_refs(&windows, 1);
            SCContentFilter::builder()
                .display(display)
                .include_applications(&include_apps, &except_windows)
                .build()
        }
        FilterType::SingleWindow => {
            let window = windows
                .iter()
                .find(|w| w.is_on_screen())
                .unwrap_or(&windows[0]);
            SCContentFilter::builder().window(window).build()
        }
        #[cfg(feature = "macos_15_0")]
        FilterType::FullConfigWithMic => SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build(),
    };

    // Test filter properties
    let _rect = filter.content_rect();
    let _scale = filter.point_pixel_scale();

    // Configure based on filter type
    let config = match filter_type {
        #[cfg(feature = "macos_15_0")]
        FilterType::FullConfigWithMic => SCStreamConfiguration::new()
            .with_width(320)
            .with_height(240)
            .with_pixel_format(PixelFormat::BGRA)
            .with_captures_audio(true)
            .with_captures_microphone(true)
            .with_sample_rate(48000)
            .with_channel_count(2),
        _ => SCStreamConfiguration::new()
            .with_width(320)
            .with_height(240)
            .with_pixel_format(PixelFormat::BGRA)
            .with_captures_audio(true)
            .with_sample_rate(44100)
            .with_channel_count(2),
    };

    let mut stream = SCStream::new(&filter, &config);

    // Add output handlers for all types using SharedHandler wrapper
    stream.add_output_handler(SharedHandler(handler.clone()), SCStreamOutputType::Screen);
    stream.add_output_handler(SharedHandler(handler.clone()), SCStreamOutputType::Audio);

    #[cfg(feature = "macos_15_0")]
    if matches!(filter_type, FilterType::FullConfigWithMic) {
        stream.add_output_handler(
            SharedHandler(handler.clone()),
            SCStreamOutputType::Microphone,
        );
    }

    if let Err(e) = stream.start_capture() {
        eprintln!("    ⚠️  Failed to start: {e}");
        return;
    }

    thread::sleep(*duration);

    if let Err(e) = stream.stop_capture() {
        eprintln!("    ⚠️  Failed to stop: {e}");
    }

    handler.report();
    drop(stream);
    println!("    ✓ Done");
}

/// Test various configuration settings
fn test_configuration_variations() {
    let configs = [
        // Minimal config
        SCStreamConfiguration::new().with_width(64).with_height(64),
        // Different pixel formats
        SCStreamConfiguration::new()
            .with_width(128)
            .with_height(128)
            .with_pixel_format(PixelFormat::YCbCr_420v),
        // With source/destination rects
        SCStreamConfiguration::new()
            .with_width(256)
            .with_height(256)
            .with_source_rect(CGRect::new(0.0, 0.0, 100.0, 100.0))
            .with_destination_rect(CGRect::new(0.0, 0.0, 256.0, 256.0))
            .with_scales_to_fit(true),
        // High quality config
        SCStreamConfiguration::new()
            .with_width(1920)
            .with_height(1080)
            .with_shows_cursor(true)
            .with_queue_depth(8),
        // Audio only config
        SCStreamConfiguration::new()
            .with_width(100)
            .with_height(100)
            .with_captures_audio(true)
            .with_sample_rate(48000)
            .with_channel_count(2),
    ];

    println!("  Testing {} configuration variations...", configs.len());

    for (i, _config) in configs.iter().enumerate() {
        // Just create and drop the configs to test for leaks
        println!("    Config {}: ✓", i + 1);
    }
}

enum LeakResult {
    NoLeaks,
    AppleFrameworkLeaksOnly,
    LeaksDetected(String),
    Error(String),
}

fn check_for_leaks() -> LeakResult {
    let pid = std::process::id();

    let output = match Command::new("leaks")
        .args([pid.to_string(), "-c".to_string()])
        .output()
    {
        Ok(output) => output,
        Err(e) => return LeakResult::Error(format!("Failed to execute leaks command: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print raw output for debugging
    if !stdout.is_empty() {
        println!("leaks stdout:\n{stdout}");
    }
    if !stderr.is_empty() {
        println!("leaks stderr:\n{stderr}");
    }

    // Check for no leaks
    if stdout.contains("0 leaks for 0 total leaked bytes") {
        return LeakResult::NoLeaks;
    }

    // Check if all leaks are from Apple frameworks (not our code)
    let apple_framework_leaks = stdout.contains("CMCapture")
        || stdout.contains("FigRemoteOperationReceiver")
        || stdout.contains("SCStream(SCContentSharing)")
        || stdout.contains("CoreMedia")
        || stdout.contains("VideoToolbox");

    let our_code_leaks = stdout.contains("screencapturekit");

    if apple_framework_leaks && !our_code_leaks {
        return LeakResult::AppleFrameworkLeaksOnly;
    }

    LeakResult::LeaksDetected(stdout.to_string())
}
