#![allow(clippy::pedantic, clippy::nursery)]
//! Stream lifecycle tests
//!
//! Tests for SCStream lifecycle and operations.

use screencapturekit::prelude::*;

#[test]
fn test_stream_creation() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build();
    
    let stream = SCStream::new(&filter, &config);
    
    println!("✓ Stream created successfully");
    drop(stream);
}

#[test]
fn test_stream_with_custom_config() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build()
        .set_width(1920).unwrap()
        .set_height(1080).unwrap()
        .set_shows_cursor(false).unwrap();
    
    let stream = SCStream::new(&filter, &config);
    
    println!("✓ Stream with custom config created");
    drop(stream);
}

#[test]
fn test_stream_multiple_instances() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build();
    
    let stream1 = SCStream::new(&filter, &config);
    let stream2 = SCStream::new(&filter, &config);
    
    println!("✓ Multiple stream instances created");
    drop(stream1);
    drop(stream2);
}

#[test]
fn test_stream_clone() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build();
    
    let stream1 = SCStream::new(&filter, &config);
    let stream2 = stream1.clone();
    
    println!("✓ Stream clone works");
    drop(stream1);
    drop(stream2);
}

#[test]
fn test_stream_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<SCStream>();
    assert_sync::<SCStream>();
    
    println!("✓ SCStream is Send + Sync");
}

#[test]
fn test_stream_update_configuration() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config1 = SCStreamConfiguration::build();
    
    let stream = SCStream::new(&filter, &config1);
    
    let config2 = SCStreamConfiguration::build()
        .set_width(1280).unwrap()
        .set_height(720).unwrap();
    
    let result = stream.update_configuration(&config2);
    
    match result {
        Ok(()) => println!("✓ Configuration updated successfully"),
        Err(e) => println!("⚠ Configuration update failed (expected): {}", e),
    }
}

#[test]
fn test_stream_update_filter() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter1 = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build();
    
    let stream = SCStream::new(&filter1, &config);
    
    let filter2 = SCContentFilter::build().display(display).build();
    
    let result = stream.update_content_filter(&filter2);
    
    match result {
        Ok(()) => println!("✓ Filter updated successfully"),
        Err(e) => println!("⚠ Filter update failed (expected): {}", e),
    }
}

#[test]
fn test_stream_output_types() {
    // Test that output types are accessible
    let _screen = SCStreamOutputType::Screen;
    let _audio = SCStreamOutputType::Audio;
    
    println!("✓ Stream output types accessible");
}

#[test]
fn test_stream_output_type_clone() {
    let output_type1 = SCStreamOutputType::Screen;
    let output_type2 = output_type1;
    
    assert_eq!(
        std::mem::discriminant(&output_type1),
        std::mem::discriminant(&output_type2)
    );
    
    println!("✓ Output type is Copy");
}

#[test]
#[ignore = "Window filter causes CGS_REQUIRE_INIT assertion in test environment"]
fn test_stream_with_window_filter() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.windows().is_empty() {
        println!("⚠ No windows available");
        return;
    }
    
    let window = &content.windows()[0];
    let filter = SCContentFilter::build().window(window).build();
    let config = SCStreamConfiguration::build();
    
    let stream = SCStream::new(&filter, &config);
    
    println!("✓ Stream with window filter created");
    drop(stream);
}

#[test]
fn test_stream_lifecycle() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::build().display(display).build();
    let config = SCStreamConfiguration::build();
    
    {
        let _stream = SCStream::new(&filter, &config);
        println!("✓ Stream created in scope");
        // Stream drops here
    }
    
    println!("✓ Stream lifecycle complete");
}

#[test]
fn test_stream_different_displays() {
    let content = match SCShareableContent::get() {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Skipping - no screen recording permission");
            return;
        }
    };
    
    if content.displays().len() < 2 {
        println!("⚠ Need multiple displays for this test");
        return;
    }
    
    let display1 = &content.displays()[0];
    let display2 = &content.displays()[1];
    
    let filter1 = SCContentFilter::build().display(display1).build();
    let filter2 = SCContentFilter::build().display(display2).build();
    let config = SCStreamConfiguration::build();
    
    let stream1 = SCStream::new(&filter1, &config);
    let stream2 = SCStream::new(&filter2, &config);
    
    println!("✓ Streams for different displays created");
    drop(stream1);
    drop(stream2);
}
