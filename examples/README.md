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
| 16 | `full_metal_app` | Full Metal GUI application | `macos_14_0` |
| 17 | `metal_textures` | Metal texture creation from IOSurface | - |
| 18 | `wgpu_integration` | Zero-copy wgpu integration | - |
| 19 | `ffmpeg_encoding` | Real-time H.264 encoding via FFmpeg | - |
| 20 | `egui_viewer` | egui screen viewer integration | - |
| 21 | `bevy_streaming` | Bevy texture streaming | - |
| 22 | `tauri_app` | Tauri 2.0 desktop app with WebGL | `macos_14_0` |
| 23 | `client_server` | Client/server screen sharing | - |
| 24 | `webrtc` | WebRTC screen streaming to browser | - |

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
cargo run --example 16_full_metal_app --features macos_14_0

# Metal textures (no features needed)
cargo run --example 17_metal_textures

# wgpu integration
cargo run --example 18_wgpu_integration

# FFmpeg encoding (requires: brew install ffmpeg)
cargo run --example 19_ffmpeg_encoding

# egui viewer
cargo run --example 20_egui_viewer

# Bevy streaming
cargo run --example 21_bevy_streaming

# Tauri app (separate project, use npm)
cd examples/22_tauri_app && npm install && npm run tauri dev

# Client/server screen sharing
cargo run --example 23_client_server_server  # Terminal 1
cargo run --example 23_client_server_client  # Terminal 2

# WebRTC screen streaming (open http://127.0.0.1:8080 in browser)
cargo run --example 24_webrtc

# All features
cargo run --example 08_async --all-features
```

## Tips

- Examples are numbered by complexity - start with `01`
- Each example focuses on one API concept
- Check source code for detailed comments

## Full Metal App Example

Example 16 (`16_full_metal_app`) is a complete macOS application showcasing the full ScreenCaptureKit API:

### Features

- **Metal GPU Rendering** - Hardware-accelerated graphics with runtime shader compilation
- **Screen Capture** - Real-time display/window capture via ScreenCaptureKit
- **Content Picker** - System UI for selecting capture source (macOS 14.0+)
- **Audio Visualization** - Real-time waveform display with VU meters
- **Screenshot Capture** - Single-frame capture with HDR support (macOS 14.0+/26.0+)
- **Video Recording** - Direct-to-file recording (macOS 15.0+)
- **Microphone Capture** - Audio input with device selection (macOS 15.0+)
- **Bitmap Font Rendering** - Custom 8x8 pixel glyph overlay text
- **Interactive Menu** - Keyboard-navigable settings UI

### Running

```bash
# Basic (macOS 14.0+)
cargo run --example 16_full_metal_app --features macos_14_0

# With recording support (macOS 15.0+)
cargo run --example 16_full_metal_app --features macos_15_0

# With HDR screenshots (macOS 26.0+)
cargo run --example 16_full_metal_app --features macos_26_0
```

### Controls

**Initial Menu** (before picking a source):
- `↑`/`↓` - Navigate menu items  
- `Enter` - Pick Source / Quit

**Main Menu** (after picking a source - capture auto-starts):
- `↑`/`↓` - Navigate menu items
- `Enter` - Select item (Stop/Start Capture, Screenshot, Record, Config, Change Source, Quit)
- `Esc`/`H` - Hide menu

**Direct Controls** (when menu hidden):
- `P` - Open content picker
- `Space` - Start/stop capture
- `S` - Take screenshot
- `R` - Start/stop recording (macOS 15.0+)
- `W` - Toggle waveform display
- `M` - Toggle microphone
- `C` - Open config menu
- `H` - Show menu
- `Q`/`Esc` - Quit

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
