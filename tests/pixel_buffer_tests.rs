//! Pixel buffer tests

#[cfg(test)]
mod iosurface_plane_tests {
    use screencapturekit::output::iosurface::{IOSurface, IOSurfaceLockOptions, PlaneProperties};

    #[test]
    fn test_ycbcr_iosurface_plane_methods() {
        let width = 64;
        let height = 64;

        let y_bytes_per_row = (width + 15) & !15;
        let y_size = y_bytes_per_row * height;

        let uv_width = width / 2;
        let uv_height = height / 2;
        let uv_bytes_per_row = ((uv_width * 2) + 15) & !15;
        let uv_size = uv_bytes_per_row * uv_height;

        let alloc_size = y_size + uv_size;

        let planes = [
            PlaneProperties {
                width,
                height,
                bytes_per_row: y_bytes_per_row,
                bytes_per_element: 1,
                offset: 0,
                size: y_size,
            },
            PlaneProperties {
                width: uv_width,
                height: uv_height,
                bytes_per_row: uv_bytes_per_row,
                bytes_per_element: 2,
                offset: y_size,
                size: uv_size,
            },
        ];

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x34323076, // '420v'
            1,
            y_bytes_per_row,
            alloc_size,
            Some(&planes),
        ) {
            // Test plane count on IOSurface
            assert_eq!(surface.plane_count(), 2);

            // Test plane 0 (Y)
            assert_eq!(surface.width_of_plane(0), width);
            assert_eq!(surface.height_of_plane(0), height);
            assert!(surface.bytes_per_row_of_plane(0) >= width);

            // Test plane 1 (UV)
            assert_eq!(surface.width_of_plane(1), uv_width);
            assert_eq!(surface.height_of_plane(1), uv_height);
            assert!(surface.bytes_per_row_of_plane(1) >= uv_width * 2);
        }
    }

    #[test]
    fn test_iosurface_lock_guard_as_ptr() {
        let width = 32;
        let height = 32;
        let bytes_per_row = (width * 4 + 15) & !15;
        let alloc_size = bytes_per_row * height;

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x42475241, // BGRA
            4,
            bytes_per_row,
            alloc_size,
            None,
        ) {
            if let Ok(guard) = surface.lock(IOSurfaceLockOptions::ReadOnly) {
                // Test as_ptr
                let ptr = guard.as_ptr();
                assert!(!ptr.is_null());

                // Test dimensions
                assert_eq!(guard.width(), width);
                assert_eq!(guard.height(), height);
                assert!(guard.bytes_per_row() >= width * 4);
            }

            // Test mutable access with AvoidSync (allows write)
            if let Ok(mut guard) = surface.lock(IOSurfaceLockOptions::AvoidSync) {
                let mut_ptr = guard.as_mut_ptr();
                assert!(!mut_ptr.is_null());
            }
        }
    }

    #[test]
    fn test_single_plane_surface() {
        let width = 64;
        let height = 64;
        let bytes_per_row = (width * 4 + 15) & !15;
        let alloc_size = bytes_per_row * height;

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x42475241, // BGRA
            4,
            bytes_per_row,
            alloc_size,
            None,
        ) {
            // Single plane surface
            assert_eq!(surface.plane_count(), 0);
        }
    }

    #[test]
    fn test_lock_guard_plane_access() {
        let width = 64;
        let height = 64;

        let y_bytes_per_row = (width + 15) & !15;
        let y_size = y_bytes_per_row * height;

        let uv_width = width / 2;
        let uv_height = height / 2;
        let uv_bytes_per_row = ((uv_width * 2) + 15) & !15;
        let uv_size = uv_bytes_per_row * uv_height;

        let alloc_size = y_size + uv_size;

        let planes = [
            PlaneProperties {
                width,
                height,
                bytes_per_row: y_bytes_per_row,
                bytes_per_element: 1,
                offset: 0,
                size: y_size,
            },
            PlaneProperties {
                width: uv_width,
                height: uv_height,
                bytes_per_row: uv_bytes_per_row,
                bytes_per_element: 2,
                offset: y_size,
                size: uv_size,
            },
        ];

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x34323076, // '420v'
            1,
            y_bytes_per_row,
            alloc_size,
            Some(&planes),
        ) {
            if let Ok(guard) = surface.lock(IOSurfaceLockOptions::ReadOnly) {
                // Test base_address_of_plane
                let plane0_addr = guard.base_address_of_plane(0);
                assert!(plane0_addr.is_some());
                assert!(!plane0_addr.unwrap().is_null());

                let plane1_addr = guard.base_address_of_plane(1);
                assert!(plane1_addr.is_some());
                assert!(!plane1_addr.unwrap().is_null());

                // Out of bounds should return None
                assert!(guard.base_address_of_plane(2).is_none());
                assert!(guard.base_address_of_plane(100).is_none());

                // Test plane_data
                let plane0_data = guard.plane_data(0);
                assert!(plane0_data.is_some());
                let data = plane0_data.unwrap();
                assert!(data.len() >= y_size);

                let plane1_data = guard.plane_data(1);
                assert!(plane1_data.is_some());
                let data = plane1_data.unwrap();
                assert!(data.len() >= uv_size);

                // Out of bounds
                assert!(guard.plane_data(2).is_none());

                // Test plane_row
                let row0 = guard.plane_row(0, 0);
                assert!(row0.is_some());
                assert!(row0.unwrap().len() >= width);

                let row_last = guard.plane_row(0, height - 1);
                assert!(row_last.is_some());

                // Out of bounds row
                assert!(guard.plane_row(0, height).is_none());
                assert!(guard.plane_row(0, height + 100).is_none());

                // Out of bounds plane
                assert!(guard.plane_row(2, 0).is_none());

                // Plane 1 row access
                let uv_row = guard.plane_row(1, 0);
                assert!(uv_row.is_some());
                assert!(uv_row.unwrap().len() >= uv_width * 2);
            }
        }
    }

    #[test]
    fn test_lock_guard_plane_no_planes() {
        // Single-plane surface should return None for plane methods
        let width = 32;
        let height = 32;
        let bytes_per_row = (width * 4 + 15) & !15;
        let alloc_size = bytes_per_row * height;

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x42475241, // BGRA - single plane
            4,
            bytes_per_row,
            alloc_size,
            None,
        ) {
            if let Ok(guard) = surface.lock(IOSurfaceLockOptions::ReadOnly) {
                // Single-plane surfaces have plane_count() == 0
                // So plane methods should return None
                assert!(guard.base_address_of_plane(0).is_none());
                assert!(guard.plane_data(0).is_none());
                assert!(guard.plane_row(0, 0).is_none());
            }
        }
    }
}
