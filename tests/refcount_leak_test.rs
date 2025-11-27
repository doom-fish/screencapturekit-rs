#[cfg(test)]
mod refcount_leak_tests {
    use std::{error::Error, process::Command, thread, time::Duration};

    use screencapturekit::{
        cm::CMSampleBuffer,
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration,
            content_filter::SCContentFilter,
            output_trait::SCStreamOutputTrait,
            output_type::SCStreamOutputType,
            SCStream,
        },
    };

    pub struct TestCapturer {}

    impl TestCapturer {
        pub fn new() -> Self {
            TestCapturer {}
        }
    }

    impl Default for TestCapturer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SCStreamOutputTrait for TestCapturer {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
            let _timestamp = sample.get_presentation_timestamp();
        }
    }

    /// Test for memory leaks in basic shareable content operations
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_shareable_content_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing SCShareableContent for leaks...");
        
        // Stress test with multiple clone/drop cycles
        for iteration in 0..50 {
            let content = SCShareableContent::get()?;
            
            // Clone multiple times
            let content2 = content.clone();
            let content3 = content2.clone();
            let content4 = content3.clone();
            
            // Access displays, windows, apps
            let _ = content.displays();
            let _ = content2.windows();
            let _ = content3.applications();
            
            // All should be dropped here
            drop(content4);
            drop(content3);
            drop(content2);
            drop(content);
            
            if iteration % 10 == 0 {
                println!("  ✓ Iteration {}/50", iteration + 1);
            }
        }
        
        check_for_leaks("SCShareableContent")?;
        Ok(())
    }

    /// Test for memory leaks in display cloning
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_display_clone_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing SCDisplay cloning for leaks...");
        
        let content = SCShareableContent::get()?;
        let display = content.displays().first().ok_or("No displays found")?.clone();
        
        for iteration in 0..100 {
            // Clone display many times
            let displays: Vec<_> = (0..10).map(|_| display.clone()).collect();
            
            // Use them
            for d in &displays {
                let _ = d.display_id();
                let _ = d.width();
                let _ = d.height();
            }
            
            // All dropped at end of scope
            drop(displays);
            
            if iteration % 25 == 0 {
                println!("  ✓ Iteration {}/100", iteration + 1);
            }
        }
        
        check_for_leaks("SCDisplay")?;
        Ok(())
    }

    /// Test for memory leaks in window cloning
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_window_clone_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing SCWindow cloning for leaks...");
        
        let content = SCShareableContent::get()?;
        
        if let Some(window) = content.windows().first() {
            let window = window.clone();
            
            for iteration in 0..100 {
                let windows: Vec<_> = (0..10).map(|_| window.clone()).collect();
                
                for w in &windows {
                    let _ = w.window_id();
                    let _ = w.frame();
                    let _ = w.title();
                }
                
                drop(windows);
                
                if iteration % 25 == 0 {
                    println!("  ✓ Iteration {}/100", iteration + 1);
                }
            }
        } else {
            println!("  ⚠ No windows found, skipping test");
        }
        
        check_for_leaks("SCWindow")?;
        Ok(())
    }

    /// Test for memory leaks in stream configuration
    #[test]
    fn test_stream_config_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing SCStreamConfiguration for leaks...");
        
        for iteration in 0..100 {
            let config = SCStreamConfiguration::build()
                .set_width(1920)?
                .set_height(1080)?
                .set_captures_audio(true)?
                .set_shows_cursor(true);
            
            // Clone multiple times
            let configs: Vec<_> = (0..10).map(|_| config.clone()).collect();
            
            drop(configs);
            drop(config);
            
            if iteration % 25 == 0 {
                println!("  ✓ Iteration {}/100", iteration + 1);
            }
        }
        
        check_for_leaks("SCStreamConfiguration")?;
        Ok(())
    }

    /// Test for memory leaks in content filter
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_content_filter_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing SCContentFilter for leaks...");
        
        let content = SCShareableContent::get()?;
        let displays = content.displays();
        let display = displays.first().ok_or("No displays found")?;
        
        for iteration in 0..100 {
            #[allow(deprecated)]
            let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();
            
            // Clone multiple times
            let filters: Vec<_> = (0..10).map(|_| filter.clone()).collect();
            
            drop(filters);
            drop(filter);
            
            if iteration % 25 == 0 {
                println!("  ✓ Iteration {}/100", iteration + 1);
            }
        }
        
        check_for_leaks("SCContentFilter")?;
        Ok(())
    }

    /// Test for memory leaks in full stream lifecycle
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_stream_lifecycle_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing full SCStream lifecycle for leaks...");
        
        for iteration in 0..5 {
            let config = SCStreamConfiguration::build()
                .set_captures_audio(true)?
                .set_width(100)?
                .set_height(100)?;

            let content = SCShareableContent::get()?;
            let display = content.displays().first().ok_or("No displays found")?.clone();
            
            #[allow(deprecated)]
            let filter = SCContentFilter::build().display(&display).exclude_windows(&[]).build();
            
            let mut stream = SCStream::new(&filter, &config);
            stream.add_output_handler(TestCapturer::new(), SCStreamOutputType::Audio);
            stream.add_output_handler(TestCapturer::new(), SCStreamOutputType::Screen);
            
            stream.start_capture().ok();
            thread::sleep(Duration::from_millis(500));
            stream.stop_capture().ok();
            
            // Explicit drop
            drop(stream);
            drop(filter);
            drop(display);
            drop(config);
            drop(content);
            
            println!("  ✓ Stream lifecycle {}/5 completed", iteration + 1);
        }
        
        check_for_leaks("Full Stream Lifecycle")?;
        Ok(())
    }

    /// Test for memory leaks with nested structures
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_nested_structures_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing nested structures for leaks...");
        
        for iteration in 0..50 {
            let content = SCShareableContent::get()?;
            
            // Create nested vectors of cloned objects
            let displays: Vec<_> = content
                .displays()
                .iter()
                .flat_map(|d| vec![d.clone(), d.clone(), d.clone()])
                .collect();
            
            let windows: Vec<_> = content
                .windows()
                .iter()
                .flat_map(|w| vec![w.clone(), w.clone()])
                .collect();
            
            let apps: Vec<_> = content
                .applications()
                .iter()
                .map(|a| a.clone())
                .collect();
            
            // Use the cloned objects
            for d in &displays {
                let _ = d.display_id();
            }
            for w in &windows {
                let _ = w.window_id();
            }
            for a in &apps {
                let _ = a.process_id();
            }
            
            // All should be properly dropped
            drop(displays);
            drop(windows);
            drop(apps);
            drop(content);
            
            if iteration % 10 == 0 {
                println!("  ✓ Iteration {}/50", iteration + 1);
            }
        }
        
        check_for_leaks("Nested Structures")?;
        Ok(())
    }

    /// Test for memory leaks with multi-threaded cloning
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_multithreaded_clone_no_leak() -> Result<(), Box<dyn Error>> {
        println!("🧪 Testing multi-threaded cloning for leaks...");
        
        let content = SCShareableContent::get()?;
        let display = content.displays().first().ok_or("No displays found")?.clone();
        
        for iteration in 0..20 {
            let mut handles = vec![];
            
            // Spawn multiple threads that clone and use the display
            for _ in 0..5 {
                let display = display.clone();
                let handle = thread::spawn(move || {
                    // Clone multiple times in each thread
                    let displays: Vec<_> = (0..10).map(|_| display.clone()).collect();
                    
                    for d in &displays {
                        let _ = d.display_id();
                        let _ = d.width();
                        let _ = d.height();
                    }
                    
                    // All dropped at end of scope
                });
                handles.push(handle);
            }
            
            // Wait for all threads
            for handle in handles {
                handle.join().unwrap();
            }
            
            if iteration % 5 == 0 {
                println!("  ✓ Multi-threaded iteration {}/20", iteration + 1);
            }
        }
        
        check_for_leaks("Multi-threaded Cloning")?;
        Ok(())
    }

    /// Test for leaks with recording output (macOS 15.0+)
    #[test]
    #[cfg(feature = "macos_15_0")]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_recording_output_no_leak() -> Result<(), Box<dyn Error>> {
        use screencapturekit::recording_output::{
            SCRecordingOutputCodec, SCRecordingOutputConfiguration,
        };
        use std::path::PathBuf;
        
        println!("🧪 Testing SCRecordingOutput for leaks...");
        
        for iteration in 0..50 {
            let mut config = SCRecordingOutputConfiguration::new();
            config.set_output_url(&PathBuf::from("/tmp/test_recording.mp4"));
            config.set_video_codec(SCRecordingOutputCodec::H264);
            config.set_average_bitrate(5_000_000);
            
            // Clone multiple times
            let configs: Vec<_> = (0..10).map(|_| config.clone()).collect();
            
            drop(configs);
            drop(config);
            
            if iteration % 10 == 0 {
                println!("  ✓ Iteration {}/50", iteration + 1);
            }
        }
        
        check_for_leaks("SCRecordingOutput")?;
        Ok(())
    }

    /// Test for leaks with content sharing picker (macOS 14.0+)
    #[test]
    #[cfg(feature = "macos_14_0")]
    fn test_content_sharing_picker_no_leak() -> Result<(), Box<dyn Error>> {
        use screencapturekit::content_sharing_picker::{
            SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
        };
        
        println!("🧪 Testing SCContentSharingPicker for leaks...");
        
        for iteration in 0..50 {
            let mut config = SCContentSharingPickerConfiguration::new();
            config.set_allowed_picker_modes(&[
                SCContentSharingPickerMode::SingleWindow,
                SCContentSharingPickerMode::SingleDisplay,
            ]);
            
            // Clone multiple times
            let configs: Vec<_> = (0..10).map(|_| config.clone()).collect();
            
            drop(configs);
            drop(config);
            
            if iteration % 10 == 0 {
                println!("  ✓ Iteration {}/50", iteration + 1);
            }
        }
        
        check_for_leaks("SCContentSharingPicker")?;
        Ok(())
    }

    /// Helper function to check for memory leaks using macOS 'leaks' command
    fn check_for_leaks(test_name: &str) -> Result<(), Box<dyn Error>> {
        // Give system time to clean up
        thread::sleep(Duration::from_millis(100));
        
        let pid = std::process::id();
        
        println!("  🔍 Checking for leaks (PID: {})...", pid);
        
        let output = Command::new("leaks")
            .args(&[pid.to_string(), "-c".to_string()])
            .output()
            .expect("Failed to execute leaks command");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Check for success pattern
        if stdout.contains("0 leaks for 0 total leaked bytes") {
            println!("  ✅ {} - No leaks detected!", test_name);
            return Ok(());
        }
        
        // If not successful, print detailed information
        println!("  ❌ {} - Memory leaks detected!", test_name);
        println!("\n  === STDOUT ===");
        println!("{}", stdout);
        println!("\n  === STDERR ===");
        println!("{}", stderr);
        
        Err(format!("Memory leaks detected in {}", test_name).into())
    }

    /// Comprehensive leak test that runs all scenarios
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_comprehensive_no_leak() -> Result<(), Box<dyn Error>> {
        println!("\n🧪 Running comprehensive leak test...\n");
        
        // Test basic operations
        println!("Phase 1: Basic Operations");
        {
            let content = SCShareableContent::get()?;
            for _ in 0..10 {
                let _c = content.clone();
            }
        }
        
        // Test with actual objects
        println!("Phase 2: Object Cloning");
        {
            let content = SCShareableContent::get()?;
            if let Some(display) = content.displays().first() {
                for _ in 0..10 {
                    let _d = display.clone();
                }
            }
        }
        
        // Test configurations
        println!("Phase 3: Configurations");
        {
            for _ in 0..10 {
                let config = SCStreamConfiguration::build()
                    .set_width(1920)?
                    .set_height(1080)?;
                let _c = config.clone();
            }
        }
        
        // Give time for cleanup
        thread::sleep(Duration::from_millis(200));
        
        check_for_leaks("Comprehensive Test")?;
        Ok(())
    }
}
