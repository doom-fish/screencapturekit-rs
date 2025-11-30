# Copilot Instructions for screencapturekit-rs

This project provides safe, idiomatic Rust bindings for macOS ScreenCaptureKit framework.

## Project Overview

- **Language:** Rust with Swift bridge code
- **Platform:** macOS 12.3+ (with feature flags for newer APIs)
- **Architecture:** FFI bindings via `swift-bridge/` to ScreenCaptureKit

## Key Directories

```
src/                     # Rust source code
├── stream/              # SCStream, SCStreamConfiguration, SCContentFilter
├── shareable_content/   # SCShareableContent, SCDisplay, SCWindow
├── cm/                  # Core Media types (CMSampleBuffer, CMTime)
├── cg/                  # Core Graphics types (CGRect, CGImage)
├── output/              # Frame output handling
├── screenshot_manager/  # SCScreenshotManager (macOS 14.0+)
├── content_sharing_picker/ # SCContentSharingPicker (macOS 14.0+)
├── recording_output/    # SCRecordingOutput (macOS 15.0+)
├── async_api/           # Async wrappers
└── ffi/                 # Raw FFI declarations

swift-bridge/            # Swift implementation bridging to ScreenCaptureKit
docs/apple/              # Apple documentation (gitignored, regenerate with scripts/download_apple_docs.py)
```

## Apple ScreenCaptureKit API Reference

**Source this file for complete API signatures:** `docs/apple/API-COMPLETE.md`

To regenerate documentation: `python3 scripts/download_apple_docs.py`

### Core Classes

```swift
// SCStream - Main capture stream
class SCStream
init(filter: SCContentFilter, configuration: SCStreamConfiguration, delegate: SCStreamDelegate?)
func startCapture() async throws
func stopCapture() async throws
func addStreamOutput(_ output: SCStreamOutput, type: SCStreamOutputType, sampleHandlerQueue: DispatchQueue?) throws
func removeStreamOutput(_ output: SCStreamOutput, type: SCStreamOutputType) throws
func updateConfiguration(_ config: SCStreamConfiguration) async throws
func updateContentFilter(_ filter: SCContentFilter) async throws
func addRecordingOutput(_ output: SCRecordingOutput) throws
func removeRecordingOutput(_ output: SCRecordingOutput) throws

// SCStreamConfiguration - Stream settings
class SCStreamConfiguration
var width: Int { get set }
var height: Int { get set }
var pixelFormat: OSType { get set }
var colorSpaceName: CFString { get set }
var showsCursor: Bool { get set }
var capturesAudio: Bool { get set }
var sampleRate: Int { get set }
var channelCount: Int { get set }
var minimumFrameInterval: CMTime { get set }
var queueDepth: Int { get set }
var scalesToFit: Bool { get set }
var sourceRect: CGRect { get set }
var destinationRect: CGRect { get set }
var captureDynamicRange: SCCaptureDynamicRange { get set }  // macOS 15.0+
var captureMicrophone: Bool { get set }                      // macOS 15.0+
var presenterOverlayPrivacyAlertSetting: SCPresenterOverlayAlertSetting { get set }
convenience init(preset: SCStreamConfiguration.Preset)

// SCContentFilter - What to capture
class SCContentFilter
init(desktopIndependentWindow: SCWindow)
init(display: SCDisplay, including: [SCWindow])
init(display: SCDisplay, excludingWindows: [SCWindow])
init(display: SCDisplay, including: [SCRunningApplication], exceptingWindows: [SCWindow])
init(display: SCDisplay, excludingApplications: [SCRunningApplication], exceptingWindows: [SCWindow])
var contentRect: CGRect { get }
var pointPixelScale: Float { get }
var includeMenuBar: Bool { get set }

// SCShareableContent - Query available content
class SCShareableContent
class func getExcludingDesktopWindows(_ excludeDesktop: Bool, onScreenWindowsOnly: Bool) async throws -> SCShareableContent
class func getCurrentProcessShareableContent() async throws -> SCShareableContent  // macOS 14.4+
var displays: [SCDisplay] { get }
var windows: [SCWindow] { get }
var applications: [SCRunningApplication] { get }

// SCDisplay
class SCDisplay
var displayID: CGDirectDisplayID { get }
var width: Int { get }
var height: Int { get }
var frame: CGRect { get }

// SCWindow
class SCWindow
var windowID: CGWindowID { get }
var title: String? { get }
var owningApplication: SCRunningApplication? { get }
var frame: CGRect { get }
var isOnScreen: Bool { get }
var windowLayer: Int { get }

// SCRunningApplication
class SCRunningApplication
var bundleIdentifier: String { get }
var applicationName: String { get }
var processID: pid_t { get }
```

### Screenshot API (macOS 14.0+)

```swift
class SCScreenshotManager
class func captureSampleBuffer(contentFilter: SCContentFilter, configuration: SCStreamConfiguration) async throws -> CMSampleBuffer
class func captureImage(contentFilter: SCContentFilter, configuration: SCStreamConfiguration) async throws -> CGImage
class func captureImage(inRect: CGRect) async throws -> CGImage  // macOS 15.2+
```

