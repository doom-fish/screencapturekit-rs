use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::{
    shareable_content::SCShareableContent,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
    },
};

use std::{
    fs::OpenOptions,
    io::Write,
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

struct AudioStreamOutput {
    sender: Sender<CMSampleBuffer>,
}

impl SCStreamOutputTrait for AudioStreamOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        self.sender
            .send(sample_buffer)
            .expect("could not send to output_buffer");
    }
}

fn main() -> Result<(), CFError> {
    let (tx, rx) = channel();
    let stream = get_stream(tx)?;
    stream.start_capture()?;

    let max_number_of_samples: i32 = 400;

    for sample_index in 0..max_number_of_samples {
        println!("sample_index={}", sample_index);

        let sample = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("could not receive from output_buffer");

        let b = sample.get_audio_buffer_list().expect("should work");
        for buffer_index in 0..b.num_buffers() {
            let buffer = b.get(buffer_index).expect("should work");

            let mut file = OpenOptions::new()
                .create(true)
                .append(true) // Use append mode
                .open(format!("out_{buffer_index}.raw"))
                .expect("failed to open file");

            println!(
                "{}: channels={}, size={}",
                buffer_index, buffer.number_channels, buffer.data_bytes_size
            );

            if let Err(e) = file.write_all(buffer.data()) {
                eprintln!("failed to write to file: {:?}", e);
            }
        }
    }

    stream.stop_capture().ok();
    thread::sleep(Duration::from_secs(1));
    Ok(())
}

fn get_stream(tx: Sender<CMSampleBuffer>) -> Result<SCStream, CFError> {
    let config = SCStreamConfiguration::new().set_captures_audio(true)?;

    let display = SCShareableContent::get().unwrap().displays().remove(0);
    let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(AudioStreamOutput { sender: tx }, SCStreamOutputType::Audio);

    Ok(stream)
}
