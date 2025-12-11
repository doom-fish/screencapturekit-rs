//! # ScreenCaptureKit-rs
//!
//! Safe, idiomatic Rust bindings for Apple's [ScreenCaptureKit] framework.
//!
//! Capture screen content, windows, and applications with high performance on macOS 12.3+.
//!
//! [ScreenCaptureKit]: https://developer.apple.com/documentation/screencapturekit
//!
//! ## Features
//!
//! - **Screen and window capture** - Capture displays, windows, or specific applications
//! - **Audio capture** - System audio and microphone input (macOS 13.0+)
//! - **Real-time frame processing** - High-performance callbacks with custom dispatch queues
//! - **Async support** - Runtime-agnostic async API (Tokio, async-std, smol, etc.)
//! - **Zero-copy GPU access** - Direct [`IOSurface`] access for Metal/OpenGL integration
//! - **Screenshots** - Single-frame capture without streaming (macOS 14.0+)
//! - **Recording** - Direct-to-file video recording (macOS 15.0+)
//! - **Content Picker** - System UI for user content selection (macOS 14.0+)
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! screencapturekit = "1"
//! ```
//!
//! For async support:
//!
//! ```toml
//! [dependencies]
//! screencapturekit = { version = "1", features = ["async"] }
//! ```
//!
//! ## Quick Start
//!
//! ### 1. Request Permission
//!
//! Screen recording requires user permission. Add to your `Info.plist`:
//!
//! ```xml
//! <key>NSScreenCaptureUsageDescription</key>
//! <string>This app needs screen recording permission.</string>
//! ```
//!
//! ### 2. Implement a Frame Handler
//!
//! You can use either a struct or a closure:
//!
//! **Struct-based handler:**
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use std::sync::Arc;
//!
//! struct FrameHandler {
//!     count: Arc<AtomicUsize>,
//! }
//!
//! impl SCStreamOutputTrait for FrameHandler {
//!     fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
//!         match of_type {
//!             SCStreamOutputType::Screen => {
//!                 let n = self.count.fetch_add(1, Ordering::Relaxed);
//!                 if n % 60 == 0 {
//!                     println!("Frame {n}");
//!                 }
//!             }
//!             SCStreamOutputType::Audio => {
//!                 println!("Got audio samples!");
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! **Closure-based handler:**
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use std::sync::Arc;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let content = SCShareableContent::get()?;
//! # let display = content.displays().into_iter().next().unwrap();
//! # let filter = SCContentFilter::builder().display(&display).exclude_windows(&[]).build();
//! # let config = SCStreamConfiguration::new();
//! let frame_count = Arc::new(AtomicUsize::new(0));
//! let count_clone = frame_count.clone();
//!
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_output_handler(
//!     move |_sample: CMSampleBuffer, _of_type: SCStreamOutputType| {
//!         count_clone.fetch_add(1, Ordering::Relaxed);
//!     },
//!     SCStreamOutputType::Screen
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. Start Capturing
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # struct MyHandler;
//! # impl SCStreamOutputTrait for MyHandler {
//! #     fn did_output_sample_buffer(&self, _: CMSampleBuffer, _: SCStreamOutputType) {}
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Get available displays
//! let content = SCShareableContent::get()?;
//! let display = content.displays().into_iter().next().ok_or("No display")?;
//!
//! // Configure what to capture
//! let filter = SCContentFilter::builder()
//!     .display(&display)
//!     .exclude_windows(&[])
//!     .build();
//!
//! // Configure how to capture
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080)
//!     .with_pixel_format(PixelFormat::BGRA)
//!     .with_shows_cursor(true);
//!
//! // Create stream and add handler
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_output_handler(MyHandler, SCStreamOutputType::Screen);
//!
//! // Start capturing
//! stream.start_capture()?;
//!
//! // ... capture runs in background ...
//! std::thread::sleep(std::time::Duration::from_secs(5));
//!
//! stream.stop_capture()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration Options
//!
//! Use the builder pattern for fluent configuration:
//!
//! ```rust
//! use screencapturekit::prelude::*;
//!
//! // For 60 FPS, use CMTime to specify frame interval
//! let frame_interval = CMTime::new(1, 60); // 1/60th of a second
//!
//! let config = SCStreamConfiguration::new()
//!     // Video settings
//!     .with_width(1920)
//!     .with_height(1080)
//!     .with_pixel_format(PixelFormat::BGRA)
//!     .with_shows_cursor(true)
//!     .with_minimum_frame_interval(&frame_interval)
//!     
//!     // Audio settings
//!     .with_captures_audio(true)
//!     .with_sample_rate(48000)
//!     .with_channel_count(2);
//! ```
//!
//! ### Available Pixel Formats
//!
//! | Format | Description | Use Case |
//! |--------|-------------|----------|
//! | [`PixelFormat::BGRA`] | 32-bit BGRA | General purpose, easy to use |
//! | [`PixelFormat::l10r`] | 10-bit RGB | HDR content |
//! | [`PixelFormat::YCbCr_420v`] | YCbCr 4:2:0 | Video encoding (H.264/HEVC) |
//! | [`PixelFormat::YCbCr_420f`] | YCbCr 4:2:0 full range | Video encoding |
//!
//! ## Accessing Frame Data
//!
//! ### Pixel Data (CPU)
//!
//! Lock the pixel buffer for direct CPU access:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use screencapturekit::cv::{CVPixelBuffer, CVPixelBufferLockFlags, PixelBufferCursorExt};
//! use std::io::{Read, Seek, SeekFrom};
//!
//! # fn handle(sample: CMSampleBuffer) {
//! if let Some(buffer) = sample.image_buffer() {
//!     if let Ok(guard) = buffer.lock(CVPixelBufferLockFlags::READ_ONLY) {
//!         // Method 1: Direct slice access (fast)
//!         let pixels = guard.as_slice();
//!         let width = guard.width();
//!         let height = guard.height();
//!
//!         // Method 2: Use cursor for reading specific pixels
//!         let mut cursor = guard.cursor();
//!         
//!         // Read first pixel (BGRA)
//!         if let Ok(pixel) = cursor.read_pixel() {
//!             println!("First pixel: {:?}", pixel);
//!         }
//!
//!         // Seek to center pixel
//!         let center_x = width / 2;
//!         let center_y = height / 2;
//!         if cursor.seek_to_pixel(center_x, center_y, guard.bytes_per_row()).is_ok() {
//!             if let Ok(pixel) = cursor.read_pixel() {
//!                 println!("Center pixel: {:?}", pixel);
//!             }
//!         }
//!     }
//! }
//! # }
//! ```
//!
//! ### [`IOSurface`] (GPU)
//!
//! For Metal/OpenGL integration, access the underlying [`IOSurface`]:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use screencapturekit::cm::IOSurfaceLockOptions;
//! use screencapturekit::cv::PixelBufferCursorExt;
//!
//! # fn handle(sample: CMSampleBuffer) {
//! if let Some(buffer) = sample.image_buffer() {
//!     // Check if IOSurface-backed (usually true for ScreenCaptureKit)
//!     if buffer.is_backed_by_io_surface() {
//!         if let Some(surface) = buffer.io_surface() {
//!             println!("Dimensions: {}x{}", surface.width(), surface.height());
//!             println!("Pixel format: 0x{:08X}", surface.pixel_format());
//!             println!("Bytes per row: {}", surface.bytes_per_row());
//!             println!("In use: {}", surface.is_in_use());
//!
//!             // Lock for CPU access to IOSurface data
//!             if let Ok(guard) = surface.lock(IOSurfaceLockOptions::READ_ONLY) {
//!                 let mut cursor = guard.cursor();
//!                 if let Ok(pixel) = cursor.read_pixel() {
//!                     println!("First pixel: {:?}", pixel);
//!                 }
//!             }
//!         }
//!     }
//! }
//! # }
//! ```
//!
//! ## Audio + Video Capture
//!
//! Capture system audio alongside video:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use std::sync::Arc;
//!
//! struct AVHandler {
//!     video_count: Arc<AtomicUsize>,
//!     audio_count: Arc<AtomicUsize>,
//! }
//!
//! impl SCStreamOutputTrait for AVHandler {
//!     fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, of_type: SCStreamOutputType) {
//!         match of_type {
//!             SCStreamOutputType::Screen => {
//!                 self.video_count.fetch_add(1, Ordering::Relaxed);
//!             }
//!             SCStreamOutputType::Audio => {
//!                 self.audio_count.fetch_add(1, Ordering::Relaxed);
//!             }
//!             SCStreamOutputType::Microphone => {
//!                 // Requires macOS 15.0+ and .with_captures_microphone(true)
//!             }
//!         }
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = content.displays().into_iter().next().ok_or("No display")?;
//!
//! let filter = SCContentFilter::builder()
//!     .display(&display)
//!     .exclude_windows(&[])
//!     .build();
//!
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080)
//!     .with_captures_audio(true)  // Enable system audio
//!     .with_sample_rate(48000)    // 48kHz
//!     .with_channel_count(2);     // Stereo
//!
//! let handler = AVHandler {
//!     video_count: Arc::new(AtomicUsize::new(0)),
//!     audio_count: Arc::new(AtomicUsize::new(0)),
//! };
//!
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_output_handler(handler, SCStreamOutputType::Screen);
//! stream.start_capture()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Dynamic Stream Updates
//!
//! Update configuration or content filter while streaming:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let content = SCShareableContent::get()?;
//! # let display = content.displays().into_iter().next().unwrap();
//! # let filter = SCContentFilter::builder().display(&display).exclude_windows(&[]).build();
//! # let config = SCStreamConfiguration::new().with_width(640).with_height(480);
//! # struct MyHandler;
//! # impl SCStreamOutputTrait for MyHandler {
//! #     fn did_output_sample_buffer(&self, _: CMSampleBuffer, _: SCStreamOutputType) {}
//! # }
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_output_handler(MyHandler, SCStreamOutputType::Screen);
//! stream.start_capture()?;
//!
//! // Capture at initial resolution...
//! std::thread::sleep(std::time::Duration::from_secs(2));
//!
//! // Update to higher resolution while streaming
//! let new_config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//! stream.update_configuration(&new_config)?;
//!
//! // Switch to a different window
//! let windows = content.windows();
//! if let Some(window) = windows.iter().find(|w| w.is_on_screen()) {
//!     let window_filter = SCContentFilter::builder().window(window).build();
//!     stream.update_content_filter(&window_filter)?;
//! }
//!
//! stream.stop_capture()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling with Delegates
//!
//! Handle stream errors gracefully using delegates:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use screencapturekit::stream::ErrorHandler;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let content = SCShareableContent::get()?;
//! # let display = content.displays().into_iter().next().unwrap();
//! # let filter = SCContentFilter::builder().display(&display).exclude_windows(&[]).build();
//! # let config = SCStreamConfiguration::new();
//! // Create an error handler using a closure
//! let error_handler = ErrorHandler::new(|error| {
//!     eprintln!("Stream error: {error}");
//! });
//!
//! // Create stream with delegate
//! let mut stream = SCStream::new_with_delegate(&filter, &config, error_handler);
//! stream.add_output_handler(
//!     |_sample, _type| { /* process frames */ },
//!     SCStreamOutputType::Screen
//! );
//! stream.start_capture()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Dispatch Queues
//!
//! Control which thread/queue handles frame callbacks:
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let content = SCShareableContent::get()?;
//! # let display = content.displays().into_iter().next().unwrap();
//! # let filter = SCContentFilter::builder().display(&display).exclude_windows(&[]).build();
//! # let config = SCStreamConfiguration::new();
//! let mut stream = SCStream::new(&filter, &config);
//!
//! // Create a high-priority queue for frame processing
//! let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);
//!
//! stream.add_output_handler_with_queue(
//!     |_sample, _type| { /* called on custom queue */ },
//!     SCStreamOutputType::Screen,
//!     Some(&queue)
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ## Async API
//!
//! Enable the `async` feature for async/await support. The async API is
//! **executor-agnostic** and works with Tokio, async-std, smol, or any runtime:
//!
//! ```ignore
//! use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
//! use screencapturekit::prelude::*;
//!
//! async fn capture() -> Result<(), Box<dyn std::error::Error>> {
//!     // Get content asynchronously (true async - no blocking)
//!     let content = AsyncSCShareableContent::get().await?;
//!     let display = &content.displays()[0];
//!     
//!     let filter = SCContentFilter::builder()
//!         .display(display)
//!         .exclude_windows(&[])
//!         .build();
//!     
//!     let config = SCStreamConfiguration::new()
//!         .with_width(1920)
//!         .with_height(1080);
//!     
//!     // Create async stream with 30-frame buffer
//!     let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
//!     stream.start_capture()?;
//!     
//!     // Async iteration over frames
//!     let mut count = 0;
//!     while count < 100 {
//!         if let Some(_frame) = stream.next().await {
//!             count += 1;
//!         }
//!     }
//!     
//!     stream.stop_capture()?;
//!     Ok(())
//! }
//!
//! // Concurrent async operations
//! async fn concurrent_queries() -> Result<(), Box<dyn std::error::Error>> {
//!     let (result1, result2) = tokio::join!(
//!         AsyncSCShareableContent::get(),
//!         AsyncSCShareableContent::with_options()
//!             .on_screen_windows_only(true)
//!             .get(),
//!     );
//!     Ok(())
//! }
//! ```
//!
//! ## Screenshots (macOS 14.0+)
//!
//! Take single screenshots without setting up a stream:
//!
//! ```ignore
//! use screencapturekit::prelude::*;
//! use screencapturekit::screenshot_manager::SCScreenshotManager;
//!
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//!
//! let filter = SCContentFilter::builder()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//!
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! // Capture screenshot as CGImage
//! let image = SCScreenshotManager::capture_image(&filter, &config)?;
//! println!("Screenshot: {}x{}", image.width(), image.height());
//!
//! // Or capture as CMSampleBuffer for more control
//! let sample_buffer = SCScreenshotManager::capture_sample_buffer(&filter, &config)?;
//! ```
//!
//! ## Recording (macOS 15.0+)
//!
//! Record directly to a video file:
//!
//! ```ignore
//! use screencapturekit::prelude::*;
//! use screencapturekit::recording_output::{
//!     SCRecordingOutput, SCRecordingOutputConfiguration,
//!     SCRecordingOutputCodec, SCRecordingOutputFileType
//! };
//! use std::path::PathBuf;
//!
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//!
//! let filter = SCContentFilter::builder()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//!
//! let stream_config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! // Configure recording output
//! let output_path = PathBuf::from("/tmp/screen_recording.mp4");
//! let recording_config = SCRecordingOutputConfiguration::new()
//!     .with_output_url(&output_path)
//!     .with_video_codec(SCRecordingOutputCodec::H264)
//!     .with_output_file_type(SCRecordingOutputFileType::MP4);
//!
//! let recording_output = SCRecordingOutput::new(&recording_config)
//!     .ok_or("Failed to create recording output")?;
//!
//! // Start stream and add recording
//! let stream = SCStream::new(&filter, &stream_config);
//! stream.add_recording_output(&recording_output)?;
//! stream.start_capture()?;
//!
//! // Record for 10 seconds
//! std::thread::sleep(std::time::Duration::from_secs(10));
//!
//! // Check recording stats
//! let duration = recording_output.recorded_duration();
//! let file_size = recording_output.recorded_file_size();
//! println!("Recorded {}/{} seconds, {} bytes", duration.value, duration.timescale, file_size);
//!
//! stream.remove_recording_output(&recording_output)?;
//! stream.stop_capture()?;
//! ```
//!
//! ## Module Organization
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`stream`] | Stream configuration and management ([`SCStream`], [`SCContentFilter`]) |
//! | [`shareable_content`] | Display, window, and application enumeration |
//! | [`cm`] | Core Media types ([`CMSampleBuffer`], [`CMTime`], [`IOSurface`]) |
//! | [`cv`] | Core Video types ([`CVPixelBuffer`], lock guards) |
//! | [`cg`] | Core Graphics types ([`CGRect`], [`CGSize`]) |
//! | [`metal`] | Metal texture helpers for zero-copy GPU rendering |
//! | [`dispatch_queue`] | Custom dispatch queues for callbacks |
//! | [`error`] | Error types and result aliases |
//! | `async_api` | Async wrappers (requires `async` feature) |
//! | [`screenshot_manager`] | Single-frame capture (macOS 14.0+) |
//! | `recording_output` | Direct file recording (macOS 15.0+) |
//!
//! [`SCStream`]: stream::sc_stream::SCStream
//! [`SCContentFilter`]: stream::content_filter::SCContentFilter
//! [`CMSampleBuffer`]: cm::CMSampleBuffer
//! [`CMTime`]: cm::CMTime
//! [`IOSurface`]: cm::IOSurface
//! [`CVPixelBuffer`]: cv::CVPixelBuffer
//! [`CGRect`]: cg::CGRect
//! [`CGSize`]: cg::CGSize
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async` | Runtime-agnostic async API |
//! | `macos_13_0` | macOS 13.0+ APIs (audio capture, synchronization clock) |
//! | `macos_14_0` | macOS 14.0+ APIs (screenshots, content picker) |
//! | `macos_14_2` | macOS 14.2+ APIs (menu bar, child windows, presenter overlay) |
//! | `macos_14_4` | macOS 14.4+ APIs (current process shareable content) |
//! | `macos_15_0` | macOS 15.0+ APIs (recording output, HDR, microphone) |
//! | `macos_15_2` | macOS 15.2+ APIs (screenshot in rect, stream delegates) |
//! | `macos_26_0` | macOS 26.0+ APIs (advanced screenshot config, HDR output) |
//!
//! Features are cumulative: enabling `macos_15_0` also enables all earlier versions.
//!
//! ## Platform Requirements
//!
//! - **macOS 12.3+** (Monterey) - Base `ScreenCaptureKit` support
//! - **Screen Recording Permission** - Must be granted by user in System Preferences
//! - **Hardened Runtime** - Required for notarized apps
//!
//! ## Examples
//!
//! See the [examples directory](https://github.com/doom-fish/screencapturekit-rs/tree/main/examples):
//!
//! | Example | Description |
//! |---------|-------------|
//! | `01_basic_capture` | Simplest screen capture |
//! | `02_window_capture` | Capture specific windows |
//! | `03_audio_capture` | Audio + video capture |
//! | `04_pixel_access` | Read pixel data with cursor API |
//! | `05_screenshot` | Single screenshot (macOS 14.0+) |
//! | `06_iosurface` | Zero-copy GPU buffer access |
//! | `07_list_content` | List available displays, windows, apps |
//! | `08_async` | Async/await API with any runtime |
//! | `09_closure_handlers` | Closure-based handlers |
//! | `10_recording_output` | Direct video recording (macOS 15.0+) |
//! | `11_content_picker` | System content picker UI (macOS 14.0+) |
//! | `12_stream_updates` | Dynamic config/filter updates |
//! | `13_advanced_config` | HDR, presets, microphone (macOS 15.0+) |
//! | `14_app_capture` | Application-based filtering |
//! | `15_memory_leak_check` | Memory leak detection |
//! | `16_full_metal_app` | Full Metal GUI application |
//! | `17_metal_textures` | Metal texture creation from `IOSurface` |
//!
//! ## Common Patterns
//!
//! ### Capture Window by Title
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let windows = content.windows();
//! let window = windows
//!     .iter()
//!     .find(|w| w.title().is_some_and(|t| t.contains("Safari")))
//!     .ok_or("Window not found")?;
//!
//! let filter = SCContentFilter::builder()
//!     .window(window)
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ### Capture Specific Application
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = content.displays().into_iter().next().ok_or("No display")?;
//!
//! // Find app by bundle ID
//! let apps = content.applications();
//! let safari = apps
//!     .iter()
//!     .find(|app| app.bundle_identifier() == "com.apple.Safari")
//!     .ok_or("Safari not found")?;
//!
//! // Capture only windows from this app
//! let filter = SCContentFilter::builder()
//!     .display(&display)
//!     .include_applications(&[safari])
//!     .exclude_windows(&[])  // Don't exclude any windows
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ### Exclude Your Own App's Windows
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = content.displays().into_iter().next().ok_or("No display")?;
//!
//! // Find our app's windows
//! let windows = content.windows();
//! let my_windows: Vec<&SCWindow> = windows
//!     .iter()
//!     .filter(|w| w.owning_application()
//!         .map(|app| app.bundle_identifier() == "com.mycompany.myapp")
//!         .unwrap_or(false))
//!     .collect();
//!
//! // Capture everything except our windows
//! let filter = SCContentFilter::builder()
//!     .display(&display)
//!     .exclude_windows(&my_windows)
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ### List All Available Content
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//!
//! println!("=== Displays ===");
//! for display in content.displays() {
//!     println!("  Display {}: {}x{}", display.display_id(), display.width(), display.height());
//! }
//!
//! println!("\n=== Windows ===");
//! for window in content.windows().iter().filter(|w| w.is_on_screen()) {
//!     println!("  [{}] {} - {}",
//!         window.window_id(),
//!         window.owning_application()
//!             .map(|app| app.application_name())
//!             .unwrap_or_default(),
//!         window.title().unwrap_or_default()
//!     );
//! }
//!
//! println!("\n=== Applications ===");
//! for app in content.applications() {
//!     println!("  {} ({})", app.application_name(), app.bundle_identifier());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [`PixelFormat::BGRA`]: stream::configuration::PixelFormat::BGRA
//! [`PixelFormat::l10r`]: stream::configuration::PixelFormat::l10r
//! [`PixelFormat::YCbCr_420v`]: stream::configuration::PixelFormat::YCbCr_420v
//! [`PixelFormat::YCbCr_420f`]: stream::configuration::PixelFormat::YCbCr_420f

#![doc(html_root_url = "https://docs.rs/screencapturekit")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

pub mod audio_devices;
pub mod cg;
pub mod cm;
#[cfg(feature = "macos_14_0")]
pub mod content_sharing_picker;
pub mod cv;
pub mod dispatch_queue;
pub mod error;
pub mod ffi;
pub mod metal;
#[cfg(feature = "macos_15_0")]
pub mod recording_output;
pub mod screenshot_manager;
pub mod shareable_content;
pub mod stream;
pub mod utils;

#[cfg(feature = "async")]
pub mod async_api;

// Re-export commonly used types
pub use cm::{
    codec_types, media_types, AudioBuffer, AudioBufferList, CMFormatDescription, CMSampleBuffer,
    CMSampleTimingInfo, CMTime, IOSurface, SCFrameStatus,
};
pub use cv::{CVPixelBuffer, CVPixelBufferPool};
pub use utils::four_char_code::FourCharCode;

/// Prelude module for convenient imports
///
/// Import everything you need with:
/// ```rust
/// use screencapturekit::prelude::*;
/// ```
pub mod prelude {
    pub use crate::audio_devices::AudioInputDevice;
    pub use crate::cg::{CGPoint, CGRect, CGSize};
    pub use crate::cm::{CMSampleBuffer, CMTime};
    pub use crate::dispatch_queue::{DispatchQoS, DispatchQueue};
    pub use crate::error::{SCError, SCResult};
    pub use crate::shareable_content::{
        SCDisplay, SCRunningApplication, SCShareableContent, SCWindow,
    };
    pub use crate::stream::{
        configuration::{PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        sc_stream::SCStream,
        ErrorHandler,
    };
}
