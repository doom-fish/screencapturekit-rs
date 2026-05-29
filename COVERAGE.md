# ScreenCaptureKit SDK Coverage

> **Snapshot — not a live coverage status.** This document records a point-in-time
> audit performed against `screencapturekit` **v3.1.1**. It has **not** been
> re-verified against the current crate version (6.x) and should be read as a
> historical audit, not an up-to-date certification.

This document records the `screencapturekit` v3.1.1 coverage audit against Apple's `ScreenCaptureKit.framework` from Xcode 26.2 (`MacOSX26.2.sdk`).

## Certification

As of `screencapturekit` **3.1.1**, the crate provides **100% coverage of the audited ScreenCaptureKit SDK surface for this pass**, including the newer macOS 15.x and 26.0-era APIs requested for review.

That coverage includes both:

- direct bindings for concrete classes, enums, and properties
- safe Rust equivalents where Apple's surface is protocol- or attachment-key-oriented

## Audited surface

The audit covered the following public SDK areas:

- `SCStream`
- `SCStreamConfiguration`
- `SCContentFilter`
- `SCShareableContent`
- `SCShareableContentInfo`
- `SCRunningApplication`
- `SCDisplay`
- `SCWindow`
- `SCContentSharingPicker`
- `SCContentSharingPickerConfiguration`
- `SCContentSharingPickerMode`
- `SCRecordingOutput`
- `SCRecordingOutputConfiguration`
- `SCStreamErrorCode`
- `SCStreamErrorDomain`
- newer macOS 15.x / 26.0 additions in `SCScreenshotManager` and preset APIs

## Coverage map

| Apple SDK surface | Rust coverage | Status |
| --- | --- | --- |
| `SCStream`, `SCStreamDelegate`, `SCStreamOutput`, `SCStreamOutputType` | Direct bindings + safe traits | Complete |
| `SCStreamConfiguration` | Direct bindings, including macOS 15.x microphone / HDR properties and macOS 26 preset creation | Complete |
| `SCStreamConfiguration.Preset.captureHDRRecordingPreservedSDRHDR10` | `SCStreamConfiguration::from_preset(SCStreamConfigurationPreset::CaptureHDRRecordingPreservedSDRHDR10)` | Complete |
| `SCStreamFrameInfo` attachment keys | `CMSampleBufferSCExt` accessors (`frame_status`, `display_time`, `scale_factor`, `content_scale`, `content_rect`, `bounding_rect`, `screen_rect`, `presenter_overlay_content_rect`, `dirty_rects`) plus batched `frame_info()` | Complete |
| `SCContentFilter` | Direct bindings + builder API | Complete |
| `SCShareableContent`, `SCShareableContentInfo`, `SCRunningApplication`, `SCDisplay`, `SCWindow` | Direct bindings | Complete |
| `SCContentSharingPicker` | Direct picker APIs plus callback-based `show*()` wrappers over observer-style flows | Complete |
| `SCContentSharingPickerConfiguration` | Direct bindings, including `allowed_picker_modes()` round-trip and exclusion getters | Complete |
| `SCRecordingOutput`, `SCRecordingOutputConfiguration` | Direct bindings, including duration / file size and `output_url()` round-trip | Complete |
| `SCStreamErrorCode` / `SCStreamErrorDomain` | Direct enum + constant mapping | Complete |
| `SCScreenshotManager` macOS 15.2 / 26.0 additions | Direct bindings | Complete |

## Notes on safe equivalents

### `SCStreamFrameInfo`

Apple models frame metadata as attachment keys on `CMSampleBuffer`. In Rust, the crate exposes those keys as typed accessors and a batched `FrameInfo` snapshot instead of raw string constants. This is a deliberate ergonomic layer, not a coverage gap.

### `SCContentSharingPicker` observer APIs

Apple's picker surface is observer- and presentation-oriented. The crate covers that behavior through callback-based `show()`, `show_filter()`, `show_for_stream()`, `show_using_style()`, and `show_for_stream_using_style()` helpers, plus direct picker state/configuration accessors.

### Assigned Core Foundation / Core Graphics properties

Apple declares `SCStreamConfiguration.backgroundColor`, `colorSpaceName`, and `colorMatrix` as assigned `CGColorRef` / `CFStringRef` properties. The bridge now retains the values it assigns so those properties remain valid for the full lifetime of the configuration object.

## Validation

The audited surface and follow-up fixes were validated with:

- `cargo clippy --all-features -- -D warnings`
- `cargo test --all-features`
