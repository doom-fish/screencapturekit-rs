# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2024-02-06

### Fixed

- [#37](https://github.com/svtlabs/screencapturekit-rs/issues/37) Fix error handling
- [#35](https://github.com/svtlabs/screencapturekit-rs/issues/35) Make safe StreamConfiguration defaults equal to unsafe

## [0.2.5] - 2024-01-25

### Fixed

- 0.2.4 had an issue with the configuration. This is fixed now.

## [0.2.4] - 2024-01-23

### Fixed

- [#34](https://github.com/svtlabs/screencapturekit-rs/issues/34) minimum_frame_interval not working as expected
- [#33](https://github.com/svtlabs/screencapturekit-rs/issues/33) Can no longer import SCFrameStatus

## [0.2.3] - 2024-01-15

### Fixed

- [#32](https://github.com/svtlabs/screencapturekit-rs/pull/32): feat: add support for pixel_format and source_rect

## [0.2.2] - 2024-01-15

### Fixed

- [#31](https://github.com/svtlabs/screencapturekit-rs/pull/31): Fix segfault due to bad refcount

## [0.2.1] - 2023-12-15

### Fixed

- Cargo dependency mistake

## [0.2.0] - 2023-12-15

### Added

- [#13](https://github.com/svtlabs/screencapturekit-rs/pull/13): Support audio stream
- [#10](https://github.com/svtlabs/screencapturekit-rs/pull/10): Allow for audio output
- [#9](https://github.com/svtlabs/screencapturekit-rs/pull/9): High level example
- Additional examples
- Expand CMSampleBuffer API

### Fixed

- [#14](https://github.com/svtlabs/screencapturekit-rs/pull/14): Don't panic on new_completion_handler
- [#7](https://github.com/svtlabs/screencapturekit-rs/pull/7): Fix typo in UnsafeSCWindow

## [0.1.0] - 2023-08-21

### Added

- Initial commit with prototype version

[unreleased]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.6...HEAD
[0.2.6]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/svtlabs/screencapturekit-rs/compare/v0.1.0...v0.2.0
[0.0.1]: https://github.com/svtlabs/screencapturekit-rs/releases/tag/v0.1.0
