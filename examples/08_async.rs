//! Async API Examples
//!
//! Demonstrates the async/await API (requires "async" feature).
//! The async API is **executor-agnostic** and works with any runtime:
//! Tokio, async-std, smol, or even a custom executor.
//!
//! Run with:
//! ```bash
//! cargo run --example 08_async --features async
//! cargo run --example 08_async --features "async,macos_14_0"  # For picker example
//! ```

#[cfg(not(feature = "async"))]
fn main() {
    println!("⚠️  This example requires the 'async' feature");
    println!("    Run with: cargo run --example 08_async --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Async API Examples\n");
    println!("This API is executor-agnostic - works with Tokio, async-std, smol, etc.\n");
    println!("═══════════════════════════════════════════════════════════\n");

    basic_async_capture().await?;
    println!();
    concurrent_operations().await?;
    println!();
    async_stream_iteration().await?;
    println!();
    runtime_agnostic_demo().await?;
    println!();
    av_capture().await?;

    #[cfg(feature = "macos_14_0")]
    {
        println!();
        async_content_picker().await?;
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("✨ All async examples complete!");
    println!("\n💡 Key Points:");
    println!("   • True async with callback-based Swift FFI");
    println!("   • No blocking - yields to executor while waiting");
    println!("   • start/stop/update are real futures (waker-based, never block)");
    println!("   • One stream can carry audio + video (next_typed)");
    println!("   • Works with ANY async runtime");

    Ok(())
}

// ============================================================================
// Example 1: Basic Async Capture
// ============================================================================

#[cfg(feature = "async")]
async fn basic_async_capture() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::AsyncSCShareableContent;

    println!("📡 1. Basic Async Content Retrieval");
    println!("   ─────────────────────────────────");

    // Get content asynchronously (true async - no blocking)
    let content = AsyncSCShareableContent::get().await?;

    let displays = content.displays();
    let windows = content.windows();
    let apps = content.applications();

    println!("   ✅ Found:");
    println!("      • {} displays", displays.len());
    println!("      • {} windows", windows.len());
    println!("      • {} applications", apps.len());

    // Show display details
    for display in displays.iter().take(2) {
        println!(
            "      Display {}: {}x{}",
            display.display_id(),
            display.width(),
            display.height()
        );
    }

    Ok(())
}

// ============================================================================
// Example 2: Concurrent Operations
// ============================================================================

#[cfg(feature = "async")]
async fn concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::AsyncSCShareableContent;

    println!("⚡ 2. Concurrent Async Operations");
    println!("   ─────────────────────────────────");

    let start = std::time::Instant::now();

    // Run 3 async operations concurrently
    let (result1, result2, result3) = tokio::join!(
        AsyncSCShareableContent::get(),
        AsyncSCShareableContent::create()
            .with_on_screen_windows_only(true)
            .get(),
        AsyncSCShareableContent::create()
            .with_exclude_desktop_windows(true)
            .get(),
    );

    let elapsed = start.elapsed();

    println!("   ✅ 3 concurrent operations completed in {elapsed:?}");

    if let Ok(content) = result1 {
        println!("      • All content: {} windows", content.windows().len());
    }
    if let Ok(content) = result2 {
        println!(
            "      • On-screen only: {} windows",
            content.windows().len()
        );
    }
    if let Ok(content) = result3 {
        println!(
            "      • Excluding desktop: {} windows",
            content.windows().len()
        );
    }

    Ok(())
}

// ============================================================================
// Example 3: Async Stream with Frame Iteration
// ============================================================================

#[cfg(feature = "async")]
async fn async_stream_iteration() -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::StreamExt;
    use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;
    use screencapturekit::stream::output_type::SCStreamOutputType;

    println!("🎥 3. Async Stream Frame Iteration");
    println!("   ─────────────────────────────────");

    let content = AsyncSCShareableContent::get().await?;
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&[])
            .build();

        let config = SCStreamConfiguration::new()
            .with_width(1920)
            .with_height(1080);

        // Create async stream with 30-frame buffer
        let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
        // start/stop are truly async: awaiting parks the task via its Waker and
        // resumes from the Swift completion callback — the executor is never blocked.
        stream.start_capture().await?;

        println!("   Capturing frames asynchronously...");

        // Style A — manual `next().await` loop:
        let mut count = 0;
        while count < 10 {
            if let Some(_frame) = stream.next().await {
                count += 1;
                if count % 5 == 0 {
                    println!("      Frame {count}");
                }
            }
        }

        // Style B — `frames()` is a `futures::Stream`, so the whole StreamExt
        // combinator ecosystem (take / map / collect / for_each / …) just works:
        let batch: Vec<()> = stream.frames().map(|_frame| ()).take(5).collect().await;
        println!("      StreamExt collected {} more frames", batch.len());

        stream.stop_capture().await?;
        println!("   ✅ Captured {} frames", count + batch.len());
    } else {
        println!("   ⚠️  No displays available");
    }

    Ok(())
}

// ============================================================================
// Example 4: Runtime-Agnostic Demo
// ============================================================================

