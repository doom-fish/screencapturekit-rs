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
| 05 | `screenshot` | Single screenshot | `macos_14_0` |
| 06 | `iosurface` | Zero-copy GPU buffer access | - |
| 07 | `list_content` | List displays/windows/apps | - |
| 08 | `async` | Async/await API | `async` |
| 09 | `closure_handlers` | Closures as handlers | - |
| 10 | `recording_output` | Direct video recording | `macos_15_0` |
| 11 | `content_picker` | System content picker UI | `macos_14_0` |

## Running with Features

```bash
# Async example
cargo run --example 08_async --features async

# macOS 14+ examples
cargo run --example 05_screenshot --features macos_14_0
cargo run --example 11_content_picker --features macos_14_0

# macOS 15+ examples  
cargo run --example 10_recording_output --features macos_15_0

# All features
cargo run --example 08_async --all-features
```

## Tips

- Examples are numbered by complexity - start with `01`
- Each example focuses on one API concept
- Check source code for detailed comments
