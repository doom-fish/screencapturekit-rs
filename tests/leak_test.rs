#[cfg(test)]
mod leak_tests {

    use std::{error::Error, process::Command, thread};

    use core_media_rs::cm_sample_buffer::CMSampleBuffer;
    use screencapturekit::{
        output::sc_stream_frame_info::{SCFrameStatus, SCStreamFrameInfo},
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration, content_filter::SCContentFilter,
            delegate_trait::SCStreamDelegateTrait, output_trait::SCStreamOutputTrait,
            output_type::SCStreamOutputType, SCStream,
        },
    };

    pub struct Capturer {}

    impl Capturer {
        pub fn new() -> Self {
            Capturer {}
        }
    }

    impl Default for Capturer {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SCStreamDelegateTrait for Capturer {}

    impl SCStreamOutputTrait for Capturer {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            let audio = sample.get_audio_buffer_list();
            let desc = sample.get_format_description();
            let info = SCStreamFrameInfo::from_sample_buffer(&sample);
            if let Ok(ref inner) = info {
                if inner.status() == SCFrameStatus::Complete {
                    println!("sample: {sample:?}");
                }

            }
            drop(sample);
            println!("audio: {audio:?}");
            println!("desc: {desc:?}");
            println!("info: {info:?}");
        }
    }

    #[test]
    fn test_if_program_leaks() -> Result<(), Box<dyn Error>> {
        for _ in 0..4 {
            // Create and immediately drop streams

            let stream = {
                let config = SCStreamConfiguration::new()
                    .set_captures_audio(true)?
                    .set_width(100)?
                    .set_height(100)?;

                let display = SCShareableContent::get();

                let d = display.unwrap().displays().remove(0);
                let filter = SCContentFilter::new().with_display_excluding_windows(&d, &[]);
                let mut stream = SCStream::new_with_delegate(&filter, &config, Capturer::default());
                stream.add_output_handler(Capturer::new(), SCStreamOutputType::Audio);
                stream.add_output_handler(Capturer::new(), SCStreamOutputType::Screen);
                stream
            };
            stream.start_capture().ok();
            thread::sleep(std::time::Duration::from_millis(100));
            stream.stop_capture().ok();
            // Force drop of sc_stream
            drop(stream);
        }
        // Get the current process ID
        let pid = std::process::id();

        // Run the 'leaks' command
        let output = Command::new("leaks")
            .args(&[pid.to_string(), "-c".to_string()])
            .output()
            .expect("Failed to execute leaks command");

        // Check the output for leaks
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);
        if !stdout.contains("0 leaks for 0 total leaked bytes") {
            panic!("Memory leaks detected");
        }

        Ok(())
    }
}
