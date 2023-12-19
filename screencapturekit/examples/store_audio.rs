use objc_foundation::INSData;
use screencapturekit::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::sc_content_filter::{InitParams, SCContentFilter};
use screencapturekit::sc_error_handler::StreamErrorHandler;
use screencapturekit::sc_output_handler::{SCStreamOutputType, StreamOutput};
use screencapturekit::sc_shareable_content::SCShareableContent;
use screencapturekit::sc_stream::SCStream;
use screencapturekit::sc_stream_configuration::SCStreamConfiguration;
use std::{
    fs,
    fs::{File, OpenOptions},
    io::Write,
    ops::Deref,
    path::PathBuf,
    process::Command,
    thread::sleep,
    time::Duration,
};

use screencapturekit_sys::{audio_buffer, sc_stream_frame_info::SCFrameStatus};

struct StoreAudioHandler {}

struct ErrorHandler;

impl StreamErrorHandler for ErrorHandler {
    fn on_error(&self) {
        eprintln!("ERROR!")
    }
}

impl StreamOutput for StoreAudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        let format_description = sample
            .sys_ref
            .get_format_description()
            .expect("format description");
        println!("format_description={:?}", format_description);
        let audio = format_description
            .audio_format_description_get_stream_basic_description()
            .expect("Get AudioStreamBasicDescription");
        println!("audio info: {:?}", audio);
        println!("  format={:?}", audio.get_format_name().unwrap());
        println!("  flags={:?}", audio.get_flag_names());

        let audio_buffer_list = sample.audio_buffer_list.unwrap();
        println!("audio buffer list: number={:?}", audio_buffer_list.len());
        for (i, buffer) in audio_buffer_list.into_iter().enumerate() {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(format!("{}.raw", i))
                .expect("fail to create file!");

            if let Err(e) = file.write_all(buffer.data.deref()) {
                eprintln!("fail to write file: {}", e);
            }
        }
    }
}
fn main() {
    let content = SCShareableContent::current();
    let display = content.displays.first().unwrap();
    let filter = SCContentFilter::new(InitParams::Display(display.clone()));

    let config = SCStreamConfiguration {
        width: 2,  // Can't be 0 or 1, otherwise it will give you nothing
        height: 2, // Same as above
        captures_audio: true,
        excludes_current_process_audio: true,
        ..Default::default()
    };

    for i in 0..8 {
        let filename = format!("{}.raw", i);
        if PathBuf::from(filename.clone()).exists() {
            fs::remove_file(PathBuf::from(filename.to_string())).unwrap();
        }
    }

    let mut stream = SCStream::new(filter, config, ErrorHandler);

    stream.add_output(StoreAudioHandler {}, SCStreamOutputType::Audio);

    stream.start_capture().ok();

    sleep(Duration::from_secs(5));

    stream.stop_capture().ok();

    // In order to play the audio, you need to convert the raw PCM file into a WAV format.
    //
    // Here's an example of how one can do this:
    // sox -t raw -r 48000 -e floating-point -b 32 -c 1 --endian little /tmp/audio-0.raw /tmp/output-0.wav
}
