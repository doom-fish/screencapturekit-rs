//! `IOSurface` and pixel buffer tests

#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::similar_names)]

use screencapturekit::cm::{IOSurface, IOSurfaceLockOptions};
use screencapturekit::cv::CVPixelBufferLockFlags;

#[test]
fn test_iosurface_lock_options_values() {
    assert_eq!(IOSurfaceLockOptions::READ_ONLY.as_u32(), 0x0000_0001);
    assert_eq!(IOSurfaceLockOptions::AVOID_SYNC.as_u32(), 0x0000_0002);
}

#[test]
fn test_iosurface_lock_options_combined() {
    let combined = IOSurfaceLockOptions::READ_ONLY | IOSurfaceLockOptions::AVOID_SYNC;
    assert_eq!(combined.as_u32(), 0x0000_0003);
    assert!(combined.contains(IOSurfaceLockOptions::READ_ONLY));
    assert!(combined.contains(IOSurfaceLockOptions::AVOID_SYNC));
    assert!(combined.is_read_only());
}

#[test]
fn test_iosurface_lock_options_debug() {
    let opt = IOSurfaceLockOptions::READ_ONLY;
    let debug_str = format!("{opt:?}");
    assert!(debug_str.contains("IOSurfaceLockOptions"));
}

#[test]
fn test_cvpixelbuffer_lock_flags_values() {
    assert_eq!(CVPixelBufferLockFlags::READ_ONLY.as_u32(), 0x0000_0001);
    assert_eq!(CVPixelBufferLockFlags::NONE.as_u32(), 0x0000_0000);
    assert!(CVPixelBufferLockFlags::READ_ONLY.is_read_only());
    assert!(!CVPixelBufferLockFlags::NONE.is_read_only());
}

#[test]
fn test_iosurface_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<IOSurface>();
    assert_sync::<IOSurface>();
}

#[test]
fn test_iosurface_create() {
    let surface = IOSurface::create(100, 100, 0x42475241, 4).expect("Failed to create IOSurface");

    assert_eq!(surface.width(), 100);
    assert_eq!(surface.height(), 100);
    assert!(surface.bytes_per_row() >= 400); // At least 100 * 4
    assert_eq!(surface.pixel_format(), 0x42475241);
}

#[test]
fn test_iosurface_plane_count() {
    let surface = IOSurface::create(100, 100, 0x42475241, 4).expect("Failed to create IOSurface");

    // BGRA is single-plane
    assert_eq!(surface.plane_count(), 0);
}

#[test]
fn test_iosurface_is_in_use() {
    let surface = IOSurface::create(50, 50, 0x42475241, 4).expect("Failed to create IOSurface");
    // Initially not in use
    let _ = surface.is_in_use();
}

#[test]
fn test_iosurface_lock_and_access() {
    let surface = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::READ_ONLY)
        .expect("Failed to lock IOSurface");

    assert_eq!(guard.width(), 10);
    assert_eq!(guard.height(), 10);
    assert!(guard.bytes_per_row() >= 40);

    // Test as_slice
    let slice = guard.as_slice();
    assert!(!slice.is_empty());
}

#[test]
fn test_iosurface_lock_guard_row() {
    let surface = IOSurface::create(20, 20, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::READ_ONLY)
        .expect("Failed to lock IOSurface");

    // Valid row
    let row0 = guard.row(0);
    assert!(row0.is_some());
    assert_eq!(row0.unwrap().len(), guard.bytes_per_row());

    let row19 = guard.row(19);
    assert!(row19.is_some());

    // Invalid row
    let row20 = guard.row(20);
    assert!(row20.is_none());
}

#[test]
fn test_iosurface_lock_guard_cursor() {
    use std::io::Read;

    let surface = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::READ_ONLY)
        .expect("Failed to lock IOSurface");

    let mut cursor = guard.cursor();
    let mut buf = [0u8; 4];
    let result = cursor.read_exact(&mut buf);
    assert!(result.is_ok());
}

#[test]
fn test_iosurface_lock_guard_debug() {
    let surface = IOSurface::create(50, 50, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::READ_ONLY)
        .expect("Failed to lock IOSurface");

    let debug_str = format!("{guard:?}");
    assert!(debug_str.contains("IOSurfaceLockGuard"));
    assert!(debug_str.contains("options"));
    assert!(debug_str.contains("surface_size"));
}

