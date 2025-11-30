# Examples

Runnable examples demonstrating core ScreenCaptureKit APIs.

## Quick Start

```bash
cargo run --example 01_basic_capture
```

## Examples

| # | Example | Description | Features |
|---|---------|-------------|----------|
| 01 | `basic_capture` | Simplest screen capture | - |
| 02 | `window_capture` | Capture specific window | - |
| 03 | `audio_capture` | Audio + video capture | - |
| 04 | `pixel_access` | Read pixel data from frames | - |
| 05 | `screenshot` | Single screenshot, HDR capture | `macos_14_0`, `macos_26_0` |
| 06 | `iosurface` | Zero-copy GPU buffer access | - |
| 07 | `list_content` | List displays/windows/apps | - |
| 08 | `async` | Async/await API, async picker | `async`, `macos_14_0` |
| 09 | `closure_handlers` | Closures as handlers | - |
| 10 | `recording_output` | Direct video recording | `macos_15_0` |
| 11 | `content_picker` | System content picker UI | `macos_14_0` |
| 12 | `stream_updates` | Dynamic config/filter updates | - |
| 13 | `advanced_config` | HDR, presets, microphone | `macos_15_0` |
| 14 | `app_capture` | Application-based filtering | - |
| 15 | `memory_leak_check` | Memory leak detection with `leaks` | - |
| - | `metal_overlay` | Metal GPU rendering + overlay UI | `macos_14_0` |

## Running with Features

```bash
# Basic examples (no features needed)
cargo run --example 01_basic_capture
cargo run --example 02_window_capture
cargo run --example 12_stream_updates
cargo run --example 14_app_capture

# Async example
cargo run --example 08_async --features async

# Async with picker
cargo run --example 08_async --features "async,macos_14_0"

# macOS 14+ examples
cargo run --example 05_screenshot --features macos_14_0
cargo run --example 11_content_picker --features macos_14_0

# macOS 15+ examples  
cargo run --example 10_recording_output --features macos_15_0
cargo run --example 13_advanced_config --features macos_15_0

# macOS 26+ HDR screenshot
cargo run --example 05_screenshot --features macos_26_0

# Metal GUI example
cargo run --example metal_overlay --features macos_14_0

# All features
cargo run --example 08_async --all-features
```

## Tips

- Examples are numbered by complexity - start with `01`
- Each example focuses on one API concept
- Check source code for detailed comments

## Metal Overlay Example

Example 12 (`metal_overlay`) is a full GUI application demonstrating:
- **Metal GPU rendering** with runtime shader compilation
- **Bitmap font** rendering for overlay text (8x8 pixel glyphs)
- **Audio waveform** visualization with VU meter
- **Interactive menu** with keyboard navigation
- **Screen capture** integration via ScreenCaptureKit

### Controls
- `S` - Start screen capture
- `X` - Stop capture
- `W` - Toggle waveform display
- `M` / `Escape` - Toggle menu
- `↑`/`↓` - Navigate menu
- `Q` - Quit

### Data Structure Alignment

The example shows proper Rust/Metal data alignment using `#[repr(C)]`:

```rust
// Rust struct matching Metal shader vertex input
#[repr(C)]
struct Vertex {
    position: [f32; 2],  // 8 bytes - matches packed_float2
    color: [f32; 4],     // 16 bytes - matches packed_float4
}

// Uniforms with explicit padding for 16-byte alignment
#[repr(C)]
struct Uniforms {
    viewport_size: [f32; 2],  // 8 bytes
    time: f32,                // 4 bytes
    _padding: f32,            // 4 bytes (16-byte alignment)
}
```
