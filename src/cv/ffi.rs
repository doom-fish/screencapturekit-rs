//! Raw FFI bindings for Core Video types.
//!
//! These are low-level bindings and are not intended for direct use.
#![allow(missing_docs)]

extern "C" {
    // Hash functions
    pub fn cv_pixel_buffer_hash(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_pool_hash(pool: *mut std::ffi::c_void) -> usize;

    pub fn cv_pixel_buffer_get_width(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_get_height(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_get_pixel_format_type(pixel_buffer: *mut std::ffi::c_void) -> u32;
    pub fn cv_pixel_buffer_get_bytes_per_row(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_lock_base_address(
        pixel_buffer: *mut std::ffi::c_void,
        flags: u32,
    ) -> i32;
    pub fn cv_pixel_buffer_unlock_base_address(
        pixel_buffer: *mut std::ffi::c_void,
        flags: u32,
    ) -> i32;
    pub fn cv_pixel_buffer_get_base_address(
        pixel_buffer: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn cv_pixel_buffer_get_io_surface(
        pixel_buffer: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn cv_pixel_buffer_release(pixel_buffer: *mut std::ffi::c_void);
    pub fn cv_pixel_buffer_retain(pixel_buffer: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn cv_pixel_buffer_create(
        width: usize,
        height: usize,
        pixel_format_type: u32,
        pixel_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_create_with_bytes(
        width: usize,
        height: usize,
        pixel_format_type: u32,
        base_address: *mut std::ffi::c_void,
        bytes_per_row: usize,
        pixel_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_fill_extended_pixels(pixel_buffer: *mut std::ffi::c_void) -> i32;

    // Planar APIs
    pub fn cv_pixel_buffer_create_with_planar_bytes(
        width: usize,
        height: usize,
        pixel_format_type: u32,
        num_planes: usize,
        plane_base_addresses: *const *mut std::ffi::c_void,
        plane_widths: *const usize,
        plane_heights: *const usize,
        plane_bytes_per_row: *const usize,
        pixel_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_create_with_io_surface(
        io_surface: *mut std::ffi::c_void,
        pixel_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_get_type_id() -> usize;

    // CVPixelBufferPool APIs
    pub fn cv_pixel_buffer_pool_create(
        width: usize,
        height: usize,
        pixel_format_type: u32,
        max_buffers: usize,
        pool_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_pool_create_pixel_buffer(
        pool: *mut std::ffi::c_void,
        pixel_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cv_pixel_buffer_pool_flush(pool: *mut std::ffi::c_void);
    pub fn cv_pixel_buffer_pool_get_type_id() -> usize;
    pub fn cv_pixel_buffer_pool_retain(pool: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn cv_pixel_buffer_pool_release(pool: *mut std::ffi::c_void);

    // Additional pool APIs
    pub fn cv_pixel_buffer_pool_get_attributes(
        pool: *mut std::ffi::c_void,
    ) -> *const std::ffi::c_void;
    pub fn cv_pixel_buffer_pool_get_pixel_buffer_attributes(
        pool: *mut std::ffi::c_void,
    ) -> *const std::ffi::c_void;

    pub fn cv_pixel_buffer_get_data_size(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_is_planar(pixel_buffer: *mut std::ffi::c_void) -> bool;
    pub fn cv_pixel_buffer_get_plane_count(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_get_width_of_plane(
        pixel_buffer: *mut std::ffi::c_void,
        plane_index: usize,
    ) -> usize;
    pub fn cv_pixel_buffer_get_height_of_plane(
        pixel_buffer: *mut std::ffi::c_void,
        plane_index: usize,
    ) -> usize;
    pub fn cv_pixel_buffer_get_base_address_of_plane(
        pixel_buffer: *mut std::ffi::c_void,
        plane_index: usize,
    ) -> *mut std::ffi::c_void;
    pub fn cv_pixel_buffer_get_bytes_per_row_of_plane(
        pixel_buffer: *mut std::ffi::c_void,
        plane_index: usize,
    ) -> usize;
    pub fn cv_pixel_buffer_get_extended_pixels(
        pixel_buffer: *mut std::ffi::c_void,
        extra_columns_on_left: *mut usize,
        extra_columns_on_right: *mut usize,
        extra_rows_on_top: *mut usize,
        extra_rows_on_bottom: *mut usize,
    );
}
