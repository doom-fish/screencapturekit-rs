# Migration Guide

This guide helps you migrate between major versions of `screencapturekit-rs`.

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
