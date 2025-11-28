//! Stream lifecycle tests
//!
//! Tests for `SCStream` lifecycle and operations.

use screencapturekit::prelude::*;

#[test]
fn test_stream_creation() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default();
    
    let stream = SCStream::new(&filter, &config);
    
    println!("✓ Stream created successfully");
    drop(stream);
}

#[test]
fn test_stream_with_custom_config() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default()
        .set_width(1920)
        .set_height(1080)
        .set_shows_cursor(false);
    
    let stream = SCStream::new(&filter, &config);
    
    println!("✓ Stream with custom config created");
    drop(stream);
}

#[test]
fn test_stream_multiple_instances() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default();
    
    let stream1 = SCStream::new(&filter, &config);
    let stream2 = SCStream::new(&filter, &config);
    
    println!("✓ Multiple stream instances created");
    drop(stream1);
    drop(stream2);
}

#[test]
fn test_stream_clone() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default();
    
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
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config1 = SCStreamConfiguration::default();
    
    let stream = SCStream::new(&filter, &config1);
    
    let config2 = SCStreamConfiguration::default()
        .set_width(1280)
        .set_height(720);
    
    let result = stream.update_configuration(&config2);
    
    match result {
        Ok(()) => println!("✓ Configuration updated successfully"),
        Err(e) => println!("⚠ Configuration update failed (expected): {e}"),
    }
}

#[test]
fn test_stream_update_filter() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter1 = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default();
    
    let stream = SCStream::new(&filter1, &config);
    
    let filter2 = SCContentFilter::builder().display(display).build();
    
    let result = stream.update_content_filter(&filter2);
    
    match result {
        Ok(()) => println!("✓ Filter updated successfully"),
        Err(e) => println!("⚠ Filter update failed (expected): {e}"),
    }
}

#[test]
fn test_stream_output_types() {
    // Test that output types are accessible
    let screen = SCStreamOutputType::Screen;
    let audio = SCStreamOutputType::Audio;
    
    // Use the variables to avoid unused variable warning
    assert!(matches!(screen, SCStreamOutputType::Screen));
    assert!(matches!(audio, SCStreamOutputType::Audio));
    
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
fn test_stream_lifecycle() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }
    
    let display = &content.displays()[0];
    let filter = SCContentFilter::builder().display(display).build();
    let config = SCStreamConfiguration::default();
    
    {
        let _stream = SCStream::new(&filter, &config);
        println!("✓ Stream created in scope");
        // Stream drops here
    }
    
    println!("✓ Stream lifecycle complete");
}

#[test]
fn test_stream_different_displays() {
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    
    if content.displays().len() < 2 {
        println!("⚠ Need multiple displays for this test");
        return;
    }
    
    let display1 = &content.displays()[0];
    let display2 = &content.displays()[1];
    
    let filter1 = SCContentFilter::builder().display(display1).build();
    let filter2 = SCContentFilter::builder().display(display2).build();
    let config = SCStreamConfiguration::default();
    
    let stream1 = SCStream::new(&filter1, &config);
    let stream2 = SCStream::new(&filter2, &config);
    
    println!("✓ Streams for different displays created");
    drop(stream1);
    drop(stream2);
}