#[test]
fn test_iosurface_debug() {
    let surface = IOSurface::create(100, 100, 0x42475241, 4).expect("Failed to create IOSurface");

    let debug_str = format!("{surface:?}");
    assert!(debug_str.contains("IOSurface"));
    assert!(debug_str.contains("width"));
    assert!(debug_str.contains("height"));
}

#[test]
fn test_iosurface_as_ptr() {
    let surface = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");
    let ptr = surface.as_ptr();
    assert!(!ptr.is_null());
}

#[test]
fn test_pixel_buffer_create_and_lock() {
    use screencapturekit::cv::CVPixelBuffer;

    // Create a test pixel buffer (BGRA format)
    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    assert_eq!(buffer.width(), 100);
    assert_eq!(buffer.height(), 100);

    // Lock for reading
    let guard = buffer.lock(CVPixelBufferLockFlags::READ_ONLY);
    assert!(guard.is_ok());

    let guard = guard.unwrap();
    assert_eq!(guard.width(), 100);
    assert_eq!(guard.height(), 100);
    assert!(guard.bytes_per_row() >= 400); // At least 100 * 4 bytes

    // Test as_slice
    let slice = guard.as_slice();
    assert!(!slice.is_empty());

    // Test Deref trait
    let _: &[u8] = &guard;
}

