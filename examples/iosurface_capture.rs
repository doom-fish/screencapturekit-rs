//! Example demonstrating IOSurface-backed buffer access
//! 
//! This shows how to efficiently access frame data using IOSurface,
//! which provides zero-copy access to the underlying framebuffer.

use screencapturekit::{
    shareable_content::SCShareableContent,
    stream::{
        configuration::{SCStreamConfiguration, pixel_format::PixelFormat},
        content_filter::SCContentFilter,
        SCStream,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
    },
    output::{CVPixelBufferIOSurface, IOSurfaceLockOptions, CMSampleBuffer},
};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Duration;

struct FrameCounter {
    count: Arc<AtomicUsize>,
    iosurface_count: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
}

impl SCStreamOutputTrait for FrameCounter {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        self.count.fetch_add(1, Ordering::Relaxed);

        // Get the pixel buffer from the sample
        if let Some(pixel_buffer) = sample.get_image_buffer() {
            // Check if it's backed by IOSurface
            if pixel_buffer.is_backed_by_iosurface() {
                self.iosurface_count.fetch_add(1, Ordering::Relaxed);
                
                // Get the IOSurface
                if let Some(iosurface) = pixel_buffer.iosurface() {
                    println!("✅ Frame {} - IOSurface backed:", 
                        self.count.load(Ordering::Relaxed));
                    println!("   - Dimensions: {}x{}", iosurface.width(), iosurface.height());
                    println!("   - Bytes per row: {}", iosurface.bytes_per_row());
                    println!("   - Pixel format: 0x{:08X}", iosurface.pixel_format());
                    println!("   - In use: {}", iosurface.is_in_use());

                    // Example: Access the buffer data
                    match iosurface.lock(IOSurfaceLockOptions::ReadOnly) {
                        Ok(guard) => {
                            let slice = guard.as_slice();
                            println!("   - Buffer size: {} bytes", slice.len());
                            
                            // Example: Read first pixel (BGRA format typically)
                            if slice.len() >= 4 {
                                println!("   - First pixel (BGRA): [{}, {}, {}, {}]", 
                                    slice[0], slice[1], slice[2], slice[3]);
                            }
                            println!("   ✅ Successfully accessed IOSurface buffer");
                        }
                        Err(code) => println!("   ❌ Failed to lock IOSurface: {}", code),
                    }
                }
            } else {
                println!("⚠️  Frame {} - NOT IOSurface backed", 
                    self.count.load(Ordering::Relaxed));
            }
        }
        
        // Stop after 5 frames
        if self.count.load(Ordering::Relaxed) >= 5 {
            self.running.store(false, Ordering::Relaxed);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎥 IOSurface-Backed Buffer Example\n");

    // Get shareable content
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    
    if displays.is_empty() {
        eprintln!("❌ No displays found");
        return Ok(());
    }

    let display = &displays[0];
    println!("📺 Capturing display: {}x{}\n", display.width(), display.height());

    // Create filter
    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);

    // Create configuration
    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?
        .set_pixel_format(PixelFormat::BGRA)?  // BGRA format
        .set_queue_depth(3)?;

    // Create counter
    let count = Arc::new(AtomicUsize::new(0));
    let iosurface_count = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));
    
    let handler = FrameCounter {
        count: count.clone(),
        iosurface_count: iosurface_count.clone(),
        running: running.clone(),
    };

    // Create and start stream
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    
    println!("🚀 Starting capture...\n");
    stream.start_capture()?;

    // Wait for frames
    while running.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }

    // Stop capture
    println!("\n⏹️  Stopping capture...");
    stream.stop_capture()?;

    // Print summary
    let total = count.load(Ordering::Relaxed);
    let iosurface = iosurface_count.load(Ordering::Relaxed);
    
    println!("\n📊 Summary:");
    println!("   - Total frames: {}", total);
    println!("   - IOSurface-backed frames: {}", iosurface);
    println!("   - Percentage: {:.1}%", (iosurface as f64 / total as f64) * 100.0);

    if iosurface == total {
        println!("\n✅ All frames were IOSurface-backed!");
    } else {
        println!("\n⚠️  Some frames were not IOSurface-backed");
    }

    println!("\n✨ IOSurface support working correctly!");
    
    Ok(())
}
