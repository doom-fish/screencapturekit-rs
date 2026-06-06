# Migration Guide

This guide helps you migrate between major versions of `screencapturekit-rs`.

> **Note:** The current release line is **7.x**. The sections below document
> historical major-version migrations (the FFI-hardening work that started in
> 2.0). For changes in recent releases, see [`CHANGELOG.md`](../CHANGELOG.md).

## Migrating from 1.x to 2.0

Version 2.0 hardens the FFI boundary. Most projects can upgrade by
bumping the dependency and addressing a handful of compile errors —
no design-level rework is required.

### Cargo.toml

```diff
 [dependencies]
-screencapturekit = "1"
+screencapturekit = "2"
```

### `Send + Sync` bound on output / delegate traits

`SCStreamOutputTrait` and `SCStreamDelegateTrait` (and the `Fn(...)` closure
overloads) now require `Send + Sync`. Apple's dispatch queues may invoke the
handler concurrently from arbitrary threads, so any state owned by the
handler must be thread-safe.

**Before (1.x):**
```rust,ignore
struct Handler { count: std::cell::Cell<usize> }   // !Sync — compiles in 1.x
impl SCStreamOutputTrait for Handler { /* ... */ }
```

**After (2.0):**
```rust,ignore
use std::sync::atomic::{AtomicUsize, Ordering};

struct Handler { count: AtomicUsize }              // Send + Sync
impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, _: CMSampleBuffer, _: SCStreamOutputType) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}
```

For closures: replace `Cell` / `Rc` with `Arc<Atomic*>` /
`Arc<Mutex<...>>` / `Arc<RwLock<...>>`.

### `PixelFormat::Unknown(FourCharCode)`

`PixelFormat` is now `#[non_exhaustive]` and surfaces unrecognised codes
via a new `Unknown(FourCharCode)` variant instead of mapping them to
`BGRA`. This means **every `match` over `PixelFormat` must include a
wildcard arm**:

**Before (1.x):**
```rust,ignore
match config.pixel_format() {
    PixelFormat::BGRA => { /* ... */ }
    PixelFormat::YCbCr420v => { /* ... */ }
    PixelFormat::YCbCr420f => { /* ... */ }
    PixelFormat::L10R => { /* ... */ }
}
```

**After (2.0):**
```rust,ignore
match config.pixel_format() {
    PixelFormat::BGRA => { /* ... */ }
    PixelFormat::YCbCr420v => { /* ... */ }
    PixelFormat::YCbCr420f => { /* ... */ }
    PixelFormat::L10R => { /* ... */ }
    PixelFormat::Unknown(code) => eprintln!("unrecognised pixel format: {code}"),
    _ => { /* future variants */ }
}
```

`PartialEq` / `Hash` are now normalised through `FourCharCode`, so two
representations of the same OSType (e.g. `PixelFormat::BGRA` vs
`PixelFormat::Unknown(FourCharCode::from_bytes(*b"BGRA"))`) compare equal.

### `SCStreamErrorCode` is `#[non_exhaustive]`

`match` arms over `SCStreamErrorCode` (typically inside an
`SCError::StreamError { code, .. }` arm) now require a wildcard:

```rust,ignore
match err_code {
    SCStreamErrorCode::UserStopped => { /* graceful */ }
    SCStreamErrorCode::UserDeclined => { /* permission */ }
    _ => { /* anything Apple adds in a future macOS */ }
}
```

### Build-time SDK enforcement

The build script no longer silently degrades when xcrun / SDK detection
fails — it bails with a clear error pointing at `xcode-select`. Make sure
Xcode Command Line Tools are installed and selected:

