# ScreenCaptureKit-rs

[![Crates.io](https://img.shields.io/crates/v/screencapturekit.svg)](https://crates.io/crates/screencapturekit)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://doom-fish.github.io/screencapturekit-rs/screencapturekit/)
[![License](https://img.shields.io/crates/l/screencapturekit.svg)](https://github.com/doom-fish/screencapturekit-rs#license)
[![Build Status](https://img.shields.io/github/actions/workflow/status/doom-fish/screencapturekit-rs/build.yml?branch=main)](https://github.com/doom-fish/screencapturekit-rs/actions)

> **💼 Looking for a hosted desktop recording API?**  
> Check out [Recall.ai](https://www.recall.ai/product/desktop-recording-sdk?utm_source=github&utm_medium=sponsorship&utm_campaign=screencapturekit-rs) - an API for recording Zoom, Google Meet, Microsoft Teams, in-person meetings, and more.

Safe, idiomatic Rust bindings for macOS ScreenCaptureKit framework.

Capture screen content, windows, and applications with high performance and low overhead on macOS 12.3+.

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
screencapturekit = "1.0"
```

For async support:

```toml
[dependencies]
screencapturekit = { version = "1.0", features = ["async"] }
```

For latest macOS features:

```toml
[dependencies]
screencapturekit = { version = "1.0", features = ["macos_15_0"] }
```

## 🚀 Quick Start

### Basic Screen Capture

```rust
use screencapturekit::prelude::*;

struct Handler;

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _type: SCStreamOutputType) {
        println!("📹 Received frame at {} pts", sample.get_presentation_timestamp());
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
    
    let config = SCStreamConfiguration::builder()
        .width(1920)
        .height(1080)
        .pixel_format(PixelFormat::BGRA)
        .build();
    
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
    
    let config = SCStreamConfiguration::builder()
        .width(1920)
        .height(1080)
        .build();
    
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
    
    let config = SCStreamConfiguration::builder()
        .width(1920)
        .height(1080)
        .captures_audio(true)
        .sample_rate(48000)
        .channel_count(2)
        .build();
    
    let mut stream = SCStream::new(&filter, &config);
    // Add handlers...
    stream.start_capture()?;
    
    Ok(())
}
```

## 🎯 Key Concepts

### Builder Pattern

All types use a consistent builder pattern with a final `.build()` call:

```rust
// Content filters
let filter = SCContentFilter::builder()
    .display(&display)
    .exclude_windows(&windows)
    .build();

// Stream configuration
let config = SCStreamConfiguration::builder()
    .width(1920)
    .height(1080)
    .pixel_format(PixelFormat::BGRA)
    .captures_audio(true)
    .build();

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
        if let Some(pixel_buffer) = sample.get_image_buffer() {
            if let Some(surface) = pixel_buffer.get_iosurface() {
                let width = surface.get_width();
                let height = surface.get_height();
                let pixel_format = surface.get_pixel_format();
                
                // Use with Metal/OpenGL...
                println!("IOSurface: {}x{} format: {}", width, height, pixel_format);
            }
        }
    }
}
```

## 🎛️ Feature Flags

### Core Features

| Feature | Description |
|---------|-------------|
| `async` | Runtime-agnostic async API (works with any executor) |

### macOS Version Features

Feature flags enable APIs for specific macOS versions. They are cumulative (enabling `macos_15_0` enables all earlier versions).

| Feature | macOS | APIs Enabled |
|---------|-------|--------------|
| `macos_13_0` | 13.0 Ventura | Opacity configuration |
| `macos_14_0` | 14.0 Sonoma | Content picker, clipboard ignore, shadow displays |
| `macos_14_2` | 14.2 | Capture fractions, shadow control, child windows |
| `macos_14_4` | 14.4 | Future features |
| `macos_15_0` | 15.0 Sequoia | Recording output, HDR capture |

### Version-Specific Example

```rust
let config = SCStreamConfiguration::builder()
    .width(1920)
    .height(1080)
    .should_be_opaque(true)  // macOS 13.0+
    .ignores_shadows_single_window(true)  // macOS 14.2+
    .includes_child_windows(false)  // macOS 14.2+
    .build();
```

## 📚 API Overview

### Core Types

- **`SCShareableContent`** - Query available displays, windows, and applications
- **`SCContentFilter`** - Define what to capture (display/window/app)
- **`SCStreamConfiguration`** - Configure resolution, format, audio, etc.
- **`SCStream`** - Main capture stream with output handlers
- **`CMSampleBuffer`** - Frame data with timing and metadata

### Async API (requires `async` feature)

- **`AsyncSCShareableContent`** - Async content queries
- **`AsyncSCStream`** - Async stream with frame iteration

### Display & Window Types

- **`SCDisplay`** - Display information (resolution, ID, etc.)
- **`SCWindow`** - Window information (title, bounds, owner, etc.)
- **`SCRunningApplication`** - Application information (name, PID, etc.)

### Media Types

- **`CMSampleBuffer`** - Sample buffer with timing and attachments
- **`CMTime`** - High-precision timestamps
- **`IOSurface`** - GPU-backed pixel buffers
- **`CGImage`** - CoreGraphics images

### Configuration Types

- **`PixelFormat`** - BGRA, YCbCr420v, YCbCr420f, l10r (10-bit)
- **`SCPresenterOverlayAlertSetting`** - Privacy alert behavior
- **`SCCaptureDynamicRange`** - HDR/SDR modes (macOS 15.0+)

## 🏃 Examples

The [`examples/`](examples/) directory contains focused API demonstrations:

### Quick Start (Numbered by Complexity)
1. **`01_basic_capture.rs`** - Simplest screen capture
2. **`02_window_capture.rs`** - Capture specific windows
3. **`03_audio_capture.rs`** - Audio + video capture
4. **`04_pixel_access.rs`** - Read pixel data with `std::io::Cursor`
5. **`05_screenshot.rs`** - Single screenshot (macOS 14.0+)
6. **`06_iosurface.rs`** - Zero-copy GPU buffers
7. **`07_list_content.rs`** - List available content
8. **`08_async.rs`** - Async/await API with multiple examples
9. **`09_closure_handlers.rs`** - Closure-based handlers and delegates
10. **`10_recording_output.rs`** - Direct video file recording (macOS 15.0+)
11. **`11_content_picker.rs`** - System UI for content selection (macOS 14.0+)

See [`examples/README.md`](examples/README.md) for detailed descriptions.

Run an example:

```bash
# Basic examples
cargo run --example 01_basic_capture
cargo run --example 09_closure_handlers

# Feature-gated examples
cargo run --example 05_screenshot --features macos_14_0
cargo run --example 08_async --features async
cargo run --example 10_recording_output --features macos_15_0
cargo run --example 11_content_picker --features macos_14_0
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

## 🔧 Platform Requirements

- **macOS 12.3+** (Monterey) - Base ScreenCaptureKit support
- **macOS 13.0+** (Ventura) - Additional features with `macos_13_0`
- **macOS 14.0+** (Sonoma) - Content picker, advanced config
- **macOS 15.0+** (Sequoia) - Recording output, HDR capture

## 🤝 Contributing

Contributions welcome! Please:

1. Follow existing code patterns (builder pattern with `::builder()`)
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
