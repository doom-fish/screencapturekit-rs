//! IOSurface Access
//!
//! Demonstrates zero-copy GPU buffer access via IOSurface.
//! This example shows:
//! - Checking if buffer is IOSurface-backed
//! - Accessing IOSurface properties
//! - Locking and reading IOSurface data

use screencapturekit::prelude::*;
use screencapturekit::output::{CVPixelBufferIOSurface, IOSurfaceLockOptions, PixelBufferCursorExt};
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Handler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if matches!(output_type, SCStreamOutputType::Screen) {
            let n = self.count.fetch_add(1, Ordering::Relaxed);
            
            if n % 60 == 0 {
                if let Some(pixel_buffer) = sample.get_image_buffer() {
                    // Check if IOSurface-backed
                    if pixel_buffer.is_backed_by_iosurface() {
                        if let Some(iosurface) = pixel_buffer.iosurface() {
                            println!("\n📹 Frame {} - IOSurface", n);
                            println!("   Dimensions: {}x{}", iosurface.width(), iosurface.height());
                            println!("   Pixel format: 0x{:08X}", iosurface.pixel_format());
                            println!("   Bytes per row: {}", iosurface.bytes_per_row());
                            println!("   In use: {}", iosurface.is_in_use());
                            
                            // Lock and access data
                            if let Ok(guard) = iosurface.lock(IOSurfaceLockOptions::ReadOnly) {
                                let mut cursor = guard.cursor();
                                
                                // Read first pixel
                                if let Ok(pixel) = cursor.read_pixel() {
                                    println!("   First pixel: {:?}", pixel);
                                }
                                
                                println!("   ✅ IOSurface access successful");
                            }
                        }
                    } else {
                        println!("⚠️  Frame {} - Not IOSurface-backed", n);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 IOSurface Access\n");

    let content = SCShareableContent::get()?;
    let display = content.displays().into_iter().next()
        .ok_or("No displays found")?;

    let filter = SCContentFilter::build()
        .display(&display)
        .exclude_windows(&[])
        .build();

    let config = SCStreamConfiguration::build()
        .set_width(1920)?
        .set_height(1080)?
        .set_pixel_format(PixelFormat::BGRA)?;

    let count = Arc::new(AtomicUsize::new(0));
    let handler = Handler { count };
    
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    
    println!("Starting capture...\n");
    stream.start_capture()?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    stream.stop_capture()?;
    
    println!("\n✅ Done");
    Ok(())
}