```bash
xcode-select --install
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

Every `macos_*` Cargo feature is also forwarded to the Swift compile, so
a feature only enabled on the Rust side (which used to silently miss the
matching Swift symbols) will now produce a coherent linker error.

### New APIs you can opt into

- `SCContentSharingPicker::is_active()` / `set_is_active()` — query and
  toggle the picker's idle state without recreating it.
- `SCContentSharingPicker::default_configuration()` — read the system
  default `SCContentSharingPickerConfiguration` so user-facing pickers
  match macOS defaults.
- `CMSampleBuffer::presenter_overlay_content_rect()` (and the matching
  field on `frame_info()`) — the new Presenter Overlay layout rect.

## Migrating from 2.0 to 2.1

2.1 is fully backwards-compatible with 2.0 — no source changes required.
New optional APIs:

- `CGImage::rgba_data_into(&mut [u8])` and `bgra_data_into(&mut [u8])` —
  render into a caller-supplied buffer to amortise the per-call
  `width*height*4` byte allocation across many screenshots.
- Native-BGRA fast path in `SCScreenshotManager` skips the channel swap
  for downstreams that accept BGRA directly (Metal / wgpu / ffmpeg).

## Migrating from 2.1 to 3.0

3.0 migrates the Core Graphics / Core Media / IOSurface / Core Video
foundation types onto the shared
[`apple-cf`](https://crates.io/crates/apple-cf) and
[`apple-metal`](https://crates.io/crates/apple-metal) crates, eliminating
`screencapturekit`'s private nominal duplicates. Most affected types are now
**re-exports**, so `use screencapturekit::cg::CGRect;` (or the prelude) keeps
working unchanged.

The one source-level change: the ScreenCaptureKit-specific accessors on
`CMSampleBuffer` moved to **extension traits**. Bring them into scope to call
them:

```rust,ignore
use screencapturekit::cm::{CMSampleBufferExt, CMSampleBufferSCExt};
// now `sample.image_buffer()`, `sample.frame_status()`, … resolve
```

The prelude already re-exports both traits, so `use screencapturekit::prelude::*;`
is enough.

## Migrating from 3.x to 4.0

4.0 removes duplicated Core Media / Core Graphics value types from the public
API in favour of the canonical `apple-cf` ones:

- `ScreenshotManager::capture_image` now returns `apple_cf::cg::CGImage`.
- `screencapturekit::cm::CMTime` is now a re-export of `apple_cf::cm::CMTime`.

If you previously converted between `screencapturekit`'s types and `apple-cf`'s
when chaining into ImageIO / VideoToolbox, **delete those conversions** — the
types are now identical.

## Migrating from 4.0 to 5.0

5.0 adopts `apple-cf` 0.8's **nested `CGRect` layout**. Flat field *access*
becomes nested through `origin` / `size`:

```diff
-let x = rect.x;
-let w = rect.width;
+let x = rect.origin.x;
+let w = rect.size.width;
```

The convenience constructor is unchanged — `CGRect::new(x, y, w, h)` still
takes four flat coordinates.

## Migrating from 5.0 to 6.0

6.0 re-exports the final Core Media timing types from `apple-cf`:
`screencapturekit::cm::{CMSampleTimingInfo, CMClock}` are now re-exports of
`apple_cf::cm::{CMSampleTimingInfo, CMClock}`. As with 4.0, drop any manual
conversions between the previously-distinct types. No other source changes are
required.

> The 4.0 → 6.0 bumps are all driven by consolidating onto `apple-cf`; if your
> code only used `screencapturekit`'s own types (via the prelude or
> `screencapturekit::{cg, cm}`) the upgrade is typically just the `CGRect`
> field-access change from 5.0.

## Migrating from 0.x to 1.0

Version 1.0 introduced a complete API redesign with builder patterns, async support, and new macOS features.

### Configuration API Changes

**Before (0.x):**
```rust
use screencapturekit::sc_stream_configuration::UnsafeSCStreamConfiguration;

let mut config = UnsafeSCStreamConfiguration::default();
config.set_width(1920);
config.set_height(1080);
config.set_shows_cursor(true);
```

**After (1.0):**
```rust
use screencapturekit::prelude::*;

let config = SCStreamConfiguration::new()
    .with_width(1920)
    .with_height(1080)
    .with_shows_cursor(true);
```

### Content Filter API Changes

**Before (0.x):**
```rust
use screencapturekit::sc_content_filter::UnsafeSCContentFilter;

let filter = UnsafeSCContentFilter::new(display);
```

**After (1.0):**
```rust
use screencapturekit::prelude::*;

let filter = SCContentFilter::create()
    .with_display(&display)
    .with_excluding_windows(&[])
    .build();
```

### Stream Creation Changes

**Before (0.x):**
```rust
use screencapturekit::sc_stream::UnsafeSCStream;

let stream = UnsafeSCStream::new(filter, config, handler);
stream.start_capture();
```

**After (1.0):**
```rust
use screencapturekit::prelude::*;

