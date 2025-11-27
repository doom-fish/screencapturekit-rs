# ScreenCaptureKit Swift Bridge

Professional, modular FFI bridge for ScreenCaptureKit to Rust.

## Architecture

This bridge provides clean FFI exports from Swift to Rust using the `@_cdecl` attribute for C-compatible symbol names.

### Module Organization

The bridge is organized into modular Swift files:

```
Sources/ScreenCaptureKitBridge/
├── Core.swift                  (26 lines)  - Memory management helpers
├── ShareableContent.swift      (381 lines) - Content discovery APIs
├── StreamConfiguration.swift   (467 lines) - Stream configuration
├── Stream.swift                (390 lines) - Stream control & delegates
├── ScreenshotManager.swift     (59 lines)  - Screenshot capture
├── RecordingOutput.swift       (131 lines) - Video recording (macOS 15+)
└── ContentSharingPicker.swift  (60 lines)  - System picker (macOS 14+)

Sources/CoreGraphics/           - CGImage, CGRect utilities
Sources/CoreMedia/              - CMSampleBuffer, CMTime utilities
Sources/CoreVideo/              - CVPixelBuffer utilities
Sources/IOSurface/              - IOSurface utilities
Sources/Dispatch/               - DispatchQueue utilities
```

## Design Principles

### 1. **Clear Separation of Concerns**
Each module handles a specific aspect of ScreenCaptureKit:
- **Core**: Memory safety primitives
- **ShareableContent**: Discovery and enumeration
- **Configuration**: Stream setup
- **Stream**: Capture control
- **Media**: Frame data access
- **Output**: Advanced buffer handling

### 2. **Consistent Naming**
All FFI functions follow a clear naming pattern:
```
<domain>_<object>_<action>
```
Examples:
- `sc_shareable_content_get`
- `sc_display_get_width`
- `sc_stream_start_capture`
- `cv_pixel_buffer_lock_base_address`

### 3. **Memory Management**
Three helper functions handle all Swift↔Rust memory:
- `retain<T>(_:)` - Passes retained object to Rust
- `unretained<T>(_:)` - Gets unretained reference from Rust pointer
- `release(_:)` - Releases Rust-held Swift object

### 4. **Error Handling**
Async operations use callback-based error reporting:
```swift
callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
```
- Success: `callback(true, nil)`
- Failure: `callback(false, errorCString)`

### 5. **Type Safety**
Strong typing with explicit casts:
```swift
let scDisplay: SCDisplay = unretained(display)
let outputType: SCStreamOutputType = type == 0 ? .screen : .audio
```

## Statistics

| Metric | Value |
|--------|-------|
| Total Lines | ~1,500 |
| Swift Files | 12 |
| FFI Functions | 80+ |
| Memory Helpers | 3 |
| Delegate Classes | 2 |

## Build Integration

Built automatically by `build.rs` using Swift Package Manager:
```bash
swift build -c release
```

Output: `libScreenCaptureKitBridge.a` linked into Rust binary.

## Versioning

Follows semantic versioning aligned with parent crate:
- **1.0.0**: Initial modular release
- Swift-first design
- No Objective-C dependencies
- Professional organization

## Future Improvements

1. ~~**Multi-file modules**~~ ✅ Already split into 7 Swift files
2. ~~**Documentation comments**~~ ✅ Added inline Swift documentation
3. ~~**Error types**~~ ✅ Added `SCBridgeError` enum with strongly typed errors
4. **Testing** - Swift unit tests for FFI layer
5. ~~**Performance**~~ - Skipped (not a priority)

## Related Documentation

- `../../README.md` - Main project documentation
- `../../src/ffi/mod.rs` - Rust FFI declarations
- `../../REFACTOR_SUMMARY.md` - v1.0 refactoring details

---

**Version**: 1.0.0  
**Language**: Swift 5.9+  
**Platform**: macOS 12.3+  
**License**: MIT/Apache-2.0
