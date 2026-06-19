# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [8.0.0](https://github.com/doom-fish/screencapturekit-rs/compare/v7.0.1...v8.0.0) - 2026-06-19

### Added

- *(async)* implement futures::Stream for AsyncSCStream and recording events
- *(async)* multi-output AsyncSCStream (capture A/V from one stream)
- *(async)* [**breaking**] propagate stop errors in AsyncSCStream; consolidate stop delegate
- *(async)* [**breaking**] make AsyncSCStream lifecycle methods truly async

### Fixed

- *(stream)* surface failed output-handler registration instead of dropping it silently

### Other

- *(deps-dev)* raise bitflags cap to <2.14 (fixes fresh-resolve to 2.13.0)
- *(readme)* update for v8 async API
- *(examples)* add audio+video multi-output showcase; apply rustfmt
- restore Recall.ai sponsor banner and update v6 references to v7

### Changed

- [**breaking**] `AsyncSCStream::start_capture`, `stop_capture`, `update_configuration`,
  and `update_content_filter` are now genuinely async: they return a
  `StreamControlFuture` (resolving to `Result<(), SCError>`) that you `.await`,
  instead of blocking the calling thread on a condition variable. Awaiting them
  parks the task via its `Waker` and resumes from the Swift completion callback,
  so they no longer stall single-threaded/current-thread executors. This makes
  the async surface fully waker-based and consistent with the underlying Swift
  `Task { try await … }` entry points.

  Migration: add `.await` (e.g. `stream.start_capture().await?`). For a
  blocking call, use the synchronous `SCStream` directly via
  `stream.inner().start_capture()`.

- `AsyncSCStream` now installs a stream delegate so that when `ScreenCaptureKit`
  stops the stream with an error (display disconnected, permission revoked, …)
  the sample iterator is closed — `next().await` resolves to `None` instead of
  pending forever — and the error is recorded (see `take_error`). `AsyncSCStream::new`
  likewise no longer silently swallows a failed output-handler registration: it
  closes the stream and records the error.

- The stream engine now dispatches a single canonical stop callback,
  `SCStreamDelegateTrait::did_stop_with_error`, on an error stop. It no longer
  also calls `stream_did_stop` for the same event (the previous behavior fired
  both). `StreamCallbacks::on_stop` keeps working (it is now driven by
  `did_stop_with_error`).

### Added

- `async_api::StreamControlFuture` — the `Send` future returned by the
  `AsyncSCStream` lifecycle methods.
- `AsyncSCStream::take_error` — returns the `SCError` that stopped the stream,
  if any, after `next()` reports the iterator closed.
- Multi-output async capture: `AsyncSCStream::add_output_type` registers an
  additional output type (e.g. add audio to a screen stream), and
  `AsyncSCStream::next_typed` / `try_next_typed` (plus the `NextSampleTyped`
  future) yield each sample together with its `SCStreamOutputType` so audio and
  video can be captured from one stream and told apart.
- `futures::Stream` integration (enabled by the `async` feature, via a new
  optional `futures-core` dependency): `AsyncSCStream::frames` /
  `frames_typed` and `AsyncSCRecordingOutput::events` return
  `futures_core::Stream`s (`SampleStream`, `TypedSampleStream`,
  `RecordingEventStream`) so captures plug into the `StreamExt` combinator
  ecosystem (`map`, `filter`, `take`, `for_each`, `collect`, …).

### Deprecated

- `SCStreamDelegateTrait::stream_did_stop` — `ScreenCaptureKit` only reports
  stops via `did_stop_with_error`, which is now the single source of truth;
  the engine no longer invokes `stream_did_stop`. Implement `did_stop_with_error`
  instead.

## [7.0.1](https://github.com/doom-fish/screencapturekit-rs/compare/v7.0.0...v7.0.1) - 2026-06-06

### Fixed

- resolve safety/FFI findings from deep code review

### Other

