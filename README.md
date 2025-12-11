# ScreenCaptureKit-rs

[![Crates.io](https://img.shields.io/crates/v/screencapturekit.svg)](https://crates.io/crates/screencapturekit)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://doom-fish.github.io/screencapturekit-rs/screencapturekit/)
[![License](https://img.shields.io/crates/l/screencapturekit.svg)](https://github.com/doom-fish/screencapturekit-rs#license)
[![Build Status](https://img.shields.io/github/actions/workflow/status/doom-fish/screencapturekit-rs/ci.yml?branch=main)](https://github.com/doom-fish/screencapturekit-rs/actions)

> **💼 Looking for a hosted desktop recording API?**  
> Check out [Recall.ai](https://www.recall.ai/product/desktop-recording-sdk?utm_source=github&utm_medium=sponsorship&utm_campaign=screencapturekit-rs) - an API for recording Zoom, Google Meet, Microsoft Teams, in-person meetings, and more.

https://github.com/doom-fish/screencapturekit-rs/raw/main/docs/assets/demo.mp4

Safe, idiomatic Rust bindings for Apple's [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit) framework.

Capture screen content, windows, and applications with high performance and low overhead on macOS 12.3+.

## 📑 Table of Contents

- [Features](#-features)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Key Concepts](#-key-concepts)
- [Feature Flags](#-feature-flags)
- [API Overview](#-api-overview)
- [Examples](#-examples)
- [Testing](#-testing)
- [Architecture](#-architecture)
- [Troubleshooting](#-troubleshooting)
- [Platform Requirements](#-platform-requirements)
- [Contributing](#-contributing)
- [License](#-license)

## ✨ Features

- 🎥 **Screen & Window Capture** - Capture displays, windows, or specific applications
- 🔊 **Audio Capture** - Capture system audio and microphone input
- ⚡ **Real-time Processing** - High-performance frame callbacks with custom dispatch queues
- 🏗️ **Builder Pattern API** - Clean, type-safe configuration with `::builder()`
- 🔄 **Async Support** - Runtime-agnostic async API (works with Tokio, async-std, smol, etc.)
- 🎨 **IOSurface Access** - Zero-copy GPU texture access for Metal/OpenGL
- 🛡️ **Memory Safe** - Proper reference counting and leak-free by design
- 📦 **Zero Dependencies** - No runtime dependencies (only dev dependencies for examples)

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
screencapturekit = "1"
```

For async support:

```toml
[dependencies]
screencapturekit = { version = "1", features = ["async"] }
```

For latest macOS features:

```toml
[dependencies]
screencapturekit = { version = "1", features = ["macos_26_0"] }
```

## 🚀 Quick Start

### Basic Screen Capture

```rust
use screencapturekit::prelude::*;

struct Handler;

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _type: SCStreamOutputType) {
        println!("📹 Received frame at {:?}", sample.presentation_timestamp());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get available displays
    let content = SCShareableContent::get()?;
    let display = &content.displays()[0];
    
    // Configure capture
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();
    
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);
    
    // Start streaming
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(Handler, SCStreamOutputType::Screen);
    stream.start_capture()?;
    
    // Capture runs in background...
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    stream.stop_capture()?;
    Ok(())
}
```

### Async Capture

```rust
use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
use screencapturekit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get content asynchronously
    let content = AsyncSCShareableContent::get().await?;
    let display = &content.displays()[0];
    
    // Create filter and config
    let filter = SCContentFilter::builder()
        .display(display)
        .exclude_windows(&[])
        .build();
    
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080);
    
    // Create async stream with frame buffer
    let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
    stream.start_capture()?;
    
    // Capture frames asynchronously
    for _ in 0..10 {
        if let Some(frame) = stream.next().await {
            println!("📹 Got frame!");
        }
    }
    
    stream.stop_capture()?;
    Ok(())
}
```

### Window Capture with Audio

```rust
use screencapturekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = SCShareableContent::get()?;
    
    // Find a specific window
    let window = content.windows()
        .iter()
        .find(|w| w.title().as_deref() == Some("Safari"))
        .ok_or("Safari window not found")?;
    
    // Capture window with audio
    let filter = SCContentFilter::builder()
        .window(window)
        .build();
    
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);
    
    let mut stream = SCStream::new(&filter, &config);
    // Add handlers...
    stream.start_capture()?;
    
    Ok(())
}
```

### Content Picker (macOS 14.0+)

Use the system picker UI to let users choose what to capture:

```rust
use screencapturekit::content_sharing_picker::*;
use screencapturekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SCContentSharingPickerConfiguration::new();
    
    // Show picker - callback receives result when user selects or cancels
    SCContentSharingPicker::show(&config, |outcome| {
        match outcome {
            SCPickerOutcome::Picked(result) => {
                // Get dimensions from the picked content
                let (width, height) = result.pixel_size();
                println!("Selected: {}x{} (scale: {})", width, height, result.scale());
                
                let stream_config = SCStreamConfiguration::new()
                    .with_width(width)
                    .with_height(height);
                
                // Get filter for streaming
                let filter = result.filter();
                let mut stream = SCStream::new(&filter, &stream_config);
                // ...
            }
            SCPickerOutcome::Cancelled => println!("User cancelled"),
            SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
        }
    });
    
    Ok(())
}
```

### Async Content Picker (macOS 14.0+)

Use the async version in async contexts to avoid blocking:

```rust
use screencapturekit::async_api::AsyncSCContentSharingPicker;
use screencapturekit::content_sharing_picker::*;
use screencapturekit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SCContentSharingPickerConfiguration::new();
    
    // Async picker - doesn't block the executor
    match AsyncSCContentSharingPicker::show(&config).await {
        SCPickerOutcome::Picked(result) => {
            let (width, height) = result.pixel_size();
            println!("Selected: {}x{}", width, height);
            
            let filter = result.filter();
            // Use filter with stream...
        }
        SCPickerOutcome::Cancelled => println!("User cancelled"),
        SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
```

## 🎯 Key Concepts

### Builder Pattern

Different types use slightly different patterns:

```rust
// Content filters use .builder() with .build()
let filter = SCContentFilter::builder()
    .display(&display)
    .exclude_windows(&windows)
    .build();

// Stream configuration uses ::new() with .with_*() chainable methods
let config = SCStreamConfiguration::new()
    .with_width(1920)
    .with_height(1080)
    .with_pixel_format(PixelFormat::BGRA)
    .with_captures_audio(true);

// Options for content retrieval
let content = SCShareableContent::with_options()
    .on_screen_windows_only(true)
    .exclude_desktop_windows(true)
    .get()?;
```

### Custom Dispatch Queues

Control callback threading with custom dispatch queues:

```rust
use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};

let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);

stream.add_output_handler_with_queue(
    my_handler,
    SCStreamOutputType::Screen,
    Some(&queue)
);
```

**QoS Levels:**
- `Background` - Maintenance tasks
- `Utility` - Long-running tasks
- `Default` - Standard priority
- `UserInitiated` - User-initiated tasks
- `UserInteractive` - UI updates (highest priority)

### IOSurface Access

Zero-copy GPU texture access:

```rust
impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _type: SCStreamOutputType) {
        if let Some(pixel_buffer) = sample.image_buffer() {
            if let Some(surface) = pixel_buffer.iosurface() {
                let width = surface.width();
                let height = surface.height();
                
                // Use with Metal/OpenGL...
                println!("IOSurface: {}x{}", width, height);
            }
        }
    }
}
```

### Metal Integration

Built-in Metal types for hardware-accelerated rendering without external crates:

```rust
use screencapturekit::output::metal::{
    MetalDevice, MetalLayer, MetalRenderPassDescriptor, MetalRenderPipelineDescriptor,
    MTLLoadAction, MTLStoreAction, MTLPrimitiveType, MTLPixelFormat,
    Uniforms, SHADER_SOURCE,
};
use screencapturekit::output::CVPixelBufferIOSurface;

// Get the system default Metal device
let device = MetalDevice::system_default().expect("No Metal device");
let command_queue = device.create_command_queue().unwrap();

// Compile built-in shaders (supports BGRA, YCbCr, UI overlays)
let library = device.create_library_with_source(SHADER_SOURCE)?;

// Create render pipeline for textured rendering
let vert_fn = library.get_function("vertex_fullscreen").unwrap();
let frag_fn = library.get_function("fragment_textured").unwrap();
let pipeline_desc = MetalRenderPipelineDescriptor::new();
pipeline_desc.set_vertex_function(&vert_fn);
pipeline_desc.set_fragment_function(&frag_fn);
pipeline_desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);
let pipeline = device.create_render_pipeline_state(&pipeline_desc).unwrap();

// In your frame handler - create textures and render
impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _type: SCStreamOutputType) {
        if let Some(pixel_buffer) = sample.image_buffer() {
            if let Some(surface) = pixel_buffer.iosurface() {
                // Zero-copy texture creation from IOSurface
                if let Some(textures) = surface.create_metal_textures(&device) {
                    // Create uniforms for aspect-ratio-preserving rendering
                    let uniforms = Uniforms::from_captured_textures(
                        viewport_width, viewport_height, &textures
                    );
                    let uniform_buffer = device.create_buffer_with_data(&uniforms).unwrap();
                    
                    // Render to a CAMetalLayer drawable
                    let drawable = layer.next_drawable().unwrap();
                    let cmd_buffer = command_queue.command_buffer().unwrap();
                    
                    let render_pass = MetalRenderPassDescriptor::new();
                    render_pass.set_color_attachment_texture(0, &drawable.texture());
                    render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
                    render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);
                    
                    let encoder = cmd_buffer.render_command_encoder(&render_pass).unwrap();
                    encoder.set_render_pipeline_state(&pipeline);
                    encoder.set_vertex_buffer(&uniform_buffer, 0, 0);
                    encoder.set_fragment_texture(&textures.plane0, 0);
                    if let Some(ref plane1) = textures.plane1 {
                        encoder.set_fragment_texture(plane1, 1);  // CbCr for YCbCr
                    }
                    encoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4);
                    encoder.end_encoding();
                    
                    cmd_buffer.present_drawable(&drawable);
                    cmd_buffer.commit();
                }
            }
        }
    }
}
```

**Built-in Shader Functions:**
- `vertex_fullscreen` - Aspect-ratio-preserving fullscreen quad
- `fragment_textured` - BGRA/L10R single-texture rendering
- `fragment_ycbcr` - YCbCr biplanar (420v/420f) to RGB conversion
- `vertex_colored` / `fragment_colored` - UI overlay rendering

**Metal Types:**
- `MetalDevice`, `MetalCommandQueue`, `MetalCommandBuffer`
- `MetalTexture`, `MetalBuffer`, `MetalLayer`, `MetalDrawable`
- `MetalRenderPipelineState`, `MetalRenderPassDescriptor`
- `CapturedTextures<T>` - Multi-plane texture container (Y + CbCr for YCbCr formats)

## 🎛️ Feature Flags

### Core Features

| Feature | Description |
|---------|-------------|
| `async` | Runtime-agnostic async API (works with any executor) |

### macOS Version Features

Feature flags enable APIs for specific macOS versions. They are cumulative (enabling `macos_15_0` enables all earlier versions).

| Feature | macOS | APIs Enabled |
|---------|-------|--------------|
| `macos_13_0` | 13.0 Ventura | Audio capture, synchronization clock |
| `macos_14_0` | 14.0 Sonoma | Content picker, screenshots, content info |
| `macos_14_2` | 14.2 | Menu bar capture, child windows, presenter overlay |
| `macos_14_4` | 14.4 | Current process shareable content |
| `macos_15_0` | 15.0 Sequoia | Recording output, HDR capture, microphone |
| `macos_15_2` | 15.2 | Screenshot in rect, stream active/inactive delegates |
| `macos_26_0` | 26.0 | Advanced screenshot config, HDR screenshot output |

### Version-Specific Example

```rust
let mut config = SCStreamConfiguration::new()
    .with_width(1920)
    .with_height(1080);

#[cfg(feature = "macos_13_0")]
config.set_should_be_opaque(true);

#[cfg(feature = "macos_14_2")]
{
    config.set_ignores_shadows_single_window(true);
    config.set_includes_child_windows(false);
}
```

## 📚 API Overview

### Core Types

| Type | Description |
|------|-------------|
| [`SCShareableContent`] | Query available displays, windows, and applications |
| [`SCContentFilter`] | Define what to capture (display/window/app) |
| [`SCStreamConfiguration`] | Configure resolution, format, audio, etc. |
| [`SCStream`] | Main capture stream with output handlers |
| [`CMSampleBuffer`] | Frame data with timing and metadata |

[`SCShareableContent`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/shareable_content/struct.SCShareableContent.html
[`SCContentFilter`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/stream/content_filter/struct.SCContentFilter.html
[`SCStreamConfiguration`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/stream/configuration/struct.SCStreamConfiguration.html
[`SCStream`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/stream/sc_stream/struct.SCStream.html
[`CMSampleBuffer`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/cm/struct.CMSampleBuffer.html

### Async API (requires `async` feature)

| Type | Description |
|------|-------------|
| `AsyncSCShareableContent` | Async content queries |
| `AsyncSCStream` | Async stream with frame iteration |
| `AsyncSCScreenshotManager` | Async screenshot capture (macOS 14.0+) |
| `AsyncSCContentSharingPicker` | Async content picker UI (macOS 14.0+) |

### Display & Window Types

| Type | Description |
|------|-------------|
| `SCDisplay` | Display information (resolution, ID, frame) |
| `SCWindow` | Window information (title, bounds, owner, layer) |
| `SCRunningApplication` | Application information (name, bundle ID, PID) |

### Media Types

| Type | Description |
|------|-------------|
| `CMSampleBuffer` | Sample buffer with timing and attachments |
| `CMTime` | High-precision timestamps with timescale |
| `IOSurface` | GPU-backed pixel buffers for zero-copy access |
| `CGImage` | Core Graphics images for screenshots |
| `CVPixelBuffer` | Core Video pixel buffer with lock guards |

### Metal Types (`metal` module)

| Type | Description |
|------|-------------|
| `MetalDevice` | Metal GPU device wrapper |
| `MetalTexture` | Metal texture with automatic retain/release |
| `MetalBuffer` | Vertex/uniform buffer |
| `MetalCommandQueue` / `MetalCommandBuffer` | Command submission |
| `MetalLayer` | `CAMetalLayer` for window rendering |
| `MetalRenderPipelineState` | Compiled render pipeline |
| `CapturedTextures<T>` | Multi-plane texture container (Y + CbCr for YCbCr) |
| `Uniforms` | Shader uniform structure matching `SHADER_SOURCE` |

### Configuration Types

| Type | Description |
|------|-------------|
| `PixelFormat` | BGRA, YCbCr420v, YCbCr420f, l10r (10-bit) |
| `SCPresenterOverlayAlertSetting` | Privacy alert behavior |
| `SCCaptureDynamicRange` | HDR/SDR modes (macOS 15.0+) |
| `SCScreenshotConfiguration` | Advanced screenshot config (macOS 26.0+) |
| `SCScreenshotDynamicRange` | SDR/HDR screenshot output (macOS 26.0+) |

## 🏃 Examples

The [`examples/`](examples/) directory contains focused API demonstrations:

### Quick Start (Numbered by Complexity)
1. **`01_basic_capture.rs`** - Simplest screen capture
2. **`02_window_capture.rs`** - Capture specific windows
3. **`03_audio_capture.rs`** - Audio + video capture
4. **`04_pixel_access.rs`** - Read pixel data with `std::io::Cursor`
5. **`05_screenshot.rs`** - Single screenshot, HDR capture (macOS 14.0+, 26.0+)
6. **`06_iosurface.rs`** - Zero-copy GPU buffers
7. **`07_list_content.rs`** - List available content
8. **`08_async.rs`** - Async/await API with multiple examples
9. **`09_closure_handlers.rs`** - Closure-based handlers and delegates
10. **`10_recording_output.rs`** - Direct video file recording (macOS 15.0+)
11. **`11_content_picker.rs`** - System UI for content selection (macOS 14.0+)
12. **`12_stream_updates.rs`** - Dynamic config/filter updates
13. **`13_advanced_config.rs`** - HDR, presets, microphone (macOS 15.0+)
14. **`14_app_capture.rs`** - Application-based filtering
15. **`15_memory_leak_check.rs`** - Memory leak detection with `leaks`
16. **`16_full_metal_app/`** - Full Metal GUI application (macOS 14.0+)
17. **`17_metal_textures.rs`** - Metal texture creation from IOSurface

See [`examples/README.md`](examples/README.md) for detailed descriptions.

Run an example:

```bash
# Basic examples
cargo run --example 01_basic_capture
cargo run --example 09_closure_handlers
cargo run --example 12_stream_updates
cargo run --example 14_app_capture
cargo run --example 17_metal_textures

# Feature-gated examples
cargo run --example 05_screenshot --features macos_14_0
cargo run --example 08_async --features async
cargo run --example 10_recording_output --features macos_15_0
cargo run --example 11_content_picker --features macos_14_0
cargo run --example 13_advanced_config --features macos_15_0
cargo run --example 16_full_metal_app --features macos_14_0
```

## 🧪 Testing

### Run Tests

```bash
# All tests
cargo test

# With features
cargo test --features async
cargo test --all-features

# Specific test
cargo test test_stream_configuration
```

### Linting

```bash
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

## 🏗️ Architecture

### Module Organization

```
screencapturekit/
├── cm/                     # Core Media (CMSampleBuffer, CMTime, CVPixelBuffer)
├── cg/                     # Core Graphics (CGRect, CGImage)
├── stream/                 # Stream management
│   ├── configuration/      # SCStreamConfiguration
│   ├── content_filter/     # SCContentFilter
│   └── sc_stream/          # SCStream
├── shareable_content/      # SCShareableContent, SCDisplay, SCWindow
├── output/                 # Frame buffers and pixel data
├── dispatch_queue/         # Custom dispatch queues
├── error/                  # Error types
├── screenshot_manager/     # SCScreenshotManager (macOS 14.0+)
├── content_sharing_picker/ # SCContentSharingPicker (macOS 14.0+)
├── recording_output/       # SCRecordingOutput (macOS 15.0+)
├── async_api/              # Async wrappers (feature = "async")
├── utils/                  # FFI strings, FourCharCode utilities
└── prelude/                # Convenience re-exports
```

### Memory Management

- **Reference Counting** - Proper CFRetain/CFRelease for all CoreFoundation types
- **RAII** - Automatic cleanup in Drop implementations
- **Thread Safety** - Safe to share across threads (where supported)
- **Leak Free** - Comprehensive leak tests ensure no memory leaks

## ❓ Troubleshooting

### Permission Denied / No Displays Found

**Problem**: `SCShareableContent::get()` returns an error or empty lists.

**Solution**: Grant screen recording permission:
1. Open **System Preferences** → **Privacy & Security** → **Screen Recording**
2. Add your app or Terminal to the list
3. Restart your application

For development, you may need to add Terminal.app to the allowed list.

### Entitlements for App Store / Notarization

**Problem**: App crashes or permissions fail after notarization.

**Solution**: Add required entitlements to your `entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.screen-capture</key>
    <true/>
</dict>
</plist>
```

### Black Frames / No Video Data

**Problem**: Frames are received but contain no visible content.

**Solutions**:
1. Ensure the captured window/display is visible (not minimized)
2. Check that `pixel_format` matches your processing expectations
3. Verify the content filter includes the correct display/window
4. On Apple Silicon, ensure proper GPU access

### Audio Capture Not Working

**Problem**: Audio samples not received or empty.

**Solutions**:
1. Enable audio capture: `.with_captures_audio(true)`
2. Add an audio output handler: `stream.add_output_handler(handler, SCStreamOutputType::Audio)`
3. Verify `sample_rate` and `channel_count` are set correctly

### Build Errors

**Problem**: Compilation fails with Swift bridge errors.

**Solutions**:
1. Ensure Xcode Command Line Tools are installed: `xcode-select --install`
2. Clean and rebuild: `cargo clean && cargo build`
3. Check that you're on macOS (this crate is macOS-only)

## 🔧 Platform Requirements

- **macOS 12.3+** (Monterey) - Base ScreenCaptureKit support
- **macOS 13.0+** (Ventura) - Audio capture, synchronization clock
- **macOS 14.0+** (Sonoma) - Content picker, screenshots, content info
- **macOS 15.0+** (Sequoia) - Recording output, HDR capture, microphone
- **macOS 26.0+** (Tahoe) - Advanced screenshot config, HDR screenshot output

### Screen Recording Permission

Screen recording requires explicit user permission. For development:
- Terminal/IDE must be in **System Preferences** → **Privacy & Security** → **Screen Recording**

For distribution:
- Add `NSScreenCaptureUsageDescription` to your `Info.plist`
- Sign with appropriate entitlements for notarization

## 🤝 Contributing

Contributions welcome! Please:

1. Follow existing code patterns (builder pattern with `::new()` and `.with_*()` methods)
2. Add tests for new functionality
3. Run `cargo test` and `cargo clippy`
4. Update documentation

## 👥 Contributors

Thanks to everyone who has contributed to this project!

- [Per Johansson](https://github.com/doom-fish) - Maintainer
- [Iason Paraskevopoulos](https://github.com/iasparaskev)
- [Kris Krolak](https://github.com/kriskrolak)
- [Tokuhiro Matsuno](https://github.com/tokuhirom)
- [Pranav Joglekar](https://github.com/pranavj1001)
- [Alex Jiao](https://github.com/uohzxela)
- [Charles](https://github.com/aizukanne)
- [bigduu](https://github.com/bigduu)

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