### Recording API (macOS 15.0+)

```swift
class SCRecordingOutput
init(configuration: SCRecordingOutputConfiguration, delegate: SCRecordingOutputDelegate)
var recordedDuration: CMTime { get }
var recordedFileSize: Int { get }

class SCRecordingOutputConfiguration
var outputURL: URL { get set }
var outputFileType: AVFileType { get set }
var videoCodecType: AVVideoCodecType { get set }

protocol SCRecordingOutputDelegate
func recordingOutputDidStartRecording(_ output: SCRecordingOutput)
func recordingOutputDidFinishRecording(_ output: SCRecordingOutput)
func recordingOutput(_ output: SCRecordingOutput, didFailWithError: Error)
```

### Content Picker (macOS 14.0+)

```swift
class SCContentSharingPicker
class var shared: SCContentSharingPicker { get }
var isActive: Bool { get set }
func present()
func present(for stream: SCStream?, using contentStyle: SCShareableContentStyle)
func add(_ observer: SCContentSharingPickerObserver)
func remove(_ observer: SCContentSharingPickerObserver)

protocol SCContentSharingPickerObserver
func contentSharingPicker(_ picker: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for stream: SCStream?)
func contentSharingPicker(_ picker: SCContentSharingPicker, didCancelFor stream: SCStream?)
func contentSharingPickerStartDidFail(with error: Error)
```

### Protocols

```swift
protocol SCStreamOutput
func stream(_ stream: SCStream, didOutputSampleBuffer: CMSampleBuffer, of type: SCStreamOutputType)

protocol SCStreamDelegate
func stream(_ stream: SCStream, didStopWithError: Error)
func outputEffectDidStart(for stream: SCStream)  // Presenter Overlay
func outputEffectDidStop(for stream: SCStream)

enum SCStreamOutputType { case screen, audio, microphone }
enum SCCaptureDynamicRange { case sdr, hdrLocalDisplay, hdrCanonicalDisplay }
enum SCPresenterOverlayAlertSetting { case system, never, always }
```

## Coding Conventions

### Builder Pattern

```rust
// Content filters use .builder() with .build()
let filter = SCContentFilter::builder()
    .display(&display)
    .exclude_windows(&windows)
    .build();

// Configuration uses ::new() with .with_*() chainable methods
let config = SCStreamConfiguration::new()
    .with_width(1920)
    .with_height(1080)
    .with_pixel_format(PixelFormat::BGRA);
```

### Feature Flags

Version-gated APIs require feature flags:

```rust
#[cfg(feature = "macos_15_0")]
pub fn set_capture_dynamic_range(&mut self, range: SCCaptureDynamicRange) -> &mut Self {
    // ...
}
```

Feature hierarchy (cumulative):
- `macos_13_0` - Audio capture, sync clock
- `macos_14_0` - Content picker, screenshots
- `macos_14_2` - Menu bar, child windows, presenter overlay
- `macos_14_4` - Current process content
- `macos_15_0` - Recording output, HDR, microphone
- `macos_15_2` - Screenshot in rect
- `macos_26_0` - Advanced screenshot config

### Memory Management

- All CoreFoundation types use RAII with proper retain/release in Drop
- FFI functions that transfer ownership are named `*_create`, `*_copy`
- FFI functions that borrow are named `*_get_*`
- Always pair `*_retain` with `*_release`

### Error Handling

- Use `SCError` for all error types
- Map Swift/ObjC errors to appropriate variants
- Include context in error messages

## Testing

```bash
cargo test                           # All tests
cargo test --features async          # With async
cargo test --all-features            # All features
cargo clippy --all-features -- -D warnings  # Lint
```

## Updating Apple Documentation

To regenerate the full Apple documentation:

```bash
python3 scripts/download_apple_docs.py
```

This downloads:
1. JSON documentation from Apple's developer docs
2. Converts to markdown for easy reference
3. Extracts Swift sample code from Apple's sample projects
4. WWDC session transcripts with code snippets
5. API-COMPLETE.md with all function signatures

## Common Tasks

### Adding a New API

1. Check `docs/apple/API-COMPLETE.md` for Swift signatures
2. Check `docs/apple/wwdc/` for WWDC session explaining the feature
3. Reference `docs/apple/samples/` for Swift usage patterns
4. Add FFI declaration in `src/ffi/mod.rs`
5. Implement Swift bridge in `swift-bridge/Sources/`
6. Create Rust wrapper with builder pattern
7. Add feature flag if version-specific
8. Add tests and update `CHANGELOG.md`

### Mapping Swift to Rust

| Swift | Rust |
|-------|------|
| `class SCStream` | `struct SCStream { ptr: *const c_void }` |
| `@escaping closure` | `Box<dyn Fn(...) + Send>` |
| `async throws` | `Result<T, SCError>` |
| `Optional<T>` | `Option<T>` |
| `NSError` | `SCError` |
