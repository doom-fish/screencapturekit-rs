# Tauri Screen Capture Example

A complete Tauri 2.0 application demonstrating screencapturekit-rs integration for macOS screen capture.

## Features

- 📸 **Screenshot capture** - Take screenshots via the system picker
- 🎥 **Screen recording** - Record screen to MP4 (macOS 15.0+)
- 📋 **List content** - View available displays, windows, and apps
- 🖼️ **Preview** - Display captured screenshots in the UI

## Project Structure

```
22_tauri_app/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs          # Tauri commands using screencapturekit
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   └── Info.plist          # macOS permissions
├── src/
│   ├── index.html          # Main UI
│   ├── main.js             # Frontend logic
│   └── styles.css          # Styling
├── package.json            # Node dependencies
└── README.md
```

## Setup

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Xcode Command Line Tools
- macOS 14.0+ (for content picker)

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
| `take_screenshot` | Capture screenshot via picker |
| `start_recording` | Start screen recording |
| `stop_recording` | Stop recording and save file |

## Code Highlights

### Rust Backend (src-tauri/src/lib.rs)

```rust
use screencapturekit::prelude::*;

#[tauri::command]
async fn take_screenshot() -> Result<Vec<u8>, String> {
    let content = SCShareableContent::get().map_err(|e| e.to_string())?;
    let display = &content.displays()[0];
    
    let filter = SCContentFilter::with()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    
    let config = SCStreamConfiguration::new()
        .with_width(display.width() as u32)
        .with_height(display.height() as u32);
    
    let image = SCScreenshotManager::capture_image(&filter, &config)
        .map_err(|e| e.to_string())?;
    
    Ok(image.to_png())
}
```

### Frontend (src/main.js)

```javascript
import { invoke } from '@tauri-apps/api/core';

async function captureScreen() {
  const pngData = await invoke('take_screenshot');
  const blob = new Blob([new Uint8Array(pngData)], { type: 'image/png' });
  document.getElementById('preview').src = URL.createObjectURL(blob);
}
```

## License

MIT / Apache-2.0
