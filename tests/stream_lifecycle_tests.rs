//! Stream lifecycle tests
//!
//! Tests for `SCStream` lifecycle and operations.

mod common;

use screencapturekit::prelude::*;

#[test]
fn test_stream_creation() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream = SCStream::new(&filter, &config).expect("failed to create stream");

    println!("✓ Stream created successfully");
    drop(stream);
}

#[test]
fn test_stream_with_custom_config() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let mut config = SCStreamConfiguration::default();
    config.set_width(1920);
    config.set_height(1080);
    config.set_shows_cursor(false);

    let stream = SCStream::new(&filter, &config).expect("failed to create stream");

    println!("✓ Stream with custom config created");
    drop(stream);
}

#[test]
fn test_stream_multiple_instances() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream1 = SCStream::new(&filter, &config).expect("failed to create stream");
    let stream2 = SCStream::new(&filter, &config).expect("failed to create stream");

    println!("✓ Multiple stream instances created");
    drop(stream1);
    drop(stream2);
}

#[test]
fn test_stream_clone() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream1 = SCStream::new(&filter, &config).expect("failed to create stream");
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
#[cfg(feature = "macos_14_0")]
fn test_stream_update_configuration() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config1 = SCStreamConfiguration::default();

    let stream = SCStream::new(&filter, &config1).expect("failed to create stream");

    let mut config2 = SCStreamConfiguration::default();
    config2.set_width(1280);
    config2.set_height(720);

    let result = stream.update_configuration(&config2);

    match result {
        Ok(()) => println!("✓ Configuration updated successfully"),
        Err(e) => println!("⚠ Configuration update failed (expected): {e}"),
    }
}

#[test]
fn test_stream_update_filter() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter1 = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream = SCStream::new(&filter1, &config).expect("failed to create stream");

    let filter2 = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");

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

    println!("✓ Output types accessible");
}

#[test]
fn test_stream_different_displays() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().len() < 2 {
        println!("⚠ Only one display available");
        return;
    }

    let display1 = &content.displays()[0];
    let display2 = &content.displays()[1];

    let filter1 = SCContentFilter::create()
        .with_display(display1)
        .build()
        .expect("failed to build content filter");
    let filter2 = SCContentFilter::create()
        .with_display(display2)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream1 = SCStream::new(&filter1, &config).expect("failed to create stream");
    let stream2 = SCStream::new(&filter2, &config).expect("failed to create stream");

    println!("✓ Streams on different displays created");
    drop(stream1);
    drop(stream2);
}

#[test]
fn test_stream_debug_display() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };

    if content.displays().is_empty() {
        println!("⚠ No displays available");
        return;
    }

    let display = &content.displays()[0];
    let filter = SCContentFilter::create()
        .with_display(display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let stream = SCStream::new(&filter, &config).expect("failed to create stream");

    // Test Debug trait
    let debug_str = format!("{stream:?}");
    assert!(debug_str.contains("SCStream"));

    // Test Display trait
    let display_str = format!("{stream}");
    assert!(!display_str.is_empty());

    println!("✓ Debug and Display traits work");
}

#[test]
fn test_stream_identity_is_shared_by_clones_only() {
    if !crate::common::screen_capture_allowed() {
        return;
    }
    let Ok(content) = SCShareableContent::get() else {
        println!("⚠ Skipping - no screen recording permission");
        return;
    };
    let Some(display) = content.displays().into_iter().next() else {
        println!("⚠ No displays available");
        return;
    };
    let filter = SCContentFilter::create()
        .with_display(&display)
        .build()
        .expect("failed to build content filter");
    let config = SCStreamConfiguration::default();

    let first = SCStream::new(&filter, &config).expect("failed to create stream");
    let clone = first.clone();
    let second = SCStream::new(&filter, &config).expect("failed to create stream");

    assert_eq!(first.identity(), clone.identity());
    assert!(first.identity().matches(&clone));
    assert_ne!(first.identity(), second.identity());
    assert!(!second.identity().matches(&first));
}
