//! Raw FFI bindings for Core Media types.
//!
//! These are low-level bindings and are not intended for direct use.
#![allow(missing_docs)]

extern "C" {
    pub fn cm_sample_buffer_get_image_buffer(
        sample_buffer: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn cm_sample_buffer_get_frame_status(sample_buffer: *mut std::ffi::c_void) -> i32;

    // Frame info accessors
    pub fn cm_sample_buffer_get_display_time(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut u64,
    ) -> bool;
    pub fn cm_sample_buffer_get_scale_factor(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut f64,
    ) -> bool;
    pub fn cm_sample_buffer_get_content_scale(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut f64,
    ) -> bool;
    pub fn cm_sample_buffer_get_content_rect(
        sample_buffer: *mut std::ffi::c_void,
        out_x: *mut f64,
        out_y: *mut f64,
        out_width: *mut f64,
        out_height: *mut f64,
    ) -> bool;
    pub fn cm_sample_buffer_get_bounding_rect(
        sample_buffer: *mut std::ffi::c_void,
        out_x: *mut f64,
        out_y: *mut f64,
        out_width: *mut f64,
        out_height: *mut f64,
    ) -> bool;
    pub fn cm_sample_buffer_get_screen_rect(
        sample_buffer: *mut std::ffi::c_void,
        out_x: *mut f64,
        out_y: *mut f64,
        out_width: *mut f64,
        out_height: *mut f64,
    ) -> bool;
    pub fn cm_sample_buffer_get_dirty_rects(
        sample_buffer: *mut std::ffi::c_void,
        out_rects: *mut *mut std::ffi::c_void,
        out_count: *mut usize,
    ) -> bool;
    pub fn cm_sample_buffer_free_dirty_rects(rects_ptr: *mut std::ffi::c_void);

    pub fn cm_sample_buffer_get_presentation_timestamp(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut i64,
        out_timescale: *mut i32,
        out_flags: *mut u32,
        out_epoch: *mut i64,
    );
    pub fn cm_sample_buffer_get_duration(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut i64,
        out_timescale: *mut i32,
        out_flags: *mut u32,
        out_epoch: *mut i64,
    );
    pub fn cm_sample_buffer_release(sample_buffer: *mut std::ffi::c_void);
    pub fn cm_sample_buffer_retain(sample_buffer: *mut std::ffi::c_void);
    pub fn cm_sample_buffer_is_valid(sample_buffer: *mut std::ffi::c_void) -> bool;
    pub fn cm_sample_buffer_get_num_samples(sample_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cm_sample_buffer_get_audio_buffer_list(
        sample_buffer: *mut std::ffi::c_void,
        out_num_buffers: *mut u32,
        out_buffers_ptr: *mut *mut std::ffi::c_void,
        out_buffers_len: *mut usize,
        out_block_buffer: *mut *mut std::ffi::c_void,
    );
    pub fn cm_block_buffer_release(block_buffer: *mut std::ffi::c_void);
    pub fn cm_sample_buffer_get_data_buffer(
        sample_buffer: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;

    pub fn cm_sample_buffer_get_decode_timestamp(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut i64,
        out_timescale: *mut i32,
        out_flags: *mut u32,
        out_epoch: *mut i64,
    );
    pub fn cm_sample_buffer_get_output_presentation_timestamp(
        sample_buffer: *mut std::ffi::c_void,
        out_value: *mut i64,
        out_timescale: *mut i32,
        out_flags: *mut u32,
        out_epoch: *mut i64,
    );
    pub fn cm_sample_buffer_set_output_presentation_timestamp(
        sample_buffer: *mut std::ffi::c_void,
        value: i64,
        timescale: i32,
        flags: u32,
        epoch: i64,
    ) -> i32;
    pub fn cm_sample_buffer_get_sample_size(
        sample_buffer: *mut std::ffi::c_void,
        sample_index: usize,
    ) -> usize;
    pub fn cm_sample_buffer_get_total_sample_size(sample_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cm_sample_buffer_is_ready_for_data_access(sample_buffer: *mut std::ffi::c_void) -> bool;
    pub fn cm_sample_buffer_make_data_ready(sample_buffer: *mut std::ffi::c_void) -> i32;

    // New CMSampleBuffer APIs
    pub fn cm_sample_buffer_get_format_description(
        sample_buffer: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn cm_sample_buffer_get_sample_timing_info(
        sample_buffer: *mut std::ffi::c_void,
        sample_index: usize,
        out_duration_value: *mut i64,
        out_duration_timescale: *mut i32,
        out_duration_flags: *mut u32,
        out_duration_epoch: *mut i64,
        out_pts_value: *mut i64,
        out_pts_timescale: *mut i32,
        out_pts_flags: *mut u32,
        out_pts_epoch: *mut i64,
        out_dts_value: *mut i64,
        out_dts_timescale: *mut i32,
        out_dts_flags: *mut u32,
        out_dts_epoch: *mut i64,
    ) -> i32;
    pub fn cm_sample_buffer_invalidate(sample_buffer: *mut std::ffi::c_void) -> i32;
    pub fn cm_sample_buffer_create_copy_with_new_timing(
        sample_buffer: *mut std::ffi::c_void,
        num_timing_infos: usize,
        timing_info_array: *const std::ffi::c_void,
        sample_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn cm_sample_buffer_copy_pcm_data_into_audio_buffer_list(
        sample_buffer: *mut std::ffi::c_void,
        frame_offset: i32,
        num_frames: i32,
        buffer_list: *mut std::ffi::c_void,
    ) -> i32;

    // CMFormatDescription APIs
    pub fn cm_format_description_get_media_type(format_description: *mut std::ffi::c_void) -> u32;
    pub fn cm_format_description_get_media_subtype(
        format_description: *mut std::ffi::c_void,
    ) -> u32;
    pub fn cm_format_description_get_extensions(
        format_description: *mut std::ffi::c_void,
    ) -> *const std::ffi::c_void;
    pub fn cm_format_description_retain(
        format_description: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn cm_format_description_release(format_description: *mut std::ffi::c_void);

    // Hash functions
    pub fn cm_sample_buffer_hash(sample_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_hash(pixel_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cv_pixel_buffer_pool_hash(pool: *mut std::ffi::c_void) -> usize;
    pub fn cm_block_buffer_hash(block_buffer: *mut std::ffi::c_void) -> usize;
    pub fn cm_format_description_hash(format_description: *mut std::ffi::c_void) -> usize;
    pub fn io_surface_hash(surface: *mut std::ffi::c_void) -> usize;

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

    // New CVPixelBuffer APIs
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

    pub fn cm_sample_buffer_create_for_image_buffer(
        image_buffer: *mut std::ffi::c_void,
        presentation_time_value: i64,
        presentation_time_scale: i32,
        duration_value: i64,
        duration_scale: i32,
        sample_buffer_out: *mut *mut std::ffi::c_void,
    ) -> i32;

    pub fn io_surface_get_width(surface: *mut std::ffi::c_void) -> usize;
    pub fn io_surface_get_height(surface: *mut std::ffi::c_void) -> usize;
    pub fn io_surface_get_bytes_per_row(surface: *mut std::ffi::c_void) -> usize;
    pub fn io_surface_release(surface: *mut std::ffi::c_void);
    pub fn io_surface_retain(surface: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}
