<div align="center">
  <h1>ScreenCaptureKit-rs</h1>
  <p><strong>Safe, idiomatic Rust bindings for Apple's <a href="https://developer.apple.com/documentation/screencapturekit">ScreenCaptureKit</a> framework.</strong></p>
  <p>Capture screens, windows, and applications on macOS 12.3+ with high performance and low overhead.</p>
</div>

<div align="center"><p>
    <a href="https://crates.io/crates/screencapturekit"><img alt="Crates.io" src="https://img.shields.io/crates/v/screencapturekit?style=for-the-badge&logo=rust&color=C9CBFF&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://crates.io/crates/screencapturekit"><img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/screencapturekit?style=for-the-badge&logo=rust&color=A6E3A1&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://docs.rs/screencapturekit"><img alt="docs.rs" src="https://img.shields.io/docsrs/screencapturekit?style=for-the-badge&logo=docs.rs&color=8bd5ca&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://github.com/doom-fish/screencapturekit-rs#license"><img alt="License" src="https://img.shields.io/crates/l/screencapturekit?style=for-the-badge&logo=apache&color=ee999f&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://github.com/doom-fish/screencapturekit-rs/actions"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/doom-fish/screencapturekit-rs/ci.yml?branch=main&style=for-the-badge&logo=github&color=c69ff5&logoColor=D9E0EE&labelColor=302D41" /></a>
    <a href="https://github.com/doom-fish/screencapturekit-rs/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/doom-fish/screencapturekit-rs?style=for-the-badge&logo=starship&color=F5E0DC&logoColor=D9E0EE&labelColor=302D41" /></a>
</p></div>

