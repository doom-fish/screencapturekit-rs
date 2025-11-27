# Swift Bridge

FFI bridge from Swift's ScreenCaptureKit to Rust using `@_cdecl` for C-compatible symbols.

## Structure

```
Sources/
├── ScreenCaptureKitBridge/
│   ├── Core.swift              - Memory management & error types
│   ├── ShareableContent.swift  - Content discovery
│   ├── StreamConfiguration.swift - Stream settings
│   ├── Stream.swift            - Capture control
│   ├── ScreenshotManager.swift - Screenshots (macOS 14+)
│   ├── RecordingOutput.swift   - Recording (macOS 15+)
│   └── ContentSharingPicker.swift - System picker (macOS 14+)
├── CoreGraphics/               - CGImage, CGRect
├── CoreMedia/                  - CMSampleBuffer, CMTime
├── CoreVideo/                  - CVPixelBuffer
├── IOSurface/                  - IOSurface access
└── Dispatch/                   - DispatchQueue
```

## FFI Naming Convention

```
<domain>_<object>_<action>
```

Examples: `sc_stream_start_capture`, `cv_pixel_buffer_lock_base_address`

## Memory Management

```swift
retain<T>(_:)     // Pass retained object to Rust
unretained<T>(_:) // Get reference from Rust pointer  
release(_:)       // Release Rust-held object
```

## Error Handling

Uses `SCBridgeError` enum with typed cases:
- `contentUnavailable`, `streamError`, `configurationError`
- `screenshotError`, `recordingError`, `pickerError`
- `invalidParameter`, `permissionDenied`, `unknown`

## Build

Built automatically by `build.rs`:
```bash
swift build -c release
```

---

**Platform**: macOS 12.3+ | **Swift**: 5.9+ | **License**: MIT/Apache-2.0
