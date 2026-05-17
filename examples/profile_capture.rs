//! CPU profiling driver: runs a 30-second audio+video capture with the
//! realistic per-frame workload (`frame_info` on every frame, `audio_buffer_list`
//! on every audio buffer) so a sampling profiler (`samply`, `xctrace`) sees a
//! representative slice of the dispatch path.
//!
//! Build with `--release` (debug profiling is meaningless here) and the
//! release profile retains debug info for symbolication:
//!
//! ```bash
//! cargo build --release --example profile_capture --features macos_14_0
//! samply record --rate 4000 -- ./target/release/examples/profile_capture
//! ```
//!
//! Or with `xctrace`'s Time Profiler:
//!
//! ```bash
//! xctrace record --template "Time Profiler" --output ./cap.trace \
//!     --launch -- ./target/release/examples/profile_capture
//! ```
//!
//! To generate a flame graph after `samply`, open the printed URL in your
//! browser (Firefox profiler) and use the inverted-call-tree view to find
//! per-callback hot self-time.

use screencapturekit::cm::{CMSampleBufferExt, CMSampleBufferSCExt};
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const CAPTURE_SECONDS: u64 = 30;

extern "C" {
    fn sc_initialize_core_graphics();
}

struct Counters {
    video: AtomicUsize,
    audio: AtomicUsize,
}

fn init_core_graphics() {
    // SAFETY: This initializes the process-global Core Graphics state once
    // before capture starts; the Swift bridge exposes it specifically for this
    // setup step.
    unsafe { sc_initialize_core_graphics() };
}

fn rate_per_second(count: usize, elapsed: Duration) -> f64 {
    u32::try_from(count).map_or_else(|_| f64::from(u32::MAX), f64::from) / elapsed.as_secs_f64()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔎 profile_capture — driving a {CAPTURE_SECONDS}s capture window for sampling");

    // Force CG init so the profile doesn't capture the first-call setup cost.
    init_core_graphics();

    let content = SCShareableContent::get()?;
    let display = content.displays().into_iter().next().ok_or("no display")?;
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_pixel_format(PixelFormat::BGRA)
        .with_captures_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    let counters = Arc::new(Counters {
        video: AtomicUsize::new(0),
        audio: AtomicUsize::new(0),
    });

    let mut stream = SCStream::new(&filter, &config);

    let video_counters = Arc::clone(&counters);
    stream.add_output_handler(
        move |buf: CMSampleBuffer, _ot: SCStreamOutputType| {
            // Realistic per-video-frame workload — the path that profile
            // attributions should show.
            let _info = buf.frame_info();
            let _img = buf.image_buffer();
            video_counters.video.fetch_add(1, Ordering::Relaxed);
        },
        SCStreamOutputType::Screen,
    );

    let audio_counters = Arc::clone(&counters);
    stream.add_output_handler(
        move |buf: CMSampleBuffer, _ot: SCStreamOutputType| {
            // Realistic per-audio-buffer workload.
            let _abl = buf.audio_buffer_list();
            audio_counters.audio.fetch_add(1, Ordering::Relaxed);
        },
        SCStreamOutputType::Audio,
    );

    println!("Starting capture...");
    let t0 = Instant::now();
    stream.start_capture()?;
    std::thread::sleep(Duration::from_secs(CAPTURE_SECONDS));
    stream.stop_capture()?;
    let elapsed = t0.elapsed();

    let v = counters.video.load(Ordering::Relaxed);
    let a = counters.audio.load(Ordering::Relaxed);
    println!(
        "✅ Done in {elapsed:?} — {v} video frames ({:.1} fps) + {a} audio buffers ({:.1} bps)",
        rate_per_second(v, elapsed),
        rate_per_second(a, elapsed),
    );

    Ok(())
}
