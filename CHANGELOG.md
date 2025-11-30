# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.2.0...v2.0.0) - 2025-11-30

### Added

- *(examples)* enhance memory leak test with comprehensive API coverage
- *(ffi)* add optimized batch retrieval and owned strings
- *(examples)* add missing API coverage examples
- *(example)* refactor metal_overlay with recording/screenshot modules
- *(ffi)* add FFI bindings for new APIs
- *(picker)* add presentation mode and single window style
- *(cm)* add CMTime operations and frame status predicates
- *(stream)* add sync clock and filter stream type
- *(config)* add capture resolution type support (macOS 14.0+)
- *(filter)* add excluding applications filter variant
- *(screenshot)* add SCScreenshotConfiguration content type support
- *(async)* add async screenshot APIs for macOS 15.2/26.0
- *(stream)* add closure-based delegate builders
- *(example)* add recording menu option (macOS 15.0+)
- *(example)* use macOS 26 screenshot API when available
- *(cg)* add CGImage::save_png() with auto-open
- *(example)* add screenshot option to metal_overlay
- *(example)* enable mic-only capture without video source
- *(example)* enhance metal_overlay with synthwave UI
- *(stream)* expose as_ptr for internal use
- *(picker)* add show_for_stream methods
- *(audio)* add audio buffer list access
- *(picker)* add SCPickedSource to identify selected content type
- *(audio)* add audio input device enumeration API
- *(examples)* use SCContentSharingPicker for content selection
- *(examples)* preserve aspect ratio and center UI panels
- *(examples)* add real audio waveform and vertical gain meters
- *(shareable_content)* add missing SDK methods
- *(screenshot)* add HDR screenshot example and tests
- *(screenshot)* add macOS 26.0 advanced screenshot APIs
- *(async)* add async picker example and tests
- *(async)* add AsyncSCContentSharingPicker for non-blocking picker UI
- *(recording)* add full video codec and file type arrays
- *(config)* add CaptureHDRRecordingPreservedSDRHDR10 preset
- *(delegate)* add stream active/inactive callbacks (macOS 15.2+)
- *(examples)* add metal_overlay example with GPU rendering
- *(examples)* integrate new macOS 14.0-15.2 features into examples
- *(picker)* add SCContentSharingPicker enhancements for macOS 14.0+
- *(recording)* add SCRecordingOutput features for macOS 15.0+
- *(screenshot)* add capture_image_in_rect for macOS 15.2+
- *(content)* add SCShareableContentInfo for macOS 14.0+
- *(filter)* add SCContentFilter properties for macOS 14.0-15.2
- *(config)* add new SCStreamConfiguration options for macOS 14.0-15.0
- *(error)* add new error types for macOS 14.0+ stream events
- *(config)* [**breaking**] add builder pattern with ::new() and with_* methods

### Fixed

- add macos_13_0 feature gate to synchronization_clock in example
- add macos_14_0 feature gate to captures_shadows_only tests
- add feature gates to example for content_rect, point_pixel_scale, and Microphone
- use rawValue comparison for SCStreamOutputType to fix SDK compatibility
- add SCREENCAPTUREKIT_HAS_MACOS26_SDK guard for macOS 26+ APIs
- add macOS version availability guards for Swift APIs
- guard recording output FFI with compiler version check for macOS 13
- remove non-existent with_average_bitrate method calls
- resolve clippy warnings
- remove non-existent average_bitrate API
- *(swift)* implement stub configuration properties
- *(swift)* implement stub configuration properties
- remove dead code and fix test errors
- *(memory)* add leak fixes and comprehensive memory tests
- *(audio)* correct AudioStreamBasicDescription repr
- *(picker)* activate app and cleanup observer before showing
- *(examples)* use SCShareableContent instead of blocking picker
- *(examples)* update recording_output example to use builder pattern
- *(picker)* remove tokio-dependent async methods

### Other

- update README to match current API
- add macOS 26 (Tahoe) to build matrix
- fix formatting in memory leak check example
- improve doc comments for audio and block buffer modules
- update documentation to reflect current API
- update references to renamed memory leak example
- *(test)* convert leak test to example for better isolation
- add note about running memory tests single-threaded
- fix clippy warnings and format code
- *(swift)* use public APIs for filter content extraction
- *(examples)* update README with new examples
- *(examples)* extract input handling to separate module
- improve code quality and fix memory leaks
- add copilot instructions and Apple docs download script
- *(error)* replace specific error variants with SCStreamErrorCode
- *(example)* split metal_overlay into modules
- *(example)* remove unused metal_overlay module files
- *(config)* remove get_ prefix from getters
- *(example)* modularize metal_overlay example
- improve metal overlay UX and mic device API
- *(example)* improve metal_overlay UX with menu navigation
- *(picker)* rename async pick() to show() for consistency
- *(picker)* replace blocking API with callback-based API
- add coverage for missing SDK features
- *(api)* remove get_ prefix from non-configuration getters
- fix clippy warnings in docs and tests
- *(cm)* standardize CM/CV type method naming
- *(api)* remove deprecated APIs and standardize naming
- *(api)* standardize Rust API patterns and naming
- *(swift-bridge)* [**breaking**] standardize API patterns and error handling
- add version-specific feature testing per macOS runner
- update README with new APIs and feature flags
- *(picker)* add async API documentation and improve examples
- update README and Cargo.toml for new features
- fix clippy doc_markdown warnings and update examples
- update documentation to use builder pattern API
- fix build badge workflow filename
- use major version only in README examples

