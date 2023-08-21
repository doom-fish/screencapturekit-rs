use std::{
    sync::atomic::{AtomicI64, Ordering},
    thread,
    time::Duration,
};

use once_cell::sync::Lazy;
use screencapturekit_sys::{
    content_filter::{UnsafeContentFilter, UnsafeInitParams::Display},
    shareable_content::UnsafeSCShareableContent,
    stream::UnsafeSCStream,
    stream_configuration::UnsafeStreamConfiguration,
    stream_error_handler::UnsafeSCStreamError,
    stream_output_handler::{CMSampleBuffer, UnsafeSCStreamOutput},
};

#[repr(C)]
struct TestHandler {}
impl UnsafeSCStreamError for TestHandler {
    fn handle_error(&self) {
        eprintln!("ERROR!");
    }
}
static PREV_TIMESTAMP: Lazy<AtomicI64> = Lazy::new(|| AtomicI64::new(0));

impl UnsafeSCStreamOutput for TestHandler {
    fn got_sample(&self, sample: CMSampleBuffer) {
        let timescale_ms = 1000000;
        let prev_timestamp = PREV_TIMESTAMP.load(Ordering::Relaxed);
        let new_timestamp = sample.presentation_timestamp.value / timescale_ms;
        println!(
            "{} FPS with sample: {:?}",
            new_timestamp - prev_timestamp,
            sample
        );

        PREV_TIMESTAMP.store(new_timestamp, Ordering::Relaxed);
    }
}
fn main() {
    let display = UnsafeSCShareableContent::get()
        .unwrap()
        .displays()
        .into_iter()
        .next()
        .unwrap();
    let width = display.get_width();
    let height = display.get_height();
    println!("{width}, {height}");
    let width = display.get_width();
    let height = display.get_height();
    let filter = UnsafeContentFilter::init(Display(display));

    let config = UnsafeStreamConfiguration {
        width,
        height,
        ..Default::default()
    };

    let stream = UnsafeSCStream::init(filter, config.into(), TestHandler {});
    stream.add_stream_output(TestHandler {});
    stream.start_capture();

    thread::sleep(Duration::from_millis(10_000));
}