#[cfg(feature = "async")]
async fn runtime_agnostic_demo() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::AsyncSCShareableContent;

    println!("🌍 4. Runtime-Agnostic Demonstration");
    println!("   ─────────────────────────────────");
    println!("   This same code works with ANY async runtime:");
    println!("      • Tokio ✅");
    println!("      • async-std ✅");
    println!("      • smol ✅");
    println!("      • futures executor ✅");
    println!("      • Custom executors ✅");

    // The async API uses only std types internally:
    // - std::future::Future
    // - std::task::{Poll, Waker, Context}
    // - std::sync::{Arc, Mutex}
    // - Callback-based Swift FFI

    let content = AsyncSCShareableContent::get().await?;
    println!(
        "\n   ✅ Retrieved {} displays using executor-agnostic async",
        content.displays().len()
    );

    Ok(())
}

// ============================================================================
// Example 5: Audio + Video from a single async stream (multi-output)
// ============================================================================

#[cfg(feature = "async")]
async fn av_capture() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;
    use screencapturekit::stream::output_type::SCStreamOutputType;
    use std::time::Duration;

    println!("🎬 5. Audio + Video From One Async Stream");
    println!("   ─────────────────────────────────────");

    let content = AsyncSCShareableContent::get().await?;
    let displays = content.displays();
    let Some(display) = displays.first() else {
        println!("   ⚠️  No displays available");
        return Ok(());
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    // Enable system-audio capture in the configuration, create the stream for
    // video, then register audio as a second output type so ONE stream carries
    // both. `next_typed()` yields each sample together with its output type.
    let config = SCStreamConfiguration::new()
        .with_width(1280)
        .with_height(720)
        .with_captures_audio(true);

    let mut stream = AsyncSCStream::new(&filter, &config, 32, SCStreamOutputType::Screen);
    if stream.add_output_type(SCStreamOutputType::Audio) {
        println!("   ✅ Registered audio + video on a single stream");
    }

    stream.start_capture().await?;
    println!("   Capturing interleaved audio/video samples...");

    let mut video = 0u32;
    let mut audio = 0u32;
    // Pull samples, dispatching on their output type. A per-sample timeout keeps
    // the demo from blocking if the screen is static / system audio is silent.
    for _ in 0..60 {
        match tokio::time::timeout(Duration::from_millis(500), stream.next_typed()).await {
            Ok(Some((_sample, SCStreamOutputType::Screen))) => video += 1,
            Ok(Some((_sample, SCStreamOutputType::Audio))) => audio += 1,
            Ok(Some((_sample, SCStreamOutputType::Microphone))) => {}
            Ok(None) => break, // stream closed
            Err(_) => {}       // no sample this round; keep polling
        }
        if video >= 10 && audio >= 1 {
            break;
        }
    }

    stream.stop_capture().await?;
    println!("   ✅ Received {video} video + {audio} audio samples (one stream)");

    // Surface an error stop, if any (next_typed() would have returned None).
    if let Some(err) = stream.take_error() {
        eprintln!("   ⚠️  Stream stopped with error: {err}");
    }

    Ok(())
}

// ============================================================================
// Example 6: Async Content Picker (macOS 14.0+)
// ============================================================================

#[cfg(all(feature = "async", feature = "macos_14_0"))]
async fn async_content_picker() -> Result<(), Box<dyn std::error::Error>> {
    use screencapturekit::async_api::AsyncSCContentSharingPicker;
    use screencapturekit::content_sharing_picker::{
        SCContentSharingPickerConfiguration, SCPickerOutcome,
    };

    println!("🎯 6. Async Content Picker (macOS 14.0+)");
    println!("   ─────────────────────────────────────");
    println!("   The picker UI will appear - select content or cancel.");
    println!("   This is truly async - the executor is NOT blocked while waiting.\n");

    let config = SCContentSharingPickerConfiguration::new();

    // Async picker - doesn't block the executor thread
    match AsyncSCContentSharingPicker::show(&config).await {
        SCPickerOutcome::Picked(result) => {
            let (width, height) = result.pixel_size();
            let scale = result.scale();

            println!("   ✅ User selected content:");
            println!("      • Dimensions: {width}x{height} pixels");
            println!("      • Scale factor: {scale}");

            // Show what was picked
            let windows = result.windows();
            let displays = result.displays();

            if !displays.is_empty() {
                println!("      • Displays: {}", displays.len());
                for display in displays.iter().take(2) {
                    println!(
                        "        - Display {}: {}x{}",
                        display.display_id(),
                        display.width(),
                        display.height()
                    );
                }
            }

            if !windows.is_empty() {
                println!("      • Windows: {}", windows.len());
                for window in windows.iter().take(3) {
                    println!(
                        "        - {} (ID: {})",
                        window.title().unwrap_or_else(|| "<untitled>".to_string()),
                        window.window_id()
                    );
                }
            }

            // The filter is ready to use with SCStream
            let _filter = result.filter();
            println!("      • Filter ready for streaming ✓");
        }
        SCPickerOutcome::Cancelled => {
            println!("   ℹ️  User cancelled the picker");
        }
        SCPickerOutcome::Error(e) => {
            println!("   ❌ Picker error: {e}");
        }
    }

    Ok(())
}