- apply rustfmt to satisfy CI formatting check
- *(deps)* update bitflags requirement from >=2.0, <2.12 to >=2.0, <2.13 ([#150](https://github.com/doom-fish/screencapturekit-rs/pull/150))

## [7.0.0](https://github.com/doom-fish/screencapturekit-rs/compare/v6.1.0...v7.0.0) - 2026-05-29

### Added

- add strided pixel render + locked IOSurface CPU view; use ffi_string helper for file_url

### Fixed

- use MaybeUninit for batch FFI scratch buffers

### Other

- cap dev-only bitflags below 2.12 to fix dispatch2 recursion overflow
- Merge ffi/wrappers: sc_retained! macro + null-checked constructors
- consolidate retain/release wrappers via sc_retained! macro and standardize null-checked constructors
- Merge ffi/picker: reclaim observer callback on replacement + consolidate one-shot trampolines
- Merge ffi/screenshot: strided pixel render + ffi_string helper for file_url

## [6.1.0](https://github.com/doom-fish/screencapturekit-rs/compare/v6.0.1...v6.1.0) - 2026-05-29

### Added

- add SCContentFilterBuilder::try_build and fix CI/release flow

### Other

- add comment to CARGO_TERM_COLOR env (no-op)

## [6.0.1] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [6.0.0] - 2026-05-18

### Changed

- [**breaking**] re-export `apple_cf::cm::{CMSampleTimingInfo, CMClock}` from `screencapturekit::cm`, removing the last local Core Media nominal duplicates from the public API
- widen `apple-cf` to `>=0.6, <0.10`

## [5.0.1] - 2026-05-18

### Changed

- derive `Debug` for the remaining public zero-sized helper types and add manual `Debug` impls for closure-backed delegate wrappers

## [5.0.0] - 2026-05-18

### Changed

- [**breaking**] widen `apple-cf` to `>=0.6, <0.9` and migrate all `CGRect` field access to the nested `origin`/`size` layout introduced in `apple-cf` 0.8.0
- the re-exported `apple-metal` dependency now stays on its minimal non-IOSurface surface so `apple-cf` 0.8 can resolve cleanly

## [4.0.0] - 2026-05-18

### Changed

- [**breaking**] `ScreenshotManager::capture_image` now returns `apple_cf::cg::CGImage` (was a private nominal duplicate)
- [**breaking**] `screencapturekit::cm::CMTime` is now a re-export of `apple_cf::cm::CMTime` (was a private nominal duplicate)
- both changes eliminate cross-crate type conversions for downstream consumers chaining `ScreenCaptureKit` → `ImageIO` / `VideoToolbox` / related Apple frameworks

## [3.1.4] - 2026-05-17

### Fixed

- *(async)* wrap all `extern "C"` completion callbacks in `catch_user_panic` to prevent panic UB across the FFI boundary
- *(unsafe)* add `SAFETY:` comments to `unsafe impl Send/Sync` blocks and all unsafe blocks inside FFI callbacks
- *(unsafe)* add `SAFETY:` comments to FFI-call unsafe blocks in async API impl methods
- *(docs)* correct install version in `README.md` and `src/lib.rs` (was 1/2, now 3)

## [3.1.3](https://github.com/doom-fish/screencapturekit-rs/compare/v3.1.2...v3.1.3) - 2026-05-17

### Other

- *(docs)* pin Documentation workflow to macos-26
- widen apple-metal to <0.9 (use v0.8.0 with @available guards)

## [3.1.2](https://github.com/doom-fish/screencapturekit-rs/compare/v3.1.1...v3.1.2) - 2026-05-17

### Fixed

- *(docs)* backtick ScreenCaptureKit in README to satisfy doc-markdown
- *(docs)* drop non-existent com.apple.security.screen-capture entitlement ([#144](https://github.com/doom-fish/screencapturekit-rs/pull/144))
- *(stream-config)* pin BGRA as default pixel format ([#145](https://github.com/doom-fish/screencapturekit-rs/pull/145))

### Other

- *(deps)* publish-friendly Cargo.toml; move sibling paths to local .cargo/config
- add explicit CodeQL workflow on macos-26 (Rust-only)
- pin apple-metal-rs sibling to v0.7.1 + restrict matrix to macos-26
- drop macos-14 from build + leak-check matrix
- *(siblings)* pin apple-cf/apple-metal sibling clones to v0.6.x
- *(lint)* run on macos-26 so --all-features can compile macos_26_0 code
- *(workflow)* clone sibling path-dependency repos before build
- widen apple-cf / apple-metal version constraints
- fmt + audit-v2 follow-ups

## [3.1.1](https://github.com/doom-fish/screencapturekit-rs/compare/v3.1.0...v3.1.1) - 2026-05-16

### Fixed

- *(error)* correct `SC_STREAM_ERROR_DOMAIN` to match Apple's exported `SCStreamErrorDomain` constant

## [3.1.0](https://github.com/doom-fish/screencapturekit-rs/compare/v3.0.1...v3.1.0) - 2026-05-16

### Added

- *(coverage)* add `COVERAGE.md` and certify the audited `ScreenCaptureKit` surface
- *(picker)* add `SCContentSharingPickerConfiguration::allowed_picker_modes()`
- *(recording)* add `SCRecordingOutputConfiguration::output_url()`
- *(config)* add `SCStreamConfiguration::color_space_name()` and RGBA background-color accessors

### Fixed

- *(preset)* map `CaptureHDRRecordingPreservedSDRHDR10` to Apple's macOS 26 HDR-recording preset
- *(config)* retain assigned `CGColorRef` / `CFStringRef` values for `backgroundColor`, `colorSpaceName`, and `colorMatrix`
- *(config)* round-trip `SCStreamConfiguration` color and preset properties in tests
- *(deps)* align local `apple-cf` / `apple-metal` version bounds with 0.6
- *(examples)* import `IOSurfaceMetalExt` where `apple-metal` 0.6 requires the trait in scope

## [2.1.0](https://github.com/doom-fish/screencapturekit-rs/compare/v2.0.0...v2.1.0) - 2026-05-11

### Added

- *(screenshot)* add rgba_data_into / bgra_data_into for buffer reuse

### Fixed

- *(test)* make data_into test robust to non-deterministic capture content
- *(docs)* wrap README batched-API code samples for doctest
- *(lint)* add backticks + Eq derives to satisfy clippy -D warnings
- *(examples)* 03_audio_capture only registered Screen handler

### Other

- *(ffi-string)* stack-allocate the buffer for small strings
- migrate 02_window_capture + 14_app_capture to snapshot(); document batched APIs in README
- *(examples)* migrate 07_list_content to snapshot() + add 24 showcase
- *(perf)* add live capture profiling driver + symbolicate scripts
- *(screenshot)* add native-BGRA fast path skipping channel swap
- *(bench)* add audio+video throughput + audio_buffer_list micro-bench
- batched FFI + zero-init removal across hot paths

## [2.0.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.5.4...v2.0.0) - 2026-05-06

### Added

- *(picker)* expose SCContentSharingPicker.isActive getter and setter
- *(picker)* expose SCContentSharingPicker.defaultConfiguration
- *(cm)* expose SCStreamFrameInfo.presenterOverlayContentRect attachment
- *(pixel-format)* [**breaking**] surface unrecognised codes via PixelFormat::Unknown
- *(error)* [**breaking**] mark SCStreamErrorCode as #[non_exhaustive]

### Fixed

- *(ci)* use release-plz prebuilt action instead of cargo install
- *(build)* force `--sdk macosx` in xcrun invocation to bypass stale CLT defaults
- *(pixel-format)* [**breaking**] normalise PartialEq/Hash through FourCharCode
- *(panic-safety)* wrap user code in remaining extern "C" callbacks
- *(build)* [**breaking**] propagate every macos_* feature; fail loudly on SDK detection failure
- *(build)* improve rebuild triggers and xcode-select error visibility
- *(ffi)* defend FFI string helpers against malformed Swift output
- *(stream)* [**breaking**] require Send + Sync on output and delegate traits
- *(completion)* atomic guard against double-invocation; lost-wakeup race in poll
- *(async)* lost-wakeup race in NextSample/NextRecordingEvent polls; clippy
- *(swift)* correct C-header drift; document dispatch-queue affinity deviation
- *(stream)* panic-safety, lock contention, and hot-path allocation in sample dispatch

### Other

- *(deps)* upgrade dev-dependencies to current major versions
- *(swift-bridge)* re-enable SwiftLint force_unwrapping rule
- *(examples)* add Tauri entitlements + warn 23_client_server is toy IPC
- *(examples)* replace permission-sensitive panics with graceful skip messages
- *(api)* clarify hot-path costs, lifetime conventions, and prelude omissions
- route permission-skip messages to stderr with SKIP: prefix (M12)
- cover YUV pixel format and mid-capture handler lifecycle (M11)
- *(bench)* rewrite frame-throughput bench; add stream-startup bench
- end-to-end cross-stream isolation regression test for #135

## [1.5.4](https://github.com/doom-fish/screencapturekit-rs/compare/v1.5.3...v1.5.4) - 2026-03-09

### Fixed

- per-stream callback routing to prevent cross-stream sample leaking ([#135](https://github.com/doom-fish/screencapturekit-rs/pull/135)) ([#136](https://github.com/doom-fish/screencapturekit-rs/pull/136))

## [1.5.3](https://github.com/doom-fish/screencapturekit-rs/compare/v1.5.2...v1.5.3) - 2026-03-05

### Fixed

- skip native build on docs.rs to fix badge

### Other

- extract link_swift_bridge to fix clippy too-many-lines

## [1.5.2](https://github.com/doom-fish/screencapturekit-rs/compare/v1.5.1...v1.5.2) - 2026-03-05

### Fixed

- use pointer::cast for clippy ptr_as_ptr lint
- use system allocator for Swift-allocated AudioBufferList deallocation ([#129](https://github.com/doom-fish/screencapturekit-rs/pull/129))
- use system allocator for Swift-allocated AudioBufferList deallocation ([#132](https://github.com/doom-fish/screencapturekit-rs/pull/132))
- cross-arch swift build and CI symbol check reliability ([#131](https://github.com/doom-fish/screencapturekit-rs/pull/131))

### Other

- add allocator mismatch regression test for AudioBufferList
- Revert "fix: use system allocator for Swift-allocated AudioBufferList deallocation ([#132](https://github.com/doom-fish/screencapturekit-rs/pull/132))"

## [1.5.1](https://github.com/doom-fish/screencapturekit-rs/compare/v1.5.0...v1.5.1) - 2026-02-25

### Fixed

- use cargo features instead of macOS SDK version ([#126](https://github.com/doom-fish/screencapturekit-rs/pull/126))

### Other

- *(deps)* update eframe requirement from 0.30 to 0.33 ([#121](https://github.com/doom-fish/screencapturekit-rs/pull/121))

## [1.5.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.4.2...v1.5.0) - 2025-12-20

### Added

- add WebRTC screen streaming example
- *(examples)* add full wgpu screen capture viewer
- *(examples)* add full egui viewer with eframe

### Fixed

- remove tauri build artifacts from git tracking
- *(examples)* simplify tauri example shader and config
- *(tests)* remove unnecessary unsafe block
- *(lint)* resolve clippy warnings and update API naming
- *(lint)* resolve clippy doc_markdown warnings
- *(examples)* correct wgpu row stride and sRGB color

### Other

- update .gitignore for tauri example build artifacts
- remove RwLock+Sync changes, keep Mutex for handler registry
- fix README code block rendering on GitHub
- fix README code block rendering on GitHub
- fix clippy doc_markdown warning for IOSurface
- rename SCContentFilter::with() to create() and add audio enums
- update SCContentFilter API calls and benchmark label
- fix failing doctests
- enhance Cargo.toml with lints, badges, and example features
- update README with examples 18-23 and fix module structure
- *(examples)* remove unused list_applications from tauri example
- *(examples)* remove unused RecordingState from tauri example
- *(examples)* update tauri README to match current code
- format tauri example code
- add crates.io badges and egui/bevy examples
- add Used By section, FUNDING.yml, and integration examples
- *(deps)* update criterion requirement from 0.5 to 0.8 ([#117](https://github.com/doom-fish/screencapturekit-rs/pull/117))

## [1.4.2](https://github.com/doom-fish/screencapturekit-rs/compare/v1.4.1...v1.4.2) - 2025-12-15

### Fixed

- remove whitespace between badges

### Other

- change example sample rate that isn't 48000 to be 24000 (which is actually valid) ([#116](https://github.com/doom-fish/screencapturekit-rs/pull/116))
- update README badges to match LazyVim style

## [1.4.1](https://github.com/doom-fish/screencapturekit-rs/compare/v1.4.0...v1.4.1) - 2025-12-11

### Fixed

- correct include_applications doctest to pass required second argument

### Other

- improve lib.rs documentation with more examples and patterns
- simplify window title comparison in example
- add missing examples 16 and 17 to lib.rs

## [1.4.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.3.0...v1.4.0) - 2025-12-11

### Added

- *(iosurface)* add plane access methods to lock guard
- add IOSurface::create_with_properties for multi-planar surfaces
- *(examples)* improve metal app VU meter and add texture example
- *(cm)* add comprehensive CMBlockBuffer API with cursor support
- *(example)* demonstrate IOSurface introspection APIs
- expand IOSurface API and add async improvements
- *(metal)* add vertex descriptor and UI helpers
- *(cm)* add Debug and Clone impls for CoreMedia types
- *(recording,screenshot)* add Debug impls
- *(stream)* add delegate registry with reference counting
- *(async)* add Debug impls for async types
- add Metal/IOSurface helpers and audio format improvements
- *(screenshot)* add multi-format image saving support
- *(cm)* add frame info accessors to CMSampleBuffer

### Fixed

- *(tests)* update import after completion rename
- *(docs)* use ignore for async feature-gated doctest
- *(error)* correct SCStreamErrorCode values to match Apple SDK
- *(recording)* use ref-counted delegate registry for SCRecordingOutput
- *(example)* add window fallback and CG init to 02_window_capture
- *(example)* handle NaN in waveform vertex calculation
- *(example)* handle NaN in waveform peak calculation
- *(stream)* clone handlers when cloning SCStream
- *(stream)* increment delegate ref count on clone
- *(filter)* remove Default impl and panic on empty builder

### Other

- remove demo.mp4 from git tracking (now hosted externally)
- Update README.md
- Add GitHub asset link to README
- Update README.md
- update
- update video
- move demo video before TOC
- add 15s high quality demo video
- use GIF for README demo (GitHub compatibility)
- add demo video to README
- improve module documentation with examples and tables
- reorganize pixel buffer and IOSurface modules
- add tokio async/await tests for FFI callbacks
- add content sharing picker tests
- add async future polling and version-specific tests
- add async stream capture tests
- add async stream lifecycle tests
- add Metal vertex descriptor and render encoder tests
- expand async stream tests
- add Metal layer, render pass, buffer, and closure API tests
- add comprehensive Metal device and texture tests
- expand async API and metal tests
- add IOSurface creation API and comprehensive tests
- add comprehensive tests for CMBlockBuffer, pixel buffer, and async API
- cancel in-progress runs on new push to same branch
- *(tests)* extract inline tests from metal.rs to tests/
- remove macos-14-intel from CI matrix
- *(cm)* remove unused SCStreamFrameInfoKey
- *(utils)* rename sync_completion to completion
- *(cg)* split into separate modules
- use macos-14-intel instead of macos-14-large
- add cargo fmt to testing instructions
- apply cargo fmt formatting
- use macos-15-intel label, remove macos-26 Intel (not available)
- update matrix to include Intel/ARM for macOS 14+
- *(deps)* update winit to 0.30 and raw-window-handle to 0.6
- fix formatting issues
- *(examples)* improve full_metal_app example
- *(swift)* minor formatting cleanup
- remove unused dependency
- *(async_api)* convert doc examples from ignore to no_run
- *(content_sharing_picker)* convert doc examples from ignore to no_run
- *(metal)* convert doc examples from ignore to no_run
- *(screenshot_manager)* convert doc examples from ignore to no_run
- *(configuration)* convert doc examples from ignore to no_run
- *(sync_completion)* convert doc example from ignore to no_run
- update README to use macos_26_0 as latest feature
- fix rustfmt formatting issues
- *(tests)* rename sync_completion_tests to completion_tests

## [1.3.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.2.0...v1.3.0) - 2025-11-30

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
- *(config)* add builder pattern with ::new() and with_* methods

### Fixed

- improve leak count parsing for 'Process N: X leaks' format
- *(async)* fix memory leak in AsyncCompletion and add leak check to CI
- *(recording)* wait for recording_did_finish before opening file
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

- ignore Apple framework leaks, fail only on our code leaks
- fail leak check on any leaks including Apple framework leaks
- run leak-check on all platforms in matrix
- run leak-check parallel to build, gate releases, move docs to release trigger
- fix formatting in leak check example
- Remove non-existent SCStreamConfiguration APIs
- add 16_full_metal_app to examples list
- fix content picker API examples in README
- rename 16_metal_overlay to 16_full_metal_app
- rename metal_overlay example to 16_metal_overlay
- update lib.rs feature flags and examples list
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
- *(swift-bridge)* standardize API patterns and error handling
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
