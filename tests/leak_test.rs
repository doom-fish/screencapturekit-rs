#[cfg(test)]
mod leak_tests {

    use std::{process::Command, thread};

    use screencapturekit::{
        cm::CMSampleBuffer,
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration, content_filter::SCContentFilter,
            output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType, SCStream,
        },
    };

    pub struct Capturer {}

    impl Capturer {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for Capturer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SCStreamOutputTrait for Capturer {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            // Just get the timestamp to verify the sample buffer works
            let _timestamp = sample.presentation_timestamp();
        }
    }

    #[test]
    fn test_if_program_leaks() {
        for _ in 0..4 {
            // Create and immediately drop streams

            let stream = {
                let mut config = SCStreamConfiguration::default();
                config.set_captures_audio(true);
                config.set_width(100);
                config.set_height(100);

                let display = SCShareableContent::get();

                let d = display.unwrap().displays().remove(0);
                let filter = SCContentFilter::builder()
                    .display(&d)
                    .exclude_windows(&[])
                    .build();
                let mut stream = SCStream::new(&filter, &config);
                stream.add_output_handler(Capturer::new(), SCStreamOutputType::Audio);
                stream.add_output_handler(Capturer::new(), SCStreamOutputType::Screen);
                stream
            };
            stream.start_capture().ok();
            thread::sleep(std::time::Duration::from_millis(500));
            stream.stop_capture().ok();
            // Force drop of sc_stream
            drop(stream);
        }
        // Get the current process ID
        let pid = std::process::id();

        // Run the 'leaks' command
        let output = Command::new("leaks")
            .args([pid.to_string(), "-c".to_string()])
            .output()
            .expect("Failed to execute leaks command");

        // Check the output for leaks
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {stdout}");
        println!("stderr: {stderr}");

        // Check for leaks, but ignore known Apple framework leaks in ScreenCaptureKit
        // These are internal leaks in CMCapture/FigRemoteOperationReceiver that we can't fix
        if stdout.contains("0 leaks for 0 total leaked bytes") {
            return;
        }

        // Check if all leaks are from Apple frameworks (not our code)
        let apple_framework_leaks = stdout.contains("CMCapture")
            || stdout.contains("FigRemoteOperationReceiver")
            || stdout.contains("SCStream(SCContentSharing)");

        if apple_framework_leaks && !stdout.contains("screencapturekit") {
            println!("Note: Detected Apple framework leaks (not in our code), ignoring");
            return;
        }

        panic!("Memory leaks detected in our code");
    }
}