#[test]
fn test_pixel_buffer_lock_guard_row_access() {
    use screencapturekit::cv::CVPixelBuffer;

    let buffer = CVPixelBuffer::create(50, 50, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .expect("Failed to lock buffer");

    // Valid row access
    let row0 = guard.row(0);
    assert!(row0.is_some());
    assert_eq!(row0.unwrap().len(), guard.bytes_per_row());

    let row49 = guard.row(49);
    assert!(row49.is_some());

    // Invalid row access
    let row50 = guard.row(50);
    assert!(row50.is_none());

    let row_overflow = guard.row(1000);
    assert!(row_overflow.is_none());
}

#[test]
fn test_pixel_buffer_lock_guard_cursor() {
    use screencapturekit::cv::CVPixelBuffer;
    use std::io::Read;

    let buffer = CVPixelBuffer::create(10, 10, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .expect("Failed to lock buffer");

    let mut cursor = guard.cursor();
    let mut buf = [0u8; 4];
    let result = cursor.read_exact(&mut buf);
    assert!(result.is_ok());
}

#[test]
fn test_pixel_buffer_lock_guard_debug() {
    use screencapturekit::cv::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .expect("Failed to lock buffer");

    let debug_str = format!("{guard:?}");
    assert!(debug_str.contains("CVPixelBufferLockGuard"));
    assert!(debug_str.contains("flags"));
    assert!(debug_str.contains("buffer_size"));
}

#[test]
fn test_pixel_buffer_iosurface_trait() {
    use screencapturekit::cv::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    // Simple pixel buffers might not be backed by IOSurface
    let is_backed = buffer.is_backed_by_io_surface();
    // Just test that the method works, result depends on implementation
    let _ = is_backed;

    let iosurface = buffer.io_surface();
    // iosurface may or may not be available depending on buffer creation
    let _ = iosurface;
}

#[test]
fn test_pixel_buffer_planar_methods() {
    use screencapturekit::cv::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .expect("Failed to lock buffer");

    // BGRA is not planar, so plane_count should be 0
    let plane_count = guard.plane_count();
    assert_eq!(plane_count, 0);
}

#[test]
fn test_pixel_buffer_cursor_extension_trait() {
    use screencapturekit::cv::CVPixelBuffer;
    use screencapturekit::cv::PixelBufferCursorExt;
    use std::io::{Seek, SeekFrom};

    let buffer = CVPixelBuffer::create(10, 10, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .expect("Failed to lock buffer");

    let mut cursor = guard.cursor();

    // Test seek_to_pixel
    let bytes_per_row = guard.bytes_per_row();
    let pos = cursor.seek_to_pixel(0, 0, bytes_per_row);
    assert!(pos.is_ok());
    assert_eq!(pos.unwrap(), 0);

    // Test read_pixel
    cursor.seek(SeekFrom::Start(0)).unwrap();
    let pixel = cursor.read_pixel();
    assert!(pixel.is_ok());
    let pixel = pixel.unwrap();
    assert_eq!(pixel.len(), 4);
}

#[test]
fn test_iosurface_lock_guard_as_ptr() {
    let surface = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::READ_ONLY)
        .expect("Failed to lock IOSurface");

    let ptr = guard.as_ptr();
    assert!(!ptr.is_null());
}

#[test]
fn test_iosurface_equality() {
    let surface1 = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");
    let surface2 = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");

    // Different surfaces should not be equal (different pointers)
    assert_ne!(surface1, surface2);

    // Same surface compared to itself
    assert_eq!(surface1, surface1);
}

#[test]
fn test_iosurface_hash() {
    use std::collections::HashSet;

    let surface1 = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");
    let surface2 = IOSurface::create(10, 10, 0x42475241, 4).expect("Failed to create IOSurface");

    let mut set = HashSet::new();
    set.insert(&surface1);

    assert!(set.contains(&surface1));
    assert!(!set.contains(&surface2));
}

// ============================================================================
// Multi-planar IOSurface tests (YCbCr, L10R)
// ============================================================================

mod multiplanar_tests {
    use screencapturekit::cm::IOSurface;
    use screencapturekit::cm::PlaneProperties;

    /// Create a YCbCr 4:2:0 biplanar surface (like 420v or 420f)
    fn create_ycbcr_420(width: usize, height: usize, full_range: bool) -> Option<IOSurface> {
        // 420v = 0x34323076, 420f = 0x34323066
        let pixel_format: u32 = if full_range { 0x34323066 } else { 0x34323076 };

        // Align dimensions to even for 4:2:0
        let aligned_width = (width + 1) & !1;
        let aligned_height = (height + 1) & !1;

        // Plane 0: Y (luminance) - full resolution, 1 byte per pixel
        let plane0_width = aligned_width;
        let plane0_height = aligned_height;
        let plane0_bpr = (plane0_width + 15) & !15; // 16-byte aligned
        let plane0_size = plane0_bpr * plane0_height;

        // Plane 1: CbCr (chrominance) - half resolution, 2 bytes per pixel
        let plane1_width = aligned_width / 2;
        let plane1_height = aligned_height / 2;
        let plane1_bpr = (plane1_width * 2 + 15) & !15; // 16-byte aligned
        let plane1_size = plane1_bpr * plane1_height;

        let total_size = plane0_size + plane1_size;

        let planes = [
            PlaneProperties {
                width: plane0_width,
                height: plane0_height,
                bytes_per_row: plane0_bpr,
                bytes_per_element: 1,
                offset: 0,
                size: plane0_size,
            },
            PlaneProperties {
                width: plane1_width,
                height: plane1_height,
                bytes_per_row: plane1_bpr,
                bytes_per_element: 2,
                offset: plane0_size,
                size: plane1_size,
            },
        ];

        IOSurface::create_with_properties(
            aligned_width,
            aligned_height,
            pixel_format,
            1, // bytes_per_element for plane 0
            plane0_bpr,
            total_size,
            Some(&planes),
        )
    }

    #[test]
    fn test_ycbcr_420v_creation() {
        let surface = create_ycbcr_420(1920, 1080, false).expect("Failed to create 420v surface");

        assert_eq!(surface.width(), 1920);
        assert_eq!(surface.height(), 1080);
        assert_eq!(surface.pixel_format(), 0x34323076); // '420v'
        assert_eq!(surface.plane_count(), 2);
    }

    #[test]
    fn test_ycbcr_420f_creation() {
        let surface = create_ycbcr_420(1920, 1080, true).expect("Failed to create 420f surface");

        assert_eq!(surface.pixel_format(), 0x34323066); // '420f'
        assert_eq!(surface.plane_count(), 2);
    }

    #[test]
    fn test_ycbcr_plane_dimensions() {
        let surface = create_ycbcr_420(1920, 1080, false).expect("Failed to create 420v surface");

        // Plane 0: Y - full resolution
        assert_eq!(surface.width_of_plane(0), 1920);
        assert_eq!(surface.height_of_plane(0), 1080);

        // Plane 1: CbCr - half resolution
        assert_eq!(surface.width_of_plane(1), 960);
        assert_eq!(surface.height_of_plane(1), 540);
    }

    #[test]
    fn test_ycbcr_bytes_per_row() {
        let surface = create_ycbcr_420(1920, 1080, false).expect("Failed to create 420v surface");

        // Plane 0: Y - 1 byte per pixel, 16-byte aligned
        let plane0_bpr = surface.bytes_per_row_of_plane(0);
        assert!(plane0_bpr >= 1920);
        assert_eq!(plane0_bpr % 16, 0);

        // Plane 1: CbCr - 2 bytes per pixel, 16-byte aligned
        let plane1_bpr = surface.bytes_per_row_of_plane(1);
        assert!(plane1_bpr >= 960 * 2);
        assert_eq!(plane1_bpr % 16, 0);
    }

    #[test]
    fn test_ycbcr_is_biplanar() {
        let surface = create_ycbcr_420(640, 480, false).expect("Failed to create surface");

        // The is_ycbcr_biplanar check uses pixel_format
        use screencapturekit::metal::pixel_format;
        assert!(pixel_format::is_ycbcr_biplanar(surface.pixel_format()));
    }

    #[test]
    fn test_ycbcr_texture_params() {
        let surface = create_ycbcr_420(1920, 1080, false).expect("Failed to create surface");

        let params = surface.texture_params();
        assert_eq!(params.len(), 2);

        // Plane 0: Y
        assert_eq!(params[0].width, 1920);
        assert_eq!(params[0].height, 1080);
        assert_eq!(params[0].plane, 0);

        // Plane 1: CbCr
        assert_eq!(params[1].width, 960);
        assert_eq!(params[1].height, 540);
        assert_eq!(params[1].plane, 1);
    }

    #[test]
    fn test_ycbcr_info() {
        let surface = create_ycbcr_420(640, 480, false).expect("Failed to create surface");

        let info = surface.info();
        assert_eq!(info.width, 640);
        assert_eq!(info.height, 480);
        assert_eq!(info.plane_count, 2);
        assert_eq!(info.planes.len(), 2);

        // Plane 0
        assert_eq!(info.planes[0].index, 0);
        assert_eq!(info.planes[0].width, 640);
        assert_eq!(info.planes[0].height, 480);

        // Plane 1
        assert_eq!(info.planes[1].index, 1);
        assert_eq!(info.planes[1].width, 320);
        assert_eq!(info.planes[1].height, 240);
    }

    #[test]
    fn test_l10r_creation() {
        // L10R = 'l10r' = 0x6c313072, 4 bytes per pixel (10+10+10+2 bits)
        let width = 1920usize;
        let height = 1080usize;
        let bytes_per_element = 4usize;
        let bytes_per_row = (width * bytes_per_element + 15) & !15;
        let alloc_size = bytes_per_row * height;

        let surface = IOSurface::create_with_properties(
            width,
            height,
            0x6c313072, // 'l10r'
            bytes_per_element,
            bytes_per_row,
            alloc_size,
            None,
        )
        .expect("Failed to create L10R surface");

        assert_eq!(surface.width(), 1920);
        assert_eq!(surface.height(), 1080);
        assert_eq!(surface.pixel_format(), 0x6c313072);
        assert_eq!(surface.plane_count(), 0); // Single-plane
    }

    #[test]
    fn test_l10r_texture_params() {
        let width = 1920usize;
        let height = 1080usize;
        let bytes_per_element = 4usize;
        let bytes_per_row = (width * bytes_per_element + 15) & !15;
        let alloc_size = bytes_per_row * height;

        let surface = IOSurface::create_with_properties(
            width,
            height,
            0x6c313072,
            bytes_per_element,
            bytes_per_row,
            alloc_size,
            None,
        )
        .expect("Failed to create L10R surface");

        let params = surface.texture_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].width, 1920);
        assert_eq!(params[0].height, 1080);

        // L10R should use BGR10A2Unorm
        use screencapturekit::metal::MetalPixelFormat;
        assert_eq!(params[0].format, MetalPixelFormat::BGR10A2Unorm);
    }

    #[test]
    fn test_l10r_not_ycbcr() {
        use screencapturekit::metal::pixel_format;
        assert!(!pixel_format::is_ycbcr_biplanar(0x6c313072));
    }

    #[test]
    fn test_plane_properties_debug() {
        let props = PlaneProperties {
            width: 1920,
            height: 1080,
            bytes_per_row: 1920,
            bytes_per_element: 1,
            offset: 0,
            size: 1920 * 1080,
        };

        let debug_str = format!("{props:?}");
        assert!(debug_str.contains("PlaneProperties"));
        assert!(debug_str.contains("width"));
    }

    #[test]
    fn test_plane_properties_clone_eq() {
        let props = PlaneProperties {
            width: 100,
            height: 100,
            bytes_per_row: 128,
            bytes_per_element: 1,
            offset: 0,
            size: 12800,
        };

        let cloned = props;
        assert_eq!(props, cloned);
    }
}
