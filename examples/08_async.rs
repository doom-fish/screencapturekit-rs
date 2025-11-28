//! Async API Examples
//!
//! Demonstrates the async/await API (requires "async" feature).
//! The async API is **executor-agnostic** and works with any runtime:
//! Tokio, async-std, smol, or even a custom executor.
//!
//! Run with:
//! ```bash
//! cargo run --example 09_async --features async
//! ```

#[cfg(not(feature = "async"))]
fn main() {
    println!("⚠️  This example requires the 'async' feature");
    println!("    Run with: cargo run --example 09_async --features async");
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

    println!("\n═══════════════════════════════════════════════════════════");
    println!("✨ All async examples complete!");
    println!("\n💡 Key Points:");
    println!("   • True async with callback-based Swift FFI");
    println!("   • No blocking - yields to executor while waiting");
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
        AsyncSCShareableContent::with_options()
            .on_screen_windows_only(true)
            .get_async(),
        AsyncSCShareableContent::with_options()
            .exclude_desktop_windows(true)
            .get_async(),
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
    use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
    use screencapturekit::stream::configuration::SCStreamConfiguration;
    use screencapturekit::stream::content_filter::SCContentFilter;
    use screencapturekit::stream::output_type::SCStreamOutputType;

    println!("🎥 3. Async Stream Frame Iteration");
    println!("   ─────────────────────────────────");

    let content = AsyncSCShareableContent::get().await?;
    let displays = content.displays();

    if let Some(display) = displays.first() {
        let filter = SCContentFilter::builder()
            .display(display)
            .exclude_windows(&[])
            .build();

        let config = SCStreamConfiguration::default()
            .set_width(1920)
            .set_height(1080);

        // Create async stream with 30-frame buffer
        let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
        stream.start_capture()?;

        println!("   Capturing frames asynchronously...");

        // Capture 10 frames using async iteration
        let mut count = 0;
        while count < 10 {
            if let Some(_frame) = stream.next().await {
                count += 1;
                if count % 5 == 0 {
                    println!("      Frame {count}");
                }
            }
        }

        stream.stop_capture()?;
        println!("   ✅ Captured {count} frames");
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
