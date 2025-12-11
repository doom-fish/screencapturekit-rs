//! IOSurface and pixel buffer tests

use screencapturekit::output::{
    CVImageBufferLockExt, CVPixelBufferIOSurface, IOSurface, IOSurfaceLockOptions,
    PixelBufferLockFlags,
};

#[test]
fn test_iosurface_lock_options_values() {
    assert_eq!(IOSurfaceLockOptions::ReadOnly.as_u32(), 0x0000_0001);
    assert_eq!(IOSurfaceLockOptions::AvoidSync.as_u32(), 0x0000_0002);
}

#[test]
fn test_iosurface_lock_options_debug() {
    let opt = IOSurfaceLockOptions::ReadOnly;
    let debug_str = format!("{:?}", opt);
    assert!(debug_str.contains("ReadOnly"));
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
        .lock(IOSurfaceLockOptions::ReadOnly)
        .expect("Failed to lock IOSurface");

    assert_eq!(guard.width(), 10);
    assert_eq!(guard.height(), 10);
    assert!(guard.bytes_per_row() >= 40);

    // Test as_slice
    let slice = guard.as_slice();
    assert!(!slice.is_empty());

    // Test Deref
    let _: &[u8] = &guard;
}

#[test]
fn test_iosurface_lock_guard_row() {
    let surface = IOSurface::create(20, 20, 0x42475241, 4).expect("Failed to create IOSurface");

    let guard = surface
        .lock(IOSurfaceLockOptions::ReadOnly)
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
        .lock(IOSurfaceLockOptions::ReadOnly)
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
        .lock(IOSurfaceLockOptions::ReadOnly)
        .expect("Failed to lock IOSurface");

    let debug_str = format!("{:?}", guard);
    assert!(debug_str.contains("IOSurfaceLockGuard"));
    assert!(debug_str.contains("width"));
    assert!(debug_str.contains("height"));
}

#[test]
fn test_iosurface_debug() {
    let surface = IOSurface::create(100, 100, 0x42475241, 4).expect("Failed to create IOSurface");

    let debug_str = format!("{:?}", surface);
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
fn test_pixel_buffer_lock_flags_values() {
    assert_eq!(PixelBufferLockFlags::ReadOnly.as_u64(), 0x0000_0001);
    assert_eq!(PixelBufferLockFlags::ReadOnly.as_u32(), 0x0000_0001);
}

#[test]
fn test_pixel_buffer_lock_flags_debug() {
    let flags = PixelBufferLockFlags::ReadOnly;
    let debug_str = format!("{:?}", flags);
    assert!(debug_str.contains("ReadOnly"));
}

#[test]
fn test_pixel_buffer_create_and_lock() {
    use screencapturekit::cm::CVPixelBuffer;

    // Create a test pixel buffer (BGRA format)
    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    assert_eq!(buffer.width(), 100);
    assert_eq!(buffer.height(), 100);

    // Lock for reading
    let guard = buffer.lock(PixelBufferLockFlags::ReadOnly);
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
    use screencapturekit::cm::CVPixelBuffer;

    let buffer = CVPixelBuffer::create(50, 50, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(PixelBufferLockFlags::ReadOnly)
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
    use screencapturekit::cm::CVPixelBuffer;
    use std::io::Read;

    let buffer = CVPixelBuffer::create(10, 10, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(PixelBufferLockFlags::ReadOnly)
        .expect("Failed to lock buffer");

    let mut cursor = guard.cursor();
    let mut buf = [0u8; 4];
    let result = cursor.read_exact(&mut buf);
    assert!(result.is_ok());
}

#[test]
fn test_pixel_buffer_lock_guard_debug() {
    use screencapturekit::cm::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(PixelBufferLockFlags::ReadOnly)
        .expect("Failed to lock buffer");

    let debug_str = format!("{:?}", guard);
    assert!(debug_str.contains("PixelBufferLockGuard"));
    assert!(debug_str.contains("width"));
    assert!(debug_str.contains("height"));
}

#[test]
fn test_pixel_buffer_iosurface_trait() {
    use screencapturekit::cm::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    // Simple pixel buffers might not be backed by IOSurface
    let is_backed = buffer.is_backed_by_iosurface();
    // Just test that the method works, result depends on implementation
    let _ = is_backed;

    let iosurface = buffer.iosurface();
    // iosurface may or may not be available depending on buffer creation
    let _ = iosurface;
}

#[test]
fn test_pixel_buffer_planar_methods() {
    use screencapturekit::cm::CVPixelBuffer;

    let buffer =
        CVPixelBuffer::create(100, 100, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(PixelBufferLockFlags::ReadOnly)
        .expect("Failed to lock buffer");

    // BGRA is not planar, so plane_count should be 0
    let plane_count = guard.plane_count();
    assert_eq!(plane_count, 0);
}

#[test]
fn test_pixel_buffer_cursor_extension_trait() {
    use screencapturekit::cm::CVPixelBuffer;
    use screencapturekit::output::PixelBufferCursorExt;
    use std::io::{Seek, SeekFrom};

    let buffer = CVPixelBuffer::create(10, 10, 0x42475241).expect("Failed to create pixel buffer");

    let guard = buffer
        .lock(PixelBufferLockFlags::ReadOnly)
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
        .lock(IOSurfaceLockOptions::ReadOnly)
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
