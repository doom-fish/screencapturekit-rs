# ScreenCaptureKit SDK Coverage

Checked for `screencapturekit` 11.0.0 on 2026-09-24 against the
ScreenCaptureKit headers of the macOS 27.0 SDK and the macOS 26.5 SDK (the
default build SDK). [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) counts top-level
symbols (classes, protocols, enums and exported constants), and
[`COVERAGE_AUDIT_V2.md`](COVERAGE_AUDIT_V2.md) counts the properties and
methods of each class and protocol. Both are static checks: they show that a
public Rust item exists and calls the matching bridge export, not that every
item behaves correctly on every macOS version. Most types need a `macos_*`
Cargo feature; see the README.

## Summary

| SDK | Top-level symbols | Members |
| --- | --- | --- |
| macOS 26.5 | 41 of 41 wrapped (`SCStreamType` is deprecated and not scored) | 136 of 136 bridged |
| macOS 27.0 | 41 of 46 wrapped | 136 of 152 bridged |

Every macOS gap is a macOS 27 addition. `SCStreamErrorCode` covers every macOS
error code, including the macOS 27 `InsufficientStorage` (-3822) and
`NotSupported` (-3823).

## Not wrapped (macOS 27.0 SDK)

These additions are out of scope for 11.0:

- `SCClipBufferingOutput` and `SCClipBufferingOutputDelegate` (rolling replay
  buffer with clip export), and `SCStream addClipBufferingOutput:error:` /
  `removeClipBufferingOutput:error:`.
- `SCRecordingEditor` and `SCRecordingEditorDelegate`.
- `SCRecordingOutputConfiguration.mixesAudioWithMicrophone`.
- `SCContentSharingPicker.available`. `SCContentSharingPicker::is_available`
  reports whether the macOS 14 picker API exists, not this property.
- `SCContentFilter.microphoneEnabled` and `SCStream.capturing`.
- The `SCStreamFrameInfoVideoOrientation` frame attachment.

The 26.5 SDK that this release builds against does not declare them.

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

### `SCContentSharingPickerConfiguration` value semantics

Apple models the picker configuration as a Swift value type. The Rust wrapper
holds it in a reference-counted box, so `Clone` copies the box's contents
rather than bumping its refcount — otherwise two clones would share one mutable
configuration and the wrapper's `&mut self` setters would not be exclusive.

### Assigned Core Foundation / Core Graphics properties

Apple declares `SCStreamConfiguration.backgroundColor`, `colorSpaceName`, and `colorMatrix` as assigned `CGColorRef` / `CFStringRef` properties. The bridge now retains the values it assigns so those properties remain valid for the full lifetime of the configuration object.

### Configurations handed to ScreenCaptureKit

`SCStreamConfiguration`, `SCRecordingOutputConfiguration` and
`SCScreenshotConfiguration` are mutable Objective-C objects. The crate passes
ScreenCaptureKit a private copy when it creates a stream or recording output,
updates a stream's configuration, or takes a screenshot, so changing the Rust
value afterwards never races with work that is still in flight.

## Validation

11.0.0 was validated with:

- `cargo build --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (live capture tests skip when the process has
  no Screen Recording permission)
- `cargo +1.82.0 check --lib --all-features`
