#[cfg(test)]
mod leak_tests {

    use std::{error::Error, process::Command};

    use core_media_rs::cm_sample_buffer::CMSampleBuffer;
    use screencapturekit::{
        output::sc_stream_frame_info::SCStreamFrameInfo,
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration, content_filter::SCContentFilter,
            delegate_trait::SCStreamDelegateTrait, output_trait::SCStreamOutputTrait,
            output_type::SCStreamOutputType, SCStream,
        },
    };

    // #[global_allocator]
    // static ALLOC: dhat::Alloc = dhat::Alloc;

    pub struct Capturer {}

    impl Capturer {
        pub fn new() -> Self {
            println!("Capturer initialized");
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
            let _ = sample.get_audio_buffer_list();
            let _ = sample.get_format_description();
            let _ = SCStreamFrameInfo::from_buffer(&sample);
            println!("New frame recvd");
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
                let output = Capturer::new();
                let mut stream = SCStream::new_with_delegate(&filter, &config, Capturer::default());
                stream.add_output_handler(output, SCStreamOutputType::Audio);
                stream
            };
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
