# ScreenCaptureKit SDK Coverage

> **Snapshot — not a live coverage status.** This document records a
> point-in-time audit performed against `screencapturekit` **v3.1.1**. It has
> **not** been re-verified against the upcoming 10.0.0 package and should
> be read as a historical audit, not an up-to-date certification.

This document records the `screencapturekit` v3.1.1 coverage audit against
Apple's `ScreenCaptureKit.framework` from Xcode 26.2 (`MacOSX26.2.sdk`).

## What "coverage" means here

The audit counted **top-level declarations** — classes, protocols, enums and
exported constants — and found a Rust binding for each one. It did **not**
enumerate the individual properties and methods on those types.

So a type marked "Bound" below means *the crate binds that type*, not *the
crate binds every member of that type*. The distinction is not academic: after
this audit shipped, `SCScreenshotConfiguration` was still missing every getter
for its eleven properties, and `SCContentSharingPicker` was missing
`add(_:)` / `remove(_:)` / `defaultConfiguration` / `setConfiguration(_:for:)`
and the standalone `present(...)` family — all on types the table certified as
"Bound". Those specific gaps were closed in 8.0.

**Member-level coverage is not certified.** Treat a missing property as a bug
worth filing, not as a documented exclusion.

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

## Declaration coverage map

"Bound" = the crate exposes the type. See the caveat above: it does not assert
that every member of the type is reachable from Rust.

| Apple SDK surface | Rust coverage | Declaration |
| --- | --- | --- |
| `SCStream`, `SCStreamDelegate`, `SCStreamOutput`, `SCStreamOutputType` | Direct bindings + safe traits | Bound |
| `SCStreamConfiguration` | Direct bindings, including macOS 15.x microphone / HDR properties and macOS 26 preset creation | Bound |
| `SCStreamConfiguration.Preset.captureHDRRecordingPreservedSDRHDR10` | `SCStreamConfiguration::from_preset(SCStreamConfigurationPreset::CaptureHDRRecordingPreservedSDRHDR10)` | Bound |
| `SCStreamFrameInfo` attachment keys | `CMSampleBufferSCExt` accessors (`frame_status`, `display_time`, `scale_factor`, `content_scale`, `content_rect`, `bounding_rect`, `screen_rect`, `presenter_overlay_content_rect`, `dirty_rects`) plus batched `frame_info()` | Bound |
| `SCContentFilter` | Direct bindings + builder API | Bound |
| `SCShareableContent`, `SCShareableContentInfo`, `SCRunningApplication`, `SCDisplay`, `SCWindow` | Direct bindings | Bound |
| `SCContentSharingPicker` | Direct picker APIs plus callback-based `show*()` wrappers over observer-style flows | Bound |
| `SCContentSharingPickerConfiguration` | Direct bindings, including `allowed_picker_modes()` round-trip and exclusion getters | Bound |
| `SCRecordingOutput`, `SCRecordingOutputConfiguration` | Direct bindings, including duration / file size and `output_url()` round-trip | Bound |
| `SCStreamErrorCode` / `SCStreamErrorDomain` | Direct enum + constant mapping | Bound |
| `SCScreenshotManager` macOS 15.2 / 26.0 additions | Direct bindings | Bound |

## Notes on safe equivalents

### `SCStreamFrameInfo`

Apple models frame metadata as attachment keys on `CMSampleBuffer`. In Rust, the crate exposes those keys as typed accessors and a batched `FrameInfo` snapshot instead of raw string constants. This is a deliberate ergonomic layer, not a coverage gap.

### `SCContentSharingPicker` observer APIs

Apple's picker surface is observer- and presentation-oriented. The crate offers
two shapes over it:

- One-shot `show()`, `show_filter()`, `show_for_stream()`, `show_using_style()`
  and `show_for_stream_using_style()` helpers, which install an observer,
  present the picker, and resolve on the first event.
- `add_observer()`, which registers a **repeating** observer and returns an
  `SCPickerSubscription`, paired with the standalone `present*()` entry points.
  This is the shape that makes `allows_changing_selected_content` work — the
  one-shot helpers latch after the first selection by design.

At the time of the v3.1.1 audit only the one-shot shape existed, so
`allows_changing_selected_content` could be set but its re-selection events
were dropped. Added in 8.0.

### `SCContentSharingPickerConfiguration` value semantics

Apple models the picker configuration as a Swift value type. The Rust wrapper
holds it in a reference-counted box, so `Clone` copies the box's contents
rather than bumping its refcount — otherwise two clones would share one mutable
configuration and the wrapper's `&mut self` setters would not be exclusive.

### Assigned Core Foundation / Core Graphics properties

Apple declares `SCStreamConfiguration.backgroundColor`, `colorSpaceName`, and `colorMatrix` as assigned `CGColorRef` / `CFStringRef` properties. The bridge now retains the values it assigns so those properties remain valid for the full lifetime of the configuration object.

## Validation

The audited surface and follow-up fixes were validated with:

- `cargo clippy --all-features -- -D warnings`
- `cargo test --all-features`
