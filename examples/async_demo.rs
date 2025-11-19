//! Comprehensive async API example demonstrating executor-agnostic usage
//!
//! This example works with any async runtime: Tokio, async-std, smol, etc.
//!
//! Run with:
//! ```bash
//! cargo run --example async_demo --features "async,macos_14_0"
//! ```

#[cfg(feature = "async")]
use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream, utils};

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Complete Executor-Agnostic Async API Demo\n");
    println!("This example uses Tokio, but the API works with ANY async runtime!");
    println!("(async-std, smol, futures, etc.)\n");
    println!("═══════════════════════════════════════════════════════════\n");

    // 1. Get shareable content asynchronously
    println!("📡 1. Fetching shareable content asynchronously...");
    let content = AsyncSCShareableContent::get().await?;

    let displays = content.displays();
    let windows = content.windows();
    let apps = content.applications();

    println!("   ✅ Results:");
    println!("      Displays: {}", displays.len());
    println!("      Windows: {}", windows.len());
    println!("      Applications: {}", apps.len());

    // 2. Show display details
    println!("\n📺 2. Display Information:");
    for (i, display) in displays.iter().enumerate().take(3) {
        println!(
            "      {}. {} x {} (ID: {})",
            i + 1,
            display.width(),
            display.height(),
            display.display_id()
        );
    }

    // 3. Concurrent operations demo
    println!("\n⚡ 3. Running 3 concurrent async operations...");
    let start = std::time::Instant::now();
    
    let (result1, result2, result3): (
        Result<_, _>,
        Result<_, _>,
        Result<_, _>
    ) = tokio::join!(
        AsyncSCShareableContent::get(),
        utils::get_on_screen_windows(),
        utils::get_main_display(),
    );

    let elapsed = start.elapsed();
    println!("   ✅ All 3 operations completed in {:?}", elapsed);
    println!("      (Each spawned its own thread!)");

    if let Ok(windows) = result2 {
        println!("      Found {} on-screen windows", windows.len());
    }

    if let Ok(Some(display)) = result3 {
        println!("      Main display: {}x{}", display.width(), display.height());
    }

    // 4. Utility functions demo
    println!("\n🔧 4. Testing utility functions...");

    if !displays.is_empty() {
        let filter = utils::create_display_filter(&displays[0]).await;
        println!("   ✅ Created display filter");

        let config = utils::create_stream_config(1920, 1080).await?;
        println!("   ✅ Created stream configuration");

        // 5. Create async stream wrapper
        println!("\n🎥 5. Creating async stream wrapper...");
        let stream = AsyncSCStream::new(&filter, &config);
        println!("   ✅ AsyncSCStream created (no thread spawned for creation)");
        println!("      (start/stop would be async operations)");
    }

    // 6. Window search demo
    println!("\n🔍 6. Searching for windows...");
    let window_result = utils::find_window_by_title("Terminal".to_string()).await?;
    match window_result {
        Some(window) => {
            println!("   ✅ Found window: {:?}", window.title());
        }
        None => {
            println!("   ℹ️  Terminal window not found");
        }
    }

    // 7. Application search demo
    println!("\n📱 7. Searching for applications...");
    let app_result = utils::get_application_content("Finder".to_string()).await?;
    match app_result {
        Some(app) => {
            println!("   ✅ Found application: {}", app.application_name());
            println!("      PID: {}", app.process_id());
        }
        None => {
            println!("   ℹ️  Finder not running");
        }
    }

    // 8. Options builder demo
    println!("\n⚙️  8. Using options builder...");
    let filtered_content = AsyncSCShareableContent::with_options()
        .on_screen_windows_only(true)
        .exclude_desktop_windows(true)
        .get_async()
        .await?;

    println!(
        "   ✅ Filtered content: {} on-screen windows",
        filtered_content.windows().len()
    );

    #[cfg(feature = "macos_14_0")]
    {
        use screencapturekit::async_api::AsyncSCScreenshotManager;
        use screencapturekit::stream::configuration::SCStreamConfiguration;
        use screencapturekit::stream::content_filter::SCContentFilter;

        if !displays.is_empty() {
            println!("\n📸 9. Capturing screenshot asynchronously...");

            let display = &displays[0];
            #[allow(deprecated)]
            let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);

            let config = SCStreamConfiguration::build()
                .set_width(640)?
                .set_height(480)?;

            let start = std::time::Instant::now();
            let image = AsyncSCScreenshotManager::capture_image(&filter, &config).await?;
            let elapsed = start.elapsed();

            println!(
                "   ✅ Screenshot captured: {}x{} in {:?}",
                image.width(),
                image.height(),
                elapsed
            );
            println!("      (Spawned 1 thread for the operation)");
        }
    }

    // Summary
    println!("\n═══════════════════════════════════════════════════════════");
    println!("✨ Demo complete!\n");
    println!("💡 Key Points:");
    println!("   • Each async operation spawns exactly 1 thread");
    println!("   • thread::spawn happens in BlockingFuture::new()");
    println!("   • Location: src/async_api.rs:85");
    println!("   • Threads are cleaned up automatically");
    println!("   • Works with ANY async runtime!\n");
    
    println!("🌍 Supported Runtimes:");
    println!("   • Tokio (what we're using now)");
    println!("   • async-std");
    println!("   • smol");
    println!("   • futures::executor");
    println!("   • Any other runtime that implements standard Rust async\n");

    println!("📚 See ASYNC_API_IMPLEMENTATION.md for complete details!");

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature.");
    eprintln!("Run with: cargo run --example async_demo --features async");
    std::process::exit(1);
}
