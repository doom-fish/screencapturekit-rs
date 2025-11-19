# ScreenCaptureKit Swift Bridge

Professional, modular FFI bridge for ScreenCaptureKit to Rust.

## Architecture

This bridge provides clean FFI exports from Swift to Rust using the `@_cdecl` attribute for C-compatible symbol names.

### Module Organization

The single `ScreenCaptureKitBridge.swift` file is organized into logical modules using `// MARK:` comments:

```
ScreenCaptureKitBridge.swift (844 lines → Professional modular structure)
├── Core: Memory Management (retain/unretained/release helpers)
├── ShareableContent: Content Discovery
│   ├── SCShareableContent API
│   ├── SCDisplay API
│   ├── SCWindow API
│   └── SCRunningApplication API
├── Configuration: Stream Configuration
│   ├── Basic settings (width, height, cursor)
│   ├── Audio configuration
│   ├── Advanced settings (frame interval, queue depth, pixel format)
│   └── Color and display settings
├── Stream: Stream Control
│   ├── SCContentFilter creation
│   ├── Stream delegates and output handlers
│   └── Stream lifecycle (create, start, stop, update)
├── Media: Buffer Management
│   ├── CMSampleBuffer API
│   └── CVPixelBuffer API
└── Output: IOSurface Support
    └── IOSurface locking and properties
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
| Total Lines | ~1,000 |
| FFI Functions | 80+ |
| Modules | 6 |
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

1. **Multi-file modules** - Split into separate `.swift` files when complexity grows
2. **Documentation comments** - Add inline Swift documentation
3. **Error types** - Strongly typed error enum instead of strings
4. **Testing** - Swift unit tests for FFI layer
5. **Performance** - Profile and optimize hot paths

## Related Documentation

- `../../README.md` - Main project documentation
- `../../src/ffi/mod.rs` - Rust FFI declarations
- `../../REFACTOR_SUMMARY.md` - v1.0 refactoring details

---

**Version**: 1.0.0  
**Language**: Swift 5.9+  
**Platform**: macOS 12.3+  
**License**: MIT/Apache-2.0