let mut stream = SCStream::new(&filter, &config);
stream.add_output_handler(handler, SCStreamOutputType::Screen);
stream.start_capture()?;
```

### Handler Trait Changes

**Before (0.x):**
```rust
impl StreamOutput for MyHandler {
    fn stream_output(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        // process sample
    }
}
```

**After (1.0):**
```rust
impl SCStreamOutputTrait for MyHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        // process sample
    }
}
```

### Closure Handlers (New in 1.0)

You can now use closures instead of implementing traits:

```rust
stream.add_output_handler(
    |sample: CMSampleBuffer, output_type: SCStreamOutputType| {
        println!("Got frame!");
    },
    SCStreamOutputType::Screen
);
```

### Error Handling

**Before (0.x):**
```rust
// Errors were often panics or Option<T>
let content = SCShareableContent::get().unwrap();
```

**After (1.0):**
```rust
// Proper Result<T, SCError> types
let content = SCShareableContent::get()?;
```

### Module Path Changes

| 0.x Path | 1.0 Path |
|----------|----------|
| `screencapturekit::sc_stream::*` | `screencapturekit::stream::*` |
| `screencapturekit::sc_stream_configuration::*` | `screencapturekit::stream::configuration::*` |
| `screencapturekit::sc_content_filter::*` | `screencapturekit::stream::content_filter::*` |
| `screencapturekit::sc_shareable_content::*` | `screencapturekit::shareable_content::*` |

**Recommended:** Use the prelude for common types:

```rust
use screencapturekit::prelude::*;
```

### Feature Flag Changes

| 0.x | 1.0 |
|-----|-----|
| N/A | `async` - Async API support |
| N/A | `macos_13_0` - Audio capture |
| N/A | `macos_14_0` - Screenshots, content picker |
| N/A | `macos_14_2` - Menu bar, child windows |
| N/A | `macos_15_0` - Recording, HDR, microphone |
| N/A | `macos_15_2` - Screenshot in rect |
| N/A | `macos_26_0` - Advanced screenshot config |

## Migrating from 1.0 to 1.1

### Builder Method Rename

**Before (1.0.0):**
```rust
let filter = SCContentFilter::build()
    .with_display(&display)
    .with_excluding_windows(&[])
    .build();
```

**After (1.1+):**
```rust
let filter = SCContentFilter::create()  // build() → create()
    .with_display(&display)
    .with_excluding_windows(&[])
    .build();
```

### Configuration Setters

**Before (1.0.0):**
```rust
let mut config = SCStreamConfiguration::new();
config.set_width(1920);  // Returns &mut Self
config.set_height(1080);
```

**After (1.1+):**
```rust
let config = SCStreamConfiguration::new()
    .with_width(1920)   // Chainable
    .with_height(1080);
```

## Migrating from 1.1/1.2 to 1.3+

### Content Picker API

**Before (1.2):**
```rust
// Blocking API
let result = SCContentSharingPicker::pick(&config)?;
```

**After (1.3+):**
```rust
// Callback-based API
SCContentSharingPicker::show(&config, |outcome| {
    match outcome {
        SCPickerOutcome::Picked(result) => { /* use result */ }
        SCPickerOutcome::Cancelled => { /* handle cancel */ }
        SCPickerOutcome::Error(e) => { /* handle error */ }
    }
});

// Or async (with async feature)
let outcome = AsyncSCContentSharingPicker::show(&config).await;
```

### Getter Method Naming

The `get_` prefix was removed from getters:

**Before:**
```rust
let width = config.get_width();
let rect = filter.get_content_rect();
let time = sample.get_presentation_timestamp();
```

**After:**
```rust
let width = config.width();
let rect = filter.content_rect();
let time = sample.presentation_timestamp();
```

### CMSampleBuffer Methods

**Before:**
```rust
sample.get_image_buffer()
sample.get_format_description()
```

**After:**
```rust
sample.image_buffer()
sample.format_description()
```

## Deprecated APIs

The following APIs are deprecated and will be removed in future versions:

| Deprecated | Replacement |
|------------|-------------|
| `SCStreamConfiguration::builder()` | `SCStreamConfiguration::new()` |
| `config.get_*()` methods | `config.*()` (without get_ prefix) |

## Quick Migration Checklist

- [ ] Update `Cargo.toml` to new version
- [ ] Replace `Unsafe*` types with safe equivalents
- [ ] Use `prelude::*` for common imports
- [ ] Replace trait implementations with closures (optional)
- [ ] Update handler trait name to `SCStreamOutputTrait`
- [ ] Add `?` for error handling (functions now return `Result`)
- [ ] Add feature flags for version-specific APIs
- [ ] Remove `get_` prefix from getter calls
- [ ] Update `SCContentFilter::build()` to `::create()`
