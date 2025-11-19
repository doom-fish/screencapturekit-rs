# ScreenCaptureKit Examples

API-focused examples demonstrating core functionality.

## Quick Start Examples

These examples demonstrate the essential APIs in order of complexity:

### 1. Basic Capture (`01_basic_capture.rs`)
The simplest screen capture example.
```bash
cargo run --example 01_basic_capture
```
**Demonstrates:**
- Getting shareable content
- Creating content filters
- Configuring streams
- Starting/stopping capture

### 2. Window Capture (`02_window_capture.rs`)
Capture a specific window instead of the whole screen.
```bash
cargo run --example 02_window_capture
```
**Demonstrates:**
- Listing windows
- Filtering by application
- Window-specific capture

### 3. Audio Capture (`03_audio_capture.rs`)
Capture audio along with video.
```bash
cargo run --example 03_audio_capture
```
**Demonstrates:**
- Enabling audio capture
- Audio configuration
- Handling audio/video callbacks

### 4. Pixel Access (`04_pixel_access.rs`)
Access and read pixel data from frames.
```bash
cargo run --example 04_pixel_access
```
**Demonstrates:**
- Locking pixel buffers
- Using `std::io::Cursor` for reading
- Reading specific pixel coordinates
- Direct slice access

### 5. Screenshot (`05_screenshot.rs`)
Take a single screenshot (macOS 14.0+).
```bash
cargo run --example 05_screenshot --features macos_14_0
```
**Demonstrates:**
- `SCScreenshotManager` API
- Saving as PNG

### 6. Async API (`06_async.rs`)
Async/await API with any runtime.
```bash
cargo run --example 06_async --features async
```
**Demonstrates:**
- Async content retrieval
- Async screenshot capture
- Works with Tokio, async-std, smol, etc.

### 7. IOSurface (`07_iosurface.rs`)
Zero-copy GPU buffer access.
```bash
cargo run --example 07_iosurface
```
**Demonstrates:**
- IOSurface detection
- IOSurface properties
- Locking and accessing IOSurface data

### 8. List Content (`08_list_content.rs`)
List all available shareable content.
```bash
cargo run --example 08_list_content
```
**Demonstrates:**
- Listing displays
- Listing windows
- Listing applications
- Filtering content

## Advanced Examples

These examples show more complex usage patterns:

### `async_demo.rs`
Comprehensive async API demonstration with multiple features.
```bash
cargo run --example async_demo --features async
```

### `async_runtime_agnostic.rs`
Shows async API works with any runtime.
```bash
cargo run --example async_runtime_agnostic --features async
```

## Running Examples

All examples:
```bash
cargo run --example 01_basic_capture
cargo run --example 02_window_capture
cargo run --example 03_audio_capture
cargo run --example 04_pixel_access
cargo run --example 05_screenshot --features macos_14_0
cargo run --example 06_async --features async
cargo run --example 07_iosurface
cargo run --example 08_list_content
```

With all features:
```bash
cargo run --example 06_async --all-features
```

## Example Structure

Each example follows this pattern:

1. **Clear focus** - One API concept per example
2. **Minimal code** - Only what's needed to demonstrate the API
3. **Well commented** - Explains what each step does
4. **Runnable** - Works out of the box with `cargo run`

## Tips

- Examples are numbered by complexity
- Start with `01_basic_capture` if you're new
- All examples print helpful output
- Use `--features` flags as shown above
- Check example source code for detailed comments
