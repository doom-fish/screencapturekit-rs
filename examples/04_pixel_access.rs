//! Pixel Buffer Access
//!
//! Demonstrates accessing pixel data from captured frames.
//! This example shows:
//! - Locking pixel buffers
//! - Using `std::io::Cursor` to read pixels
//! - Reading specific pixel coordinates
//! - Direct slice access

use screencapturekit::output::{CVImageBufferLockExt, PixelBufferCursorExt, PixelBufferLockFlags};
use screencapturekit::prelude::*;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Handler {
    count: Arc<AtomicUsize>,
}

impl SCStreamOutputTrait for Handler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if matches!(output_type, SCStreamOutputType::Screen) {
            let n = self.count.fetch_add(1, Ordering::Relaxed);

            // Process every 60th frame
            if n % 60 == 0 {
                if let Some(pixel_buffer) = sample.get_image_buffer() {
                    if let Ok(guard) = pixel_buffer.lock(PixelBufferLockFlags::ReadOnly) {
                        println!("\n📹 Frame {n}");
                        println!("   Size: {}x{}", guard.width(), guard.height());
                        println!("   Bytes per row: {}", guard.bytes_per_row());

                        // Method 1: Use cursor with extension trait
                        let mut cursor = guard.cursor();
                        if let Ok(pixel) = cursor.read_pixel() {
                            println!("   First pixel (BGRA): {pixel:?}");
                        }

                        // Method 2: Seek to specific coordinates
                        let center_x = guard.width() / 2;
                        let center_y = guard.height() / 2;
                        if cursor
                            .seek_to_pixel(center_x, center_y, guard.bytes_per_row())
                            .is_ok()
                        {
                            if let Ok(pixel) = cursor.read_pixel() {
                                println!("   Center pixel: {pixel:?}");
                            }
                        }

                        // Method 3: Standard Read trait
                        cursor.seek(SeekFrom::Start(0)).ok();
                        let mut buf = [0u8; 16];
                        if cursor.read_exact(&mut buf).is_ok() {
                            println!("   First 16 bytes: {:?}", &buf[..4]);
                        }

                        // Method 4: Direct slice access (fast)
                        let slice = guard.as_slice();
                        println!("   Total bytes: {}", slice.len());
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Pixel Buffer Access\n");

    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    let filter = SCContentFilter::builder()
        .display(&display)
        .exclude_windows(&[])
        .build();

    let mut config = SCStreamConfiguration::default();
    config.set_width(640);
    config.set_height(480);
    config.set_pixel_format(PixelFormat::BGRA);

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