> **💼 Looking for a hosted desktop recording API?** Check out [Recall.ai](https://www.recall.ai/product/desktop-recording-sdk?utm_source=github&utm_medium=sponsorship&utm_campaign=screencapturekit-rs) — an API for recording Zoom, Google Meet, Microsoft Teams, in-person meetings, and more.

<https://github.com/user-attachments/assets/8a272c48-7ec3-4132-9111-4602b4fa991d>

---

## Highlights

- 🎥 **Screen, window, and app capture** with a clean builder-pattern API
- 🔊 **System audio** + **microphone** capture (macOS 13.0+ / 15.0+)
- ⚡ **Real-time, zero-copy** frame delivery via `IOSurface` / Metal
- 🔄 **Async** support that works with any executor (Tokio, async-std, smol, …)
- 📸 **Screenshots** + **direct-to-file recording** (macOS 14.0+ / 15.0+)
- 🖱️ **System content picker** UI (macOS 14.0+)
- 🛡️ **Memory safe** — proper retain/release, leak-tested
- 📦 **Zero runtime dependencies**

## Table of Contents

- [Install](#install) · [Quick Start](#quick-start) · [Recipes](#recipes)
- [Feature Flags](#feature-flags) · [Examples](#examples) · [Documentation](#documentation)
- [Requirements & Permissions](#requirements--permissions) · [Performance](#performance)
- [Troubleshooting](#troubleshooting) · [Migration](#migration) · [Contributing](#contributing)

---

## Install

```toml
[dependencies]
screencapturekit = "2"
```

Opt-in features (additive):

| Feature | Enables |
|---|---|
| `async` | Runtime-agnostic async API (Tokio / async-std / smol / …) |
| `macos_13_0` | Audio capture, sync clock |
| `macos_14_0` | Screenshots, content picker, content info |
| `macos_14_2` | Menu bar capture, child windows, presenter overlay |
| `macos_14_4` | Current-process shareable content |
| `macos_15_0` | Recording output, HDR capture, microphone |
| `macos_15_2` | Screenshot in rect, stream active/inactive delegates |
| `macos_26_0` | Advanced screenshot config, HDR screenshot output |

`macos_*` features are **cumulative** — enabling `macos_15_0` automatically enables every earlier version. Pick the highest version your minimum-supported macOS will satisfy:

```toml
screencapturekit = { version = "2", features = ["async", "macos_15_0"] }
```

> **Upgrading from 1.x?** See [`docs/MIGRATION.md`](docs/MIGRATION.md#migrating-from-1x-to-20)
> — the headline 2.0 changes are a `Send + Sync` bound on output / delegate
> traits, `#[non_exhaustive]` on `PixelFormat` and `SCStreamErrorCode`, and a
> new `PixelFormat::Unknown(FourCharCode)` variant.

## Quick Start

A minimal screen capture in ~25 lines. Everything else builds on these four steps:
**(1)** list shareable content, **(2)** build a content filter, **(3)** configure
the stream, **(4)** add an output handler and start.

```rust,no_run
use screencapturekit::prelude::*;

struct Handler;
impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _: SCStreamOutputType) {
        println!("📹 frame @ {:?}", sample.presentation_timestamp());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = SCShareableContent::get()?;
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA);

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(Handler, SCStreamOutputType::Screen);
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;
    Ok(())
}
```

> Output / delegate handlers must be `Send + Sync` — Apple's dispatch
> queues may invoke them concurrently from arbitrary threads.

Permission required — see [Requirements & Permissions](#requirements--permissions).
Run it: `cargo run --example 01_basic_capture`.

## Recipes

Short snippets for the most common follow-on tasks. Every recipe is a runnable
example in [`examples/`](examples/) — see the [Examples](#examples) table.

<details>
<summary><strong>Window capture with audio</strong></summary>

```rust,no_run
use screencapturekit::prelude::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let content = SCShareableContent::get()?;
let window = content.windows().into_iter()
    .find(|w| w.title().as_deref() == Some("Safari"))
    .ok_or("Safari window not found")?;

let filter = SCContentFilter::create().with_window(&window).build();
let config = SCStreamConfiguration::new()
    .with_captures_audio(true)
    .with_sample_rate(48_000)
    .with_channel_count(2);

let mut stream = SCStream::new(&filter, &config);
// stream.add_output_handler(...) for Screen and/or Audio
stream.start_capture()?;
# Ok(()) }
```
</details>

<details>
<summary><strong>Closure-based handler (no trait impl needed)</strong></summary>

```rust,no_run
# use screencapturekit::prelude::*;
# fn example(stream: &mut SCStream) {
stream.add_output_handler(
    |sample: CMSampleBuffer, _of_type: SCStreamOutputType| {
        println!("📹 frame @ {:?}", sample.presentation_timestamp());
    },
    SCStreamOutputType::Screen,
);
# }
```

Closures must be `Fn + Send + Sync + 'static`.
</details>

<details>
<summary><strong>Async capture (any executor)</strong></summary>

```rust,ignore
use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
use screencapturekit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = AsyncSCShareableContent::get().await?;
    let display = &content.displays()[0];

    let filter = SCContentFilter::create()
        .with_display(display).with_excluding_windows(&[]).build();
    let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);

    // 30-frame ring buffer; oldest frames are dropped if the consumer can't keep up.
    let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
    stream.start_capture()?;

    while let Some(_frame) = stream.next().await {
        // process frame
        # break;
    }

    stream.stop_capture()?;
    Ok(())
}
```

Requires the `async` feature. Works with Tokio, async-std, smol, or any
custom executor — the binding does **not** spawn its own runtime.
</details>

<details>
<summary><strong>Screenshot (macOS 14.0+)</strong></summary>

```rust,no_run
# #[cfg(feature = "macos_14_0")]
# fn example(
#     filter: &screencapturekit::stream::content_filter::SCContentFilter,
#     config: &screencapturekit::stream::configuration::SCStreamConfiguration,
# ) -> Result<(), Box<dyn std::error::Error>> {
use screencapturekit::screenshot_manager::SCScreenshotManager;

let img = SCScreenshotManager::capture_image(filter, config)?;
let pixels = img.bgra_data()?;            // native BGRA — skips R↔B swap
// For sustained loops, reuse a buffer:
// img.bgra_data_into(&mut buffer)?;
# Ok(()) }
```
</details>

<details>
<summary><strong>System content picker (macOS 14.0+)</strong></summary>

```rust,ignore
use screencapturekit::content_sharing_picker::*;
use screencapturekit::prelude::*;

let config = SCContentSharingPickerConfiguration::new();
SCContentSharingPicker::show(&config, |outcome| match outcome {
    SCPickerOutcome::Picked(result) => {
        let (w, h) = result.pixel_size();
        let filter = result.filter();
        // Use `filter` with SCStream as in the Quick Start.
        let _ = (w, h, filter);
    }
    SCPickerOutcome::Cancelled => println!("user cancelled"),
    SCPickerOutcome::Error(e)  => eprintln!("picker error: {e}"),
});
```

For async contexts, use [`AsyncSCContentSharingPicker::show`].
</details>

<details>
<summary><strong>Direct-to-file recording (macOS 15.0+)</strong></summary>

See [`examples/10_recording_output.rs`](examples/10_recording_output.rs) — it
covers `SCRecordingOutput`, `SCRecordingOutputConfiguration`, and the
delegate callbacks for start / finish / error.
</details>

<details>
<summary><strong>Custom dispatch queue / QoS</strong></summary>

```rust,no_run
use screencapturekit::prelude::*;
use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
# fn example(stream: &mut SCStream) {
let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);
stream.add_output_handler_with_queue(
    |_sample, _of_type| { /* runs on `queue` */ },
    SCStreamOutputType::Screen,
    Some(&queue),
);
# }
```

`QoS` levels: `Background`, `Utility`, `Default`, `UserInitiated`, `UserInteractive` (Quality of Service).
</details>

<details>
<summary><strong>Zero-copy GPU access (IOSurface → Metal / wgpu)</strong></summary>

```rust,no_run
use screencapturekit::prelude::*;
struct H;
impl SCStreamOutputTrait for H {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _: SCStreamOutputType) {
        if let Some(pb) = sample.image_buffer() {
            if let Some(surface) = pb.io_surface() {
                let _ = (surface.width(), surface.height());
                // Wrap as `MTLTexture` (see examples 17/18) — no copy.
            }
        }
    }
}
```

Built-in Metal helpers live in `screencapturekit::metal` and ship a small
shader library (`SHADER_SOURCE`) covering BGRA, YCbCr, and UI overlay
rendering. See [`examples/16_full_metal_app/`](examples/16_full_metal_app/)
for a complete app and [`examples/18_wgpu_integration.rs`](examples/18_wgpu_integration.rs)
for the wgpu equivalent.
</details>

[`AsyncSCContentSharingPicker::show`]: https://doom-fish.github.io/screencapturekit-rs/screencapturekit/async_api/struct.AsyncSCContentSharingPicker.html

## Examples

23 runnable examples cover every API surface. The full table with feature
requirements lives in [`examples/README.md`](examples/README.md). A few
favourites to start with:

| Example | What it shows |
|---|---|
| [`01_basic_capture`](examples/01_basic_capture.rs) | Minimal screen capture — start here |
| [`08_async`](examples/08_async.rs) | Async API, picker, runtime-agnostic patterns |
| [`09_closure_handlers`](examples/09_closure_handlers.rs) | Closures + delegate callbacks |
| [`10_recording_output`](examples/10_recording_output.rs) | Direct-to-file recording (macOS 15.0+) |
| [`11_content_picker`](examples/11_content_picker.rs) | System picker UI (macOS 14.0+) |
| [`16_full_metal_app/`](examples/16_full_metal_app/) | Full Metal viewer app (macOS 14.0+) |
| [`18_wgpu_integration`](examples/18_wgpu_integration.rs) | Zero-copy wgpu integration |
| [`19_ffmpeg_encoding`](examples/19_ffmpeg_encoding.rs) | Real-time H.264 via ffmpeg |
| [`24_batched_apis_showcase`](examples/24_batched_apis_showcase.rs) | Batched FFI vs per-element (perf) |

```bash
cargo run --example 01_basic_capture
cargo run --example 10_recording_output --features macos_15_0
cargo run --example 08_async            --features "async,macos_14_0"
```

## Feature Flags

See the full feature table under [Install](#install). One small example of
gating version-specific options:

```rust,ignore
let mut config = SCStreamConfiguration::new().with_width(1920).with_height(1080);

#[cfg(feature = "macos_14_2")]
{
    config.set_ignores_shadows_single_window(true);
    config.set_includes_child_windows(false);
}
```

## Documentation

| Where | What |
|---|---|
| [docs.rs](https://docs.rs/screencapturekit) | Full API reference |
| [`docs/MIGRATION.md`](docs/MIGRATION.md) | Upgrading between major versions |
| [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) | Benchmark methodology + results |
| [`examples/README.md`](examples/README.md) | All 23 examples + feature requirements |
| [`CHANGELOG.md`](CHANGELOG.md) | Release notes |

## Requirements & Permissions

- **macOS 12.3+** (Monterey) — base `ScreenCaptureKit`
- **macOS 13.0+** — audio capture · **14.0+** — picker / screenshots ·
  **15.0+** — recording / HDR / mic · **26.0+** — advanced screenshots
- **Xcode Command Line Tools** at build time (`xcode-select --install`)

Screen capture **always requires user permission**. To grant it:

1. **System Settings → Privacy & Security → Screen Recording**
2. Enable your binary (during development this is usually your terminal or IDE)
3. Restart the app

For distribution, add a purpose string to `Info.plist` — the user-facing
TCC prompt requires it and the app will be terminated without one:

```xml
<key>NSScreenCaptureUsageDescription</key>
<string>Capture your screen so the app can …</string>
```

ScreenCaptureKit itself does **not** require a code-signing entitlement
on either signed or sandboxed apps — capture is gated by the user's TCC
grant in **System Settings → Privacy & Security → Screen & System
Audio Recording**, not by an `Entitlements.plist` key. (There is no
`com.apple.security.screen-capture` entitlement; that key does not
exist in Apple's entitlement reference.) If your app is sandboxed you
still need the sandbox entitlement itself:

```xml
<key>com.apple.security.app-sandbox</key>      <true/>
```

## Performance

Full capture (60 fps + 48 kHz stereo) costs **~1.9% of one core** end-to-end
on Apple Silicon — the binding itself is below the noise floor of a 4 kHz
sampling profiler; nearly all CPU lives in Apple's `SkyLight` /
`libdispatch` / `libxpc` pipeline.

| Resolution | Expected FPS | First-frame latency |
|---|---|---|
| 1080p | 30–60 | 30–100 ms |
| 4K | 15–30 | 50–150 ms |

**Hot-path tips:**

- Prefer `BGRA` to skip the per-pixel R↔B swap when uploading to Metal /
  wgpu / ffmpeg (`SCScreenshotManager::bgra_data` is ~5% faster than `rgba_data`).
- Reuse a `Vec<u8>` across screenshots with the `*_data_into` variants
  (saves a ~33 MB allocation per 4K frame — new in 2.1).
- When iterating many windows / displays / apps, use the batched
  [`SCShareableContent::snapshot()`](https://doom-fish.github.io/screencapturekit-rs/screencapturekit/shareable_content/struct.SCShareableContent.html)
  API — collapses `1 + N + 6N` FFI calls into one round-trip per category
  (~2× faster on a typical desktop).
- Read every `SCStreamFrameInfo` attachment in one cast via
  `CMSampleBuffer::frame_info()`.

```rust,no_run
use screencapturekit::prelude::*;
use screencapturekit::shareable_content::ContentSnapshot;
# fn example() -> Result<(), Box<dyn std::error::Error>> {
let content = SCShareableContent::get()?;
let ContentSnapshot { displays, windows, applications } =
    content.snapshot().ok_or("snapshot failed")?;
for w in &windows {
    let app = w.owning_app_index.and_then(|i| applications.get(i));
    println!("{} - {}", app.map(|a| &*a.application_name).unwrap_or(""),
             w.title.as_deref().unwrap_or(""));
}
# let _ = displays;
# Ok(()) }
```

Run benchmarks on your hardware:

```bash
cargo bench
cargo bench --bench hotspots --features macos_14_0
```

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology, throughput
numbers at various resolutions, and tuning guidance.

## Troubleshooting

| Symptom | Likely cause / fix |
|---|---|
| `SCShareableContent::get()` returns empty / errors | Missing **Screen Recording** permission — grant it in System Settings, then restart |
| Black / empty frames | Captured window minimized; pixel format mismatch; filter doesn't include the right display/window |
| No audio samples | Did you set `.with_captures_audio(true)` **and** add a handler for `SCStreamOutputType::Audio`? |
| Build fails with Swift bridge errors | `xcode-select --install`; then `cargo clean && cargo build` |
| App crashes after notarization | Missing `NSScreenCaptureUsageDescription` in `Info.plist` — the system terminates apps that trigger the Screen Recording TCC prompt without one (see [Requirements](#requirements--permissions)) |
| `match` on `PixelFormat` / `SCStreamErrorCode` no longer compiles | Both are `#[non_exhaustive]` in 2.0 — add a wildcard `_ => …` arm |

## Migration

Upgrading? See [`docs/MIGRATION.md`](docs/MIGRATION.md) for the full guide.
The 2.0 highlights:

- `SCStreamOutputTrait` / `SCStreamDelegateTrait` (and closure overloads)
  now require `Send + Sync`
- `PixelFormat` is `#[non_exhaustive]` and gained `Unknown(FourCharCode)`
  for forward-compat with future Apple pixel formats
- `SCStreamErrorCode` is `#[non_exhaustive]`
- `PixelFormat`'s `PartialEq` / `Hash` are normalised through `FourCharCode`
- Every `macos_*` Cargo feature now propagates to the Swift bridge build
  (the build fails loudly on SDK detection failure rather than silently
  dropping symbols)

2.1 added the `bgra_data_into` / `rgba_data_into` buffer-reuse APIs and a
native-BGRA fast path on `SCScreenshotManager` — both are non-breaking.

## Contributing

Contributions welcome! Please:

1. Follow existing patterns — builder pattern with `::new()` and `.with_*()`
2. Add tests for new functionality
3. `cargo fmt && cargo clippy --all-features -- -D warnings && cargo test`
4. Update docs and `CHANGELOG.md`

See `CLAUDE.md` / `AGENTS.md` for the project conventions agents follow.

## Used By

Powering 50+ open-source projects across screen recording, AI agents,
meeting transcription, and remote desktop. A few highlights:

- **[AFFiNE](https://github.com/toeverything/AFFiNE)** — knowledge base, Notion / Miro alternative (68k+ ⭐)
- **[voicebox](https://github.com/jamiepine/voicebox)** — open-source AI voice studio (25k+ ⭐)
- **[Cap](https://github.com/CapSoftware/Cap)** — open-source Loom alternative (19k+ ⭐)
- **[Observer](https://github.com/Roy3838/Observer)** — local AI screen observer (1.4k+ ⭐)
- **[my-translator](https://github.com/phuc-nt/my-translator)** — real-time speech translation (1k+ ⭐)
- **[hylarana](https://github.com/mycrl/hylarana)** — cross-platform screen casting in Rust
- **[gst-screencapturekit](https://github.com/doom-fish/gst-screencapturekit)** — `GStreamer` plugin
- **[open-agent](https://github.com/AFK-surf/open-agent)**, **[watson.ai](https://github.com/LatentDream/watson.ai)**, **[harana/search](https://github.com/harana/search)**, **[agent-native](https://github.com/BuilderIO/agent-native)** by Builder.io

<details>
<summary>And many more…</summary>

[fl_caption](https://github.com/xkeyC/fl_caption), [Lycoris](https://github.com/solaoi/lycoris), [Hindsight](https://github.com/Tomotsugu-dev/Hindsight), [kivio](https://github.com/ZMGID/kivio), [Drift](https://github.com/diiviikk5/Drift), [Phantom](https://github.com/zruss11/Phantom), [ruhear](https://github.com/aizcutei/ruhear), [Tab5-Screen-Streamer](https://github.com/Hiroki-Kawakami/Tab5-Screen-Streamer), [macloop](https://github.com/kemsta/macloop), [beer](https://github.com/alii/beer), [phantom-ear](https://github.com/fomyio/phantom-ear), [Logia](https://github.com/daschinmoy21/Logia), [VibeTube](https://github.com/VibeCreAI/VibeTube), [silly-ai](https://github.com/zz85/silly-ai), [aresampler](https://github.com/adnissen/aresampler), [xos](https://github.com/xlateai/xos), [scriberr-desktop](https://github.com/rishikanthc/scriberr-desktop), [echonote](https://github.com/luismctech/echonote), [zest-wallpaper](https://github.com/lgcenen/zest-wallpaper), [mira](https://github.com/fluffypony/mira), [overlay-ai](https://github.com/VishnuVVR-369/overlay-ai), [open-rec](https://github.com/TommyBez/open-rec), [omnirec](https://github.com/omnirec/omnirec), [oxiremote](https://github.com/nhtera/oxiremote), [LocalWhisper](https://github.com/ly7erg1c/LocalWhisper), [Hush](https://github.com/khawkins98/Hush), [cocuyo](https://github.com/jorgeajimenezl/cocuyo), [openhush](https://github.com/claymore666/openhush), [tucknotes](https://github.com/ajgagnon/tucknotes), [domino](https://github.com/nitinm21/domino), [bridge](https://github.com/maorinka/bridge), [screen-recorder](https://github.com/forfd8960/screen-recorder), [orbit](https://github.com/divesh-balani/orbit), [audio-capture](https://github.com/birdieHyun/audio-capture), [AFFiNE-teto](https://github.com/shrik450/AFFiNE-teto), [loom](https://github.com/rkendel1/loom).

</details>

*Using screencapturekit-rs? [Open an issue](https://github.com/doom-fish/screencapturekit-rs/issues) and we'll add you.*

## Contributors

Thanks to everyone who has contributed!

[Per Johansson](https://github.com/doom-fish) (maintainer) ·
[Iason Paraskevopoulos](https://github.com/iasparaskev) ·
[Kris Krolak](https://github.com/kriskrolak) ·
[Tokuhiro Matsuno](https://github.com/tokuhirom) ·
[Pranav Joglekar](https://github.com/pranavj1001) ·
[Alex Jiao](https://github.com/uohzxela) ·
[Charles](https://github.com/aizukanne) ·
[bigduu](https://github.com/bigduu) ·
[Andrew N](https://github.com/adnissen)

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
