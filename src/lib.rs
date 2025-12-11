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
//! - **Recording** - Direct-to-file recording (macOS 15.0+)
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
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! struct MyHandler;
//!
//! impl SCStreamOutputTrait for MyHandler {
//!     fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
//!         match of_type {
//!             SCStreamOutputType::Screen => {
//!                 println!("Got video frame!");
//!                 // Access pixel data, IOSurface, etc.
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
//! let display = &content.displays()[0];
//!
//! // Configure what to capture
//! let filter = SCContentFilter::builder()
//!     .display(display)
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
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//! use screencapturekit::cv::{CVPixelBuffer, CVPixelBufferLockFlags};
//!
//! # fn handle(sample: CMSampleBuffer) {
//! if let Some(buffer) = sample.image_buffer() {
//!     // Lock for CPU access
//!     if let Ok(guard) = buffer.lock(CVPixelBufferLockFlags::READ_ONLY) {
//!         let pixels = guard.as_slice();
//!         let width = guard.width();
//!         let height = guard.height();
//!         // Process pixels...
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
//!
//! # fn handle(sample: CMSampleBuffer) {
//! if let Some(buffer) = sample.image_buffer() {
//!     if let Some(surface) = buffer.io_surface() {
//!         let width = surface.width();
//!         let height = surface.height();
//!         // Create Metal texture from IOSurface...
//!     }
//! }
//! # }
//! ```
//!
//! ## Async API
//!
//! Enable the `async` feature for async/await support:
//!
//! ```ignore
//! use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
//! use screencapturekit::prelude::*;
//!
//! async fn capture() -> Result<(), Box<dyn std::error::Error>> {
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
//!     let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
//!     stream.start_capture()?;
//!     
//!     // Async iteration over frames
//!     while let Some(frame) = stream.next().await {
//!         println!("Got frame!");
//!     }
//!     
//!     Ok(())
//! }
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
//! | [`async_api`] | Async wrappers (requires `async` feature) |
//! | [`screenshot_manager`] | Single-frame capture (macOS 14.0+) |
//! | [`recording_output`] | Direct file recording (macOS 15.0+) |
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
//! See the [examples directory](https://github.com/doom-fish/screencapturekit-rs/tree/main/examples) for complete working examples:
//!
//! - `01_basic_capture` - Simplest screen capture
//! - `02_window_capture` - Capture specific windows
//! - `03_audio_capture` - Audio + video capture
//! - `04_pixel_access` - Read pixel data
//! - `05_screenshot` - Single screenshot (macOS 14.0+)
//! - `06_iosurface` - Zero-copy GPU buffers
//! - `07_list_content` - List available displays, windows, apps
//! - `08_async` - Async/await API
//! - `09_closure_handlers` - Closure-based handlers
//! - `10_recording_output` - Direct video recording (macOS 15.0+)
//! - `11_content_picker` - System content picker UI (macOS 14.0+)
//! - `12_stream_updates` - Dynamic config/filter updates
//! - `13_advanced_config` - HDR, presets, microphone (macOS 15.0+)
//! - `14_app_capture` - Application-based filtering
//! - `15_memory_leak_check` - Memory leak detection
//! - `16_full_metal_app` - Full Metal GUI application (macOS 14.0+)
//! - `17_metal_textures` - Metal texture creation from IOSurface
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
//!     .find(|w| w.title().is_some_and(|t| t == "Safari"))
//!     .ok_or("Window not found")?;
//!
//! let filter = SCContentFilter::builder()
//!     .window(window)
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ### Exclude Specific Windows
//!
//! ```rust,no_run
//! use screencapturekit::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//!
//! // Exclude our own app's windows
//! let windows = content.windows();
//! let my_windows: Vec<&SCWindow> = windows
//!     .iter()
//!     .filter(|w| w.owning_application()
//!         .map(|app| app.bundle_identifier() == "com.myapp")
//!         .unwrap_or(false))
//!     .collect();
//!
//! let filter = SCContentFilter::builder()
//!     .display(display)
//!     .exclude_windows(&my_windows)
//!     .build();
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
