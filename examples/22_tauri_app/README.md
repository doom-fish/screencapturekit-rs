# Tauri Screen Capture Example

A complete Tauri 2.0 application demonstrating screencapturekit-rs integration for macOS screen capture with WebGL rendering.

This is a nested Cargo package, so Cargo does not include it in the published
`screencapturekit` crate archive. Build it from a repository checkout.

## Features

- 📸 **Screenshot capture** - Take screenshots of displays and windows
- 🖼️ **WebGL Preview** - Hardware-accelerated rendering of captured frames
- 📋 **List content** - View available displays and windows

## Project Structure

```
22_tauri_app/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs          # Tauri commands using screencapturekit
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   ├── icons/icon.png      # Application icon used by Tauri code generation
│   └── Info.plist          # macOS permissions
├── src/
│   ├── index.html          # Main UI
│   ├── main.js             # Frontend logic with WebGL
│   └── styles.css          # Styling
├── package.json            # Node dependencies
└── README.md
```

## Setup

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Xcode Command Line Tools
- macOS 14.0+ (for screenshots)

### Installation

```bash
cd examples/22_tauri_app

# Install Node dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Permissions

The app requires **Screen Recording** permission on macOS:
1. Run the app once
2. Go to **System Preferences** → **Privacy & Security** → **Screen Recording**
3. Enable the app
4. Restart the app

## Commands

The Tauri backend exposes these commands:

| Command | Description |
|---------|-------------|
| `list_displays` | Get available displays |
| `list_windows` | Get available windows |
| `take_screenshot_display` | Capture display screenshot (RGBA) |
| `take_screenshot_window` | Capture window screenshot (RGBA) |
| `get_status` | Get current status |

## Code Highlights

### Rust Backend (src-tauri/src/lib.rs)

```rust
use screencapturekit::prelude::*;
use screencapturekit::screenshot_manager::SCScreenshotManager;

#[tauri::command]
fn take_screenshot_display(display_id: Option<u32>) -> Result<ScreenshotResult, String> {
    let content = SCShareableContent::get().map_err(|e| e.to_string())?;
    let display = &content.displays()[0];
    
    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    
    let config = SCStreamConfiguration::new()
        .with_width(display.width() as u32)
        .with_height(display.height() as u32)
        .with_pixel_format(PixelFormat::BGRA);
    
    let image = SCScreenshotManager::capture_image(&filter, &config)?;
    
    // Get RGBA data for WebGL rendering
    let rgba_data = image.rgba_data()?;
    
    Ok(ScreenshotResult {
        data: STANDARD.encode(&rgba_data),
        width: image.width(),
        height: image.height(),
    })
}
```

### WebGL Rendering (src/main.js)

The frontend uses WebGL to render RGBA pixel data directly from `CGImage::rgba_data()`.

## License

MIT / Apache-2.0
