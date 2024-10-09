use std::{
    fs::OpenOptions, io::Write, sync::mpsc::{channel, Sender}
};

use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::{
    shareable_content::sc_shareable_content::SCShareableContent,
    stream::{
        sc_content_filter::SCContentFilter, sc_stream::SCStream,
        sc_stream_configuration::SCStreamConfiguration,
        sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
    },
};

struct AudioStreamOutput {
    sender: Sender<(CMSampleBuffer, SCStreamOutputType)>,
}

impl SCStreamOutputTrait for AudioStreamOutput {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.sender
            .send((sample_buffer, of_type))
            .expect("could not send to output_buffer");
    }
}

fn main() -> Result<(), CFError> {
    let (tx, rx) = channel();

    let stream = {
        let config = SCStreamConfiguration::new().set_captures_audio(true)?;

        let display = SCShareableContent::get().unwrap().displays().remove(0);
        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        stream.add_output_handler(AudioStreamOutput { sender: tx }, SCStreamOutputType::Audio);

        stream
    };
    stream.start_capture()?;

    let max_number_of_samples: i32 = 20;

    for _ in 0..max_number_of_samples {
        let (buf, _) = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("could not receive from output_buffer");
        let b = buf.get_audio_buffer_list().expect("should work");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true) // Use append mode
            .open("out.raw")
            .expect("failed to open file");

        for i in 0..b.number_buffers {
            let buf = b.buffers[i as usize];
            println!(
                "  {}: channels={}, size={}",
                i, buf.number_channels, buf.data_bytes_size
            );
            let s = unsafe { std::slice::from_raw_parts(buf.data, buf.data_bytes_size as usize) };

            if let Err(e) = file.write_all(s) {
                eprintln!("failed to write to file: {:?}", e);
            }
        }
    }

    stream.stop_capture()
}
