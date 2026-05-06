#ifndef SCREENCAPTUREKIT_BRIDGE_H
#define SCREENCAPTUREKIT_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>
#include <CoreGraphics/CoreGraphics.h>

/**
 * \file ScreenCaptureKitBridge.h
 *
 * \warning This header is **not** the canonical C ABI surface of the bridge.
 *          It documents a small subset of the ~500 @_cdecl symbols exposed
 *          by the Swift bridge and may lag behind the Swift sources. The
 *          authoritative signatures live in:
 *            - swift-bridge/Sources/**/*.swift   (definitions)
 *            - src/ffi/mod.rs, src/cm/ffi.rs,
 *              src/cv/ffi.rs                     (Rust FFI imports)
 *
 *          External (non-Rust) consumers should generate bindings from the
 *          Swift sources directly (e.g. via swift-bridge or a custom tool)
 *          rather than relying on this header. Mismatches between this
 *          header and the Swift definitions will be fixed on a best-effort
 *          basis but are not part of the crate's stability contract.
 */

#ifdef __cplusplus
extern "C" {
#endif

// Error handling
const char* sc_get_error_description(const void* error);
void sc_free_string(const char* str);

// Shareable Content
typedef void (*SCShareableContentCompletion)(const void* content, const void* error);
void sc_get_shareable_content(SCShareableContentCompletion completion);
void sc_get_shareable_content_with_options(bool excludeDesktop, bool onScreenOnly, SCShareableContentCompletion completion);
void sc_shareable_content_release(const void* content);
void sc_shareable_content_get_displays(const void* content, const void*** outArray, int* outCount);
void sc_shareable_content_get_windows(const void* content, const void*** outArray, int* outCount);
void sc_shareable_content_get_applications(const void* content, const void*** outArray, int* outCount);
void sc_free_array(const void** array);

// Display
void sc_display_release(const void* display);
int sc_display_get_width(const void* display);
int sc_display_get_height(const void* display);
uint32_t sc_display_get_display_id(const void* display);

// Window
void sc_window_release(const void* window);
uint32_t sc_window_get_window_id(const void* window);
const char* sc_window_get_title(const void* window);
void sc_window_get_frame(const void* window, CGRect* outFrame);
bool sc_window_is_on_screen(const void* window);

// Running Application
void sc_running_application_release(const void* app);
const char* sc_running_application_get_bundle_identifier(const void* app);
const char* sc_running_application_get_application_name(const void* app);
int32_t sc_running_application_get_process_id(const void* app);

// Content Filter
const void* sc_content_filter_create_with_display_excluding_windows(const void* display, const void** windows, int windowCount);
const void* sc_content_filter_create_with_display_including_windows(const void* display, const void** windows, int windowCount);
const void* sc_content_filter_create_with_desktop_independent_window(const void* window);
void sc_content_filter_release(const void* filter);

// Stream Configuration
const void* sc_stream_configuration_create(void);
void sc_stream_configuration_release(const void* config);
void sc_stream_configuration_set_width(const void* config, int width);
void sc_stream_configuration_set_height(const void* config, int height);
void sc_stream_configuration_set_captures_audio(const void* config, bool capturesAudio);
void sc_stream_configuration_set_sample_rate(const void* config, int sampleRate);
void sc_stream_configuration_set_channel_count(const void* config, int channelCount);
void sc_stream_configuration_set_pixel_format(const void* config, uint32_t pixelFormat);
void sc_stream_configuration_set_shows_cursor(const void* config, bool showsCursor);
void sc_stream_configuration_set_minimum_frame_interval(const void* config, double seconds);

// Stream

/// Error callback fired when SCStream encounters a fatal error.
/// \param context  Per-stream context pointer that was passed to sc_stream_create.
/// \param errorCode Apple's SCStreamError code, or 0 if no specific code is available.
/// \param errorMsg  Null-terminated UTF-8 error description (valid only for the duration of the call).
typedef void (*SCStreamErrorCallback)(void* context, int32_t errorCode, const char* errorMsg);

/// Sample-buffer callback fired for each captured frame / audio chunk / microphone sample.
/// The CMSampleBuffer is passed retained (via Unmanaged.passRetained on the Swift side);
/// the receiver is responsible for releasing it via cm_sample_buffer_release.
/// \param context      Per-stream context pointer that was passed to sc_stream_create.
/// \param sampleBuffer CMSampleBuffer pointer — passed retained, must be released by the receiver.
/// \param outputType   0 = screen, 1 = audio, 2 = microphone.
typedef void (*SCStreamOutputCallback)(void* context, const void* sampleBuffer, int32_t outputType);

/// Generic completion callback for asynchronous SCStream operations
/// (start, stop, update). The error pointer is non-null on failure only.
typedef void (*SCStreamCompletion)(const void* error);

const void* sc_stream_create(const void* filter, const void* config, void* context, SCStreamErrorCallback errorCallback, SCStreamOutputCallback sampleCallback);
void sc_stream_release(const void* stream);
bool sc_stream_add_stream_output(const void* stream, int32_t type);
bool sc_stream_add_stream_output_with_queue(const void* stream, int32_t type, const void* dispatchQueue);
bool sc_stream_remove_stream_output(const void* stream, int32_t type);
void sc_stream_start_capture(const void* stream, SCStreamCompletion completion);
void sc_stream_stop_capture(const void* stream, SCStreamCompletion completion);
void sc_stream_update_configuration(const void* stream, const void* config, SCStreamCompletion completion);

// Screenshot
typedef void (*SCScreenshotCompletion)(const void* image, const void* error);
void sc_screenshot_capture(const void* filter, const void* config, SCScreenshotCompletion completion);
void sc_cgimage_release(const void* image);

// CMSampleBuffer (release function for the buffer Swift passes retained)
void cm_sample_buffer_release(void* sampleBuffer);

#ifdef __cplusplus
}
#endif

#endif // SCREENCAPTUREKIT_BRIDGE_H
