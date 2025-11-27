# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/doom-fish/screencapturekit-rs/compare/v0.4.0...v0.5.0) - 2025-11-27

### Added

- *(swift-bridge)* add SCBridgeError enum and documentation

### Other

- *(swift-bridge)* update README to reflect modular structure
- [**breaking**] add 1.0.0 changelog
- restore Recall.ai sponsor banner

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
