//! Swift FFI bridge to `ScreenCaptureKit`
use std::ffi::c_void;

// MARK: - FFI Packed Data Structures

/// Packed CGRect for efficient FFI transfer (32 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FFIRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Packed display data for batch retrieval (48 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FFIDisplayData {
    pub display_id: u32,
    pub width: i32,
    pub height: i32,
    pub frame: FFIRect,
}

/// Packed window data for batch retrieval
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FFIWindowData {
    pub window_id: u32,
    pub window_layer: i32,
    pub is_on_screen: bool,
    pub is_active: bool,
    pub frame: FFIRect,
    pub title_offset: u32,
    pub title_length: u32,
    pub owning_app_index: i32,
    pub _padding: i32,
}

/// Packed application data for batch retrieval
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FFIApplicationData {
    pub process_id: i32,
    pub _padding: i32,
    pub bundle_id_offset: u32,
    pub bundle_id_length: u32,
    pub app_name_offset: u32,
    pub app_name_length: u32,
}

// MARK: - CoreGraphics Initialization
extern "C" {
    /// Force CoreGraphics initialization by calling `CGMainDisplayID`
    /// This prevents `CGS_REQUIRE_INIT` crashes on headless systems
    pub fn sc_initialize_core_graphics();
}

// MARK: - SCShareableContent
extern "C" {
    /// Synchronous blocking call to get shareable content
    /// Returns content pointer on success, or writes error to `error_buffer`
    pub fn sc_shareable_content_get_sync(
        exclude_desktop_windows: bool,
        on_screen_windows_only: bool,
        error_buffer: *mut i8,
        error_buffer_size: isize,
    ) -> *const c_void;

    /// Async callback-based shareable content retrieval with options
    pub fn sc_shareable_content_get_with_options(
        exclude_desktop_windows: bool,
        on_screen_windows_only: bool,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );

    pub fn sc_shareable_content_get(
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_shareable_content_get_current_process_displays(
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_shareable_content_get_below_window(
        exclude_desktop_windows: bool,
        reference_window: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_shareable_content_get_above_window(
        exclude_desktop_windows: bool,
        reference_window: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_shareable_content_retain(content: *const c_void) -> *const c_void;
    pub fn sc_shareable_content_release(content: *const c_void);
    pub fn sc_shareable_content_get_displays_count(content: *const c_void) -> isize;
    pub fn sc_shareable_content_get_display_at(
        content: *const c_void,
        index: isize,
    ) -> *const c_void;
    pub fn sc_shareable_content_get_windows_count(content: *const c_void) -> isize;
    pub fn sc_shareable_content_get_window_at(
        content: *const c_void,
        index: isize,
    ) -> *const c_void;
    pub fn sc_shareable_content_get_applications_count(content: *const c_void) -> isize;
    pub fn sc_shareable_content_get_application_at(
        content: *const c_void,
        index: isize,
    ) -> *const c_void;

    // Batch retrieval functions (optimized FFI)
    pub fn sc_shareable_content_get_displays_batch(
        content: *const c_void,
        buffer: *mut c_void, // Actually *mut FFIDisplayData
        max_displays: isize,
    ) -> isize;

    pub fn sc_shareable_content_get_applications_batch(
        content: *const c_void,
        buffer: *mut c_void, // Actually *mut FFIApplicationData
        max_apps: isize,
        string_buffer: *mut i8,
        string_buffer_size: isize,
        string_buffer_used: *mut isize,
    ) -> isize;

    pub fn sc_shareable_content_get_windows_batch(
        content: *const c_void,
        buffer: *mut c_void, // Actually *mut FFIWindowData
        max_windows: isize,
        string_buffer: *mut i8,
        string_buffer_size: isize,
        string_buffer_used: *mut isize,
        app_pointers: *mut *const c_void,
        max_apps: isize,
        app_count: *mut isize,
    ) -> isize;
}

// MARK: - SCDisplay
extern "C" {
    pub fn sc_display_retain(display: *const c_void) -> *const c_void;
    pub fn sc_display_release(display: *const c_void);
    pub fn sc_display_get_display_id(display: *const c_void) -> u32;
    pub fn sc_display_get_width(display: *const c_void) -> isize;
    pub fn sc_display_get_height(display: *const c_void) -> isize;
    pub fn sc_display_get_frame(
        display: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    /// Get display frame (same as sc_display_get_frame, kept for API compatibility)
    pub fn sc_display_get_frame_packed(
        display: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
}

// MARK: - SCWindow
extern "C" {
    pub fn sc_window_retain(window: *const c_void) -> *const c_void;
    pub fn sc_window_release(window: *const c_void);
    pub fn sc_window_get_window_id(window: *const c_void) -> u32;
    pub fn sc_window_get_frame(
        window: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    /// Get window frame (same as sc_window_get_frame, kept for API compatibility)
    pub fn sc_window_get_frame_packed(
        window: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    pub fn sc_window_get_title(window: *const c_void, buffer: *mut i8, buffer_size: isize) -> bool;
    /// Get window title as owned string (caller must free with sc_free_string)
    pub fn sc_window_get_title_owned(window: *const c_void) -> *mut i8;
    pub fn sc_window_get_window_layer(window: *const c_void) -> isize;
    pub fn sc_window_is_on_screen(window: *const c_void) -> bool;
    pub fn sc_window_get_owning_application(window: *const c_void) -> *const c_void;
    pub fn sc_window_is_active(window: *const c_void) -> bool;
}

// MARK: - SCRunningApplication
extern "C" {
    pub fn sc_running_application_retain(app: *const c_void) -> *const c_void;
    pub fn sc_running_application_release(app: *const c_void);
    pub fn sc_running_application_get_bundle_identifier(
        app: *const c_void,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;
    /// Get bundle identifier as owned string (caller must free with sc_free_string)
    pub fn sc_running_application_get_bundle_identifier_owned(app: *const c_void) -> *mut i8;
    pub fn sc_running_application_get_application_name(
        app: *const c_void,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;
    /// Get application name as owned string (caller must free with sc_free_string)
    pub fn sc_running_application_get_application_name_owned(app: *const c_void) -> *mut i8;
    pub fn sc_running_application_get_process_id(app: *const c_void) -> i32;
}

// MARK: - String memory management
extern "C" {
    /// Free a string allocated by Swift (strdup)
    pub fn sc_free_string(str: *mut i8);
}

// MARK: - SCStreamConfiguration
extern "C" {
    pub fn sc_stream_configuration_create() -> *const c_void;
    pub fn sc_stream_configuration_retain(config: *const c_void) -> *const c_void;
    pub fn sc_stream_configuration_release(config: *const c_void);

    pub fn sc_stream_configuration_set_width(config: *const c_void, width: isize);
    pub fn sc_stream_configuration_get_width(config: *const c_void) -> isize;

    pub fn sc_stream_configuration_set_height(config: *const c_void, height: isize);
    pub fn sc_stream_configuration_get_height(config: *const c_void) -> isize;

    pub fn sc_stream_configuration_set_shows_cursor(config: *const c_void, shows_cursor: bool);
    pub fn sc_stream_configuration_get_shows_cursor(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_scales_to_fit(config: *const c_void, scales_to_fit: bool);
    pub fn sc_stream_configuration_get_scales_to_fit(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_captures_audio(config: *const c_void, captures_audio: bool);
    pub fn sc_stream_configuration_get_captures_audio(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_sample_rate(config: *const c_void, sample_rate: isize);
    pub fn sc_stream_configuration_get_sample_rate(config: *const c_void) -> isize;

    pub fn sc_stream_configuration_set_channel_count(config: *const c_void, channel_count: isize);
    pub fn sc_stream_configuration_get_channel_count(config: *const c_void) -> isize;

    pub fn sc_stream_configuration_set_queue_depth(config: *const c_void, queue_depth: isize);
    pub fn sc_stream_configuration_get_queue_depth(config: *const c_void) -> isize;

    pub fn sc_stream_configuration_set_pixel_format(config: *const c_void, pixel_format: u32);
    pub fn sc_stream_configuration_get_pixel_format(config: *const c_void) -> u32;

    pub fn sc_stream_configuration_set_minimum_frame_interval(
        config: *const c_void,
        value: i64,
        timescale: i32,
        flags: u32,
        epoch: i64,
    );
    pub fn sc_stream_configuration_get_minimum_frame_interval(
        config: *const c_void,
        value: *mut i64,
        timescale: *mut i32,
        flags: *mut u32,
        epoch: *mut i64,
    );

    pub fn sc_stream_configuration_set_source_rect(
        config: *const c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn sc_stream_configuration_get_source_rect(
        config: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );

    pub fn sc_stream_configuration_set_destination_rect(
        config: *const c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn sc_stream_configuration_get_destination_rect(
        config: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );

    pub fn sc_stream_configuration_set_preserves_aspect_ratio(
        config: *const c_void,
        preserves_aspect_ratio: bool,
    );
    pub fn sc_stream_configuration_get_preserves_aspect_ratio(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_ignore_fraction_of_screen(
        config: *const c_void,
        ignore_fraction: f64,
    );
    pub fn sc_stream_configuration_get_ignore_fraction_of_screen(config: *const c_void) -> f64;

    pub fn sc_stream_configuration_set_ignores_shadows_single_window(
        config: *const c_void,
        ignores_shadows: bool,
    );
    pub fn sc_stream_configuration_get_ignores_shadows_single_window(config: *const c_void)
        -> bool;

    pub fn sc_stream_configuration_set_should_be_opaque(
        config: *const c_void,
        should_be_opaque: bool,
    );
    pub fn sc_stream_configuration_get_should_be_opaque(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_includes_child_windows(
        config: *const c_void,
        includes_child_windows: bool,
    );
    pub fn sc_stream_configuration_get_includes_child_windows(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
        config: *const c_void,
        setting: i32,
    );
    pub fn sc_stream_configuration_get_presenter_overlay_privacy_alert_setting(
        config: *const c_void,
    ) -> i32;

    pub fn sc_stream_configuration_set_background_color(
        config: *const c_void,
        r: f32,
        g: f32,
        b: f32,
    );
    pub fn sc_stream_configuration_set_color_space_name(config: *const c_void, name: *const i8);
    pub fn sc_stream_configuration_set_color_matrix(config: *const c_void, matrix: *const i8);
    pub fn sc_stream_configuration_get_color_matrix(
        config: *const c_void,
        buffer: *mut i8,
        buffer_size: usize,
    ) -> bool;

    pub fn sc_stream_configuration_set_ignore_global_clipboard(config: *const c_void, ignore: bool);
    pub fn sc_stream_configuration_get_ignore_global_clipboard(config: *const c_void) -> bool;

    // macOS 14.0+ - capture resolution type
    pub fn sc_stream_configuration_set_capture_resolution_type(config: *const c_void, value: i32);
    pub fn sc_stream_configuration_get_capture_resolution_type(config: *const c_void) -> i32;

    pub fn sc_stream_configuration_set_ignores_shadow_display_configuration(
        config: *const c_void,
        ignores_shadow: bool,
    );
    pub fn sc_stream_configuration_get_ignores_shadow_display_configuration(
        config: *const c_void,
    ) -> bool;

    pub fn sc_stream_configuration_set_preserve_aspect_ratio(config: *const c_void, preserve: bool);
    pub fn sc_stream_configuration_get_preserve_aspect_ratio(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_increase_resolution_for_retina_displays(
        config: *const c_void,
        increase: bool,
    );
    pub fn sc_stream_configuration_get_increase_resolution_for_retina_displays(
        config: *const c_void,
    ) -> bool;

    pub fn sc_stream_configuration_set_captures_shadows_only(
        config: *const c_void,
        captures_shadows_only: bool,
    );
    pub fn sc_stream_configuration_get_captures_shadows_only(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_captures_microphone(
        config: *const c_void,
        captures_microphone: bool,
    );
    pub fn sc_stream_configuration_get_captures_microphone(config: *const c_void) -> bool;

    pub fn sc_stream_configuration_set_excludes_current_process_audio(
        config: *const c_void,
        excludes: bool,
    );
    pub fn sc_stream_configuration_get_excludes_current_process_audio(
        config: *const c_void,
    ) -> bool;

    pub fn sc_stream_configuration_set_microphone_capture_device_id(
        config: *const c_void,
        device_id: *const i8,
    );
    pub fn sc_stream_configuration_get_microphone_capture_device_id(
        config: *const c_void,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;

    pub fn sc_stream_configuration_set_stream_name(config: *const c_void, name: *const i8);
    pub fn sc_stream_configuration_get_stream_name(
        config: *const c_void,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;

    pub fn sc_stream_configuration_set_capture_dynamic_range(config: *const c_void, value: i32);
    pub fn sc_stream_configuration_get_capture_dynamic_range(config: *const c_void) -> i32;
}

// MARK: - SCContentFilter
extern "C" {
    pub fn sc_content_filter_create_with_desktop_independent_window(
        window: *const c_void,
    ) -> *const c_void;
    pub fn sc_content_filter_create_with_display_excluding_windows(
        display: *const c_void,
        windows: *const *const c_void,
        windows_count: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_create_with_display_including_windows(
        display: *const c_void,
        windows: *const *const c_void,
        windows_count: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_create_with_display_including_applications_excepting_windows(
        display: *const c_void,
        apps: *const *const c_void,
        apps_count: isize,
        windows: *const *const c_void,
        windows_count: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_create_with_display_excluding_applications_excepting_windows(
        display: *const c_void,
        apps: *const *const c_void,
        apps_count: isize,
        windows: *const *const c_void,
        windows_count: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_retain(filter: *const c_void) -> *const c_void;
    pub fn sc_content_filter_release(filter: *const c_void);
    pub fn sc_content_filter_set_content_rect(
        filter: *const c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn sc_content_filter_get_content_rect(
        filter: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    /// Get content filter content rect (same as sc_content_filter_get_content_rect)
    pub fn sc_content_filter_get_content_rect_packed(
        filter: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
}

// MARK: - SCStream
extern "C" {
    pub fn sc_stream_create(
        filter: *const c_void,
        config: *const c_void,
        error_callback: extern "C" fn(*const c_void, i32, *const i8),
    ) -> *const c_void;
    pub fn sc_stream_add_stream_output(
        stream: *const c_void,
        output_type: i32,
        sample_buffer_callback: extern "C" fn(*const c_void, *const c_void, i32),
    ) -> bool;
    pub fn sc_stream_add_stream_output_with_queue(
        stream: *const c_void,
        output_type: i32,
        sample_buffer_callback: extern "C" fn(*const c_void, *const c_void, i32),
        dispatch_queue: *const c_void,
    ) -> bool;
    pub fn sc_stream_start_capture(
        stream: *const c_void,
        context: *mut c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
    );
    pub fn sc_stream_stop_capture(
        stream: *const c_void,
        context: *mut c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
    );
    pub fn sc_stream_update_configuration(
        stream: *const c_void,
        config: *const c_void,
        context: *mut c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
    );
    pub fn sc_stream_update_content_filter(
        stream: *const c_void,
        filter: *const c_void,
        context: *mut c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
    );
    pub fn sc_stream_add_recording_output(
        stream: *const c_void,
        recording_output: *const c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
        context: *mut c_void,
    );
    pub fn sc_stream_remove_recording_output(
        stream: *const c_void,
        recording_output: *const c_void,
        callback: extern "C" fn(*mut c_void, bool, *const i8),
        context: *mut c_void,
    );
    pub fn sc_stream_retain(stream: *const c_void) -> *const c_void;
    pub fn sc_stream_release(stream: *const c_void);
    
    // macOS 13.0+ - synchronizationClock
    pub fn sc_stream_get_synchronization_clock(stream: *const c_void) -> *const c_void;
}

// MARK: - Dispatch Queue
extern "C" {
    pub fn dispatch_queue_create(label: *const i8, qos: i32) -> *const c_void;
    pub fn dispatch_queue_release(queue: *const c_void);
    pub fn dispatch_queue_retain(queue: *const c_void) -> *const c_void;
}

// MARK: - IOSurface
extern "C" {
    pub fn cv_pixel_buffer_get_iosurface(pixel_buffer: *const c_void) -> *const c_void;
    pub fn cv_pixel_buffer_is_backed_by_iosurface(pixel_buffer: *const c_void) -> bool;
    pub fn iosurface_get_width(iosurface: *const c_void) -> isize;
    pub fn iosurface_get_height(iosurface: *const c_void) -> isize;
    pub fn iosurface_get_bytes_per_row(iosurface: *const c_void) -> isize;
    pub fn iosurface_get_pixel_format(iosurface: *const c_void) -> u32;
    pub fn iosurface_get_base_address(iosurface: *const c_void) -> *mut u8;
    pub fn iosurface_lock(iosurface: *const c_void, options: u32) -> i32;
    pub fn iosurface_unlock(iosurface: *const c_void, options: u32) -> i32;
    pub fn iosurface_is_in_use(iosurface: *const c_void) -> bool;
    pub fn iosurface_release(iosurface: *const c_void);
}

// MARK: - SCContentSharingPicker (macOS 14.0+)
extern "C" {
    pub fn sc_content_sharing_picker_configuration_create() -> *const c_void;
    pub fn sc_content_sharing_picker_configuration_set_allowed_picker_modes(
        config: *const c_void,
        modes: *const i32,
        count: usize,
    );
    pub fn sc_content_sharing_picker_configuration_set_allows_changing_selected_content(
        config: *const c_void,
        allows: bool,
    );
    pub fn sc_content_sharing_picker_configuration_get_allows_changing_selected_content(
        config: *const c_void,
    ) -> bool;
    pub fn sc_content_sharing_picker_configuration_set_excluded_bundle_ids(
        config: *const c_void,
        bundle_ids: *const *const i8,
        count: usize,
    );
    pub fn sc_content_sharing_picker_configuration_get_excluded_bundle_ids_count(
        config: *const c_void,
    ) -> usize;
    pub fn sc_content_sharing_picker_configuration_get_excluded_bundle_id_at(
        config: *const c_void,
        index: usize,
        buffer: *mut i8,
        buffer_size: usize,
    ) -> bool;
    pub fn sc_content_sharing_picker_configuration_set_excluded_window_ids(
        config: *const c_void,
        window_ids: *const u32,
        count: usize,
    );
    pub fn sc_content_sharing_picker_configuration_get_excluded_window_ids_count(
        config: *const c_void,
    ) -> usize;
    pub fn sc_content_sharing_picker_configuration_get_excluded_window_id_at(
        config: *const c_void,
        index: usize,
    ) -> u32;
    pub fn sc_content_sharing_picker_configuration_retain(config: *const c_void) -> *const c_void;
    pub fn sc_content_sharing_picker_configuration_release(config: *const c_void);
    
    // Picker maximum stream count
    pub fn sc_content_sharing_picker_set_maximum_stream_count(count: usize);
    pub fn sc_content_sharing_picker_get_maximum_stream_count() -> usize;
    
    pub fn sc_content_sharing_picker_show(
        config: *const c_void,
        callback: extern "C" fn(i32, *const c_void, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_content_sharing_picker_show_with_result(
        config: *const c_void,
        callback: extern "C" fn(i32, *const c_void, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_content_sharing_picker_show_for_stream(
        config: *const c_void,
        stream: *const c_void,
        callback: extern "C" fn(i32, *const c_void, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_content_sharing_picker_show_using_style(
        config: *const c_void,
        style: i32,
        callback: extern "C" fn(i32, *const c_void, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_content_sharing_picker_show_for_stream_using_style(
        config: *const c_void,
        stream: *const c_void,
        style: i32,
        callback: extern "C" fn(i32, *const c_void, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_picker_result_get_filter(result: *const c_void) -> *const c_void;
    pub fn sc_picker_result_get_content_rect(
        result: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    pub fn sc_picker_result_get_scale(result: *const c_void) -> f64;
    pub fn sc_picker_result_get_windows_count(result: *const c_void) -> usize;
    pub fn sc_picker_result_get_window_at(result: *const c_void, index: usize) -> *const c_void;
    pub fn sc_picker_result_get_displays_count(result: *const c_void) -> usize;
    pub fn sc_picker_result_get_display_at(result: *const c_void, index: usize) -> *const c_void;
    pub fn sc_picker_result_get_applications_count(result: *const c_void) -> usize;
    pub fn sc_picker_result_get_application_at(result: *const c_void, index: usize) -> *const c_void;
    pub fn sc_picker_result_release(result: *const c_void);
}

// MARK: - SCRecordingOutput (macOS 15.0+)
extern "C" {
    pub fn sc_recording_output_configuration_create() -> *const c_void;
    pub fn sc_recording_output_configuration_set_output_url(config: *const c_void, path: *const i8);
    pub fn sc_recording_output_configuration_set_video_codec(config: *const c_void, codec: i32);
    pub fn sc_recording_output_configuration_retain(config: *const c_void) -> *const c_void;
    pub fn sc_recording_output_configuration_release(config: *const c_void);
    pub fn sc_recording_output_create(config: *const c_void) -> *const c_void;
    pub fn sc_recording_output_retain(output: *const c_void) -> *const c_void;
    pub fn sc_recording_output_release(output: *const c_void);
}

// MARK: - SCScreenshotManager (macOS 14.0+)
extern "C" {
    pub fn sc_screenshot_manager_capture_image(
        content_filter: *const c_void,
        config: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_screenshot_manager_capture_sample_buffer(
        content_filter: *const c_void,
        config: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_screenshot_manager_capture_image_in_rect(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn cgimage_get_width(image: *const c_void) -> usize;
    pub fn cgimage_get_height(image: *const c_void) -> usize;
    pub fn cgimage_get_data(
        image: *const c_void,
        out_ptr: *mut *const u8,
        out_length: *mut usize,
    ) -> bool;
    pub fn cgimage_free_data(ptr: *mut u8);
    pub fn cgimage_release(image: *const c_void);
    pub fn cgimage_save_png(image: *const c_void, path: *const i8) -> bool;
}

// MARK: - SCScreenshotConfiguration (macOS 26.0+)
extern "C" {
    pub fn sc_screenshot_configuration_create() -> *const c_void;
    pub fn sc_screenshot_configuration_set_width(config: *const c_void, width: isize);
    pub fn sc_screenshot_configuration_set_height(config: *const c_void, height: isize);
    pub fn sc_screenshot_configuration_set_shows_cursor(config: *const c_void, shows_cursor: bool);
    pub fn sc_screenshot_configuration_set_source_rect(
        config: *const c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn sc_screenshot_configuration_set_destination_rect(
        config: *const c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn sc_screenshot_configuration_set_ignore_shadows(
        config: *const c_void,
        ignore_shadows: bool,
    );
    pub fn sc_screenshot_configuration_set_ignore_clipping(
        config: *const c_void,
        ignore_clipping: bool,
    );
    pub fn sc_screenshot_configuration_set_include_child_windows(
        config: *const c_void,
        include_child_windows: bool,
    );
    pub fn sc_screenshot_configuration_set_display_intent(
        config: *const c_void,
        display_intent: i32,
    );
    pub fn sc_screenshot_configuration_set_dynamic_range(
        config: *const c_void,
        dynamic_range: i32,
    );
    pub fn sc_screenshot_configuration_set_file_url(config: *const c_void, path: *const i8);
    pub fn sc_screenshot_configuration_release(config: *const c_void);
    
    // Content type support (macOS 26.0+)
    pub fn sc_screenshot_configuration_set_content_type(config: *const c_void, identifier: *const i8);
    pub fn sc_screenshot_configuration_get_content_type(
        config: *const c_void,
        buffer: *mut i8,
        buffer_size: usize,
    ) -> bool;
    pub fn sc_screenshot_configuration_get_supported_content_types_count() -> usize;
    pub fn sc_screenshot_configuration_get_supported_content_type_at(
        index: usize,
        buffer: *mut i8,
        buffer_size: usize,
    ) -> bool;
}

// MARK: - SCScreenshotOutput (macOS 26.0+)
extern "C" {
    pub fn sc_screenshot_output_get_sdr_image(output: *const c_void) -> *const c_void;
    pub fn sc_screenshot_output_get_hdr_image(output: *const c_void) -> *const c_void;
    pub fn sc_screenshot_output_get_file_url(
        output: *const c_void,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;
    pub fn sc_screenshot_output_release(output: *const c_void);

    pub fn sc_screenshot_manager_capture_screenshot(
        content_filter: *const c_void,
        config: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn sc_screenshot_manager_capture_screenshot_in_rect(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        config: *const c_void,
        callback: extern "C" fn(*const c_void, *const i8, *mut c_void),
        user_data: *mut c_void,
    );
}

// MARK: - SCStreamConfiguration additional properties
extern "C" {
    // macOS 15.0+ - showMouseClicks
    pub fn sc_stream_configuration_set_shows_mouse_clicks(config: *const c_void, value: bool);
    pub fn sc_stream_configuration_get_shows_mouse_clicks(config: *const c_void) -> bool;

    // macOS 14.0+ - ignoreShadowsDisplay
    pub fn sc_stream_configuration_set_ignores_shadows_display(config: *const c_void, value: bool);
    pub fn sc_stream_configuration_get_ignores_shadows_display(config: *const c_void) -> bool;

    // macOS 14.0+ - ignoreGlobalClipDisplay
    pub fn sc_stream_configuration_set_ignore_global_clip_display(config: *const c_void, value: bool);
    pub fn sc_stream_configuration_get_ignore_global_clip_display(config: *const c_void) -> bool;

    // macOS 14.0+ - ignoreGlobalClipSingleWindow
    pub fn sc_stream_configuration_set_ignore_global_clip_single_window(
        config: *const c_void,
        value: bool,
    );
    pub fn sc_stream_configuration_get_ignore_global_clip_single_window(
        config: *const c_void,
    ) -> bool;

    // macOS 15.0+ - preset-based configuration
    pub fn sc_stream_configuration_create_with_preset(preset: i32) -> *const c_void;
}

// MARK: - SCContentFilter additional properties
extern "C" {
    // macOS 14.0+ - style and pointPixelScale
    pub fn sc_content_filter_get_style(filter: *const c_void) -> i32;
    pub fn sc_content_filter_get_point_pixel_scale(filter: *const c_void) -> f32;
    pub fn sc_content_filter_get_stream_type(filter: *const c_void) -> i32;

    // macOS 14.2+ - includeMenuBar
    pub fn sc_content_filter_set_include_menu_bar(filter: *const c_void, include: bool);
    pub fn sc_content_filter_get_include_menu_bar(filter: *const c_void) -> bool;

    // macOS 15.2+ - included content arrays
    pub fn sc_content_filter_get_included_displays_count(filter: *const c_void) -> isize;
    pub fn sc_content_filter_get_included_display_at(
        filter: *const c_void,
        index: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_get_included_windows_count(filter: *const c_void) -> isize;
    pub fn sc_content_filter_get_included_window_at(
        filter: *const c_void,
        index: isize,
    ) -> *const c_void;
    pub fn sc_content_filter_get_included_applications_count(filter: *const c_void) -> isize;
    pub fn sc_content_filter_get_included_application_at(
        filter: *const c_void,
        index: isize,
    ) -> *const c_void;
}

// MARK: - SCShareableContentInfo (macOS 14.0+)
extern "C" {
    pub fn sc_shareable_content_info_for_filter(filter: *const c_void) -> *const c_void;
    pub fn sc_shareable_content_info_get_style(info: *const c_void) -> i32;
    pub fn sc_shareable_content_info_get_point_pixel_scale(info: *const c_void) -> f32;
    pub fn sc_shareable_content_info_get_content_rect(
        info: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    /// Get shareable content info rect (same as sc_shareable_content_info_get_content_rect)
    pub fn sc_shareable_content_info_get_content_rect_packed(
        info: *const c_void,
        x: *mut f64,
        y: *mut f64,
        width: *mut f64,
        height: *mut f64,
    );
    pub fn sc_shareable_content_info_retain(info: *const c_void) -> *const c_void;
    pub fn sc_shareable_content_info_release(info: *const c_void);
}

// MARK: - SCRecordingOutput additional (macOS 15.0+)
extern "C" {
    pub fn sc_recording_output_configuration_set_output_file_type(
        config: *const c_void,
        file_type: i32,
    );
    pub fn sc_recording_output_configuration_get_output_file_type(config: *const c_void) -> i32;
    pub fn sc_recording_output_configuration_get_video_codec(config: *const c_void) -> i32;
    pub fn sc_recording_output_configuration_get_available_video_codecs_count(
        config: *const c_void,
    ) -> isize;
    pub fn sc_recording_output_configuration_get_available_video_codec_at(
        config: *const c_void,
        index: isize,
    ) -> i32;
    pub fn sc_recording_output_configuration_get_available_output_file_types_count(
        config: *const c_void,
    ) -> isize;
    pub fn sc_recording_output_configuration_get_available_output_file_type_at(
        config: *const c_void,
        index: isize,
    ) -> i32;
    pub fn sc_recording_output_create_with_delegate(
        config: *const c_void,
        started_callback: Option<extern "C" fn(*mut c_void)>,
        failed_callback: Option<extern "C" fn(*mut c_void, i32, *const i8)>,
        finished_callback: Option<extern "C" fn(*mut c_void)>,
        context: *mut c_void,
    ) -> *const c_void;
    pub fn sc_recording_output_get_recorded_duration(
        output: *const c_void,
        value: *mut i64,
        timescale: *mut i32,
    );
    pub fn sc_recording_output_get_recorded_file_size(output: *const c_void) -> i64;
}

// MARK: - Audio Input Devices (AVFoundation)
extern "C" {
    /// Get the count of available audio input devices
    pub fn sc_audio_get_input_device_count() -> isize;

    /// Get audio input device ID at index into buffer
    pub fn sc_audio_get_input_device_id(
        index: isize,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;

    /// Get audio input device name at index into buffer
    pub fn sc_audio_get_input_device_name(
        index: isize,
        buffer: *mut i8,
        buffer_size: isize,
    ) -> bool;

    /// Check if the device at index is the default audio input device
    pub fn sc_audio_is_default_input_device(index: isize) -> bool;

    /// Get the default audio input device ID into buffer
    pub fn sc_audio_get_default_input_device_id(buffer: *mut i8, buffer_size: isize) -> bool;

    /// Get the default audio input device name into buffer
    pub fn sc_audio_get_default_input_device_name(buffer: *mut i8, buffer_size: isize) -> bool;
}