### Added

- **Memory Leak Example** - `15_memory_leak_check.rs` for comprehensive memory leak testing with `leaks`
- **New Examples** - `12_stream_updates.rs`, `13_advanced_config.rs`, `14_app_capture.rs`
- **Batch FFI Retrieval** - Optimized batch retrieval functions for displays, windows, and applications
- **Comprehensive Memory Tests** - Tests covering audio, microphone, filters, and all content types

### Changed

- **Input Module** - Extracted input handling from metal_overlay to reusable module
- **Swift FFI Boundary** - Moved parsing/validation logic to Swift side for cleaner Rust code
- **Owned Strings** - Changed FFI string functions to return owned strings where appropriate

### Fixed

- Memory leak fixes in stream lifecycle and content filter creation
- Removed dead code and stub implementations
- Fixed clippy warnings across codebase
- Corrected non-existent API method calls

## [1.2.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.1.0...v1.2.0) - 2025-11-28

### Added

- implement synchronous error propagation for stream operations
- add AsyncSCScreenshotManager for async screenshot capture

### Fixed

- add macOS 14.0 availability check for updateConfiguration
- *(ffi)* add backticks to doc comments for clippy
- *(tests)* initialize CoreGraphics for headless CI environments
- resolve clippy warnings with --all-features
- add semicolon to satisfy clippy lint
- resolve all clippy warnings
- use correct FFI function for async shareable content

### Other

- separate lint job from per-platform build matrix
- fix formatting issues
- replace builder pattern with mutable configuration
- update content_sharing_picker to use SyncCompletion
- unify async completion patterns with AsyncCompletion<T>
- unify sync completion patterns across codebase
- fix formatting issues
- consolidate workflows into single CI pipeline
- replace mpsc channel with Mutex+Condvar in content sharing picker
- replace mpsc channel with Mutex+Condvar in screenshot manager
- update version to 1.1 in README

## [1.1.0] - 2025-11-28

### Changed

- **BREAKING**: `SCContentFilter::build()` renamed to `builder()` for consistency
- **BREAKING**: Config setters now return `Self` directly instead of `&mut Self` for cleaner builder pattern

### Fixed

- Documentation fixes for builder pattern API

## [1.0.0] - 2025-11-27

### Added

- **New Builder API** - `SCStreamConfiguration::builder()` with fluent chainable methods
- **Async API** - Runtime-agnostic async support with `AsyncSCStream` and `AsyncSCShareableContent`
- **Closure Handlers** - Support for closure-based output handlers in addition to trait implementations
- **Recording Output** - Direct video file recording with `SCRecordingOutput` (macOS 15.0+)
- **Content Sharing Picker** - System UI for content selection with `SCContentSharingPicker` (macOS 14.0+)
- **Screenshot Manager** - Single-frame capture with `SCScreenshotManager` (macOS 14.0+)
- **IOSurface Access** - Zero-copy GPU texture access for Metal/OpenGL integration
- **Custom Dispatch Queues** - Control callback threading with QoS levels
- **HDR Capture** - `SCCaptureDynamicRange` for HDR/SDR modes (macOS 15.0+)
- **Microphone Capture** - Audio input capture support (macOS 15.0+)
- **Feature Flags** - Granular macOS version feature gates (`macos_13_0` through `macos_15_0`)

### Changed

- **BREAKING**: `SCStreamConfiguration::build()` is now deprecated, use `builder()` instead
- Improved memory management with proper reference counting
- Swift bridge build output now goes to `OUT_DIR` for clean `cargo publish`
- Comprehensive documentation and examples

### Fixed

- Memory leaks in stream lifecycle
- Double-free issues in SCStream
- Thread safety improvements

## [0.3.6](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.5...v0.3.6) - 2025-08-04

### Added

- Get and set sample rate via SCStreamConfiguration ([#94](https://github.com/doom-fish/screencapturekit-rs/pull/94))

### Other

- *(deps)* update core-graphics requirement from 0.24 to 0.25 ([#92](https://github.com/doom-fish/screencapturekit-rs/pull/92))
- workflows
- Update CHANGELOG.md
- Delete .github/workflows/contrib.yml

## [0.3.5](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.4...v0.3.5) - 2025-02-06

### Other

- fix releaze action
- fix bad cargo.toml

## [0.3.4](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.3...v0.3.4) - 2025-01-29

### Other

- chore(contributors) update contrib ([#80](https://github.com/doom-fish/screencapturekit-rs/pull/80))

## [0.3.3](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.2...v0.3.3) - 2025-01-29

### Added

- add showsCursor configuration option (#72)

### Other

- Fix scstream double free ([#74](https://github.com/doom-fish/screencapturekit-rs/pull/74))
- *(deps)* update block2 requirement from 0.5 to 0.6 (#75)

## [0.3.2](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.1...v0.3.2) - 2024-12-19

### Added

- add Send trait for SCShareableContent (#59)
- add screenshot manager capture (#58)
- add configuration options for captured frames (#57)

## [0.3.1](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.0...v0.3.1) - 2024-11-29

### Other

- remove old changelog
