#ifndef SCREENCAPTUREKIT_BRIDGE_H
#define SCREENCAPTUREKIT_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>
#include <CoreGraphics/CoreGraphics.h>

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
typedef void (*SCStreamOutputCallback)(const void* context, const void* stream, int32_t type, const void* sampleBuffer);
typedef void (*SCStreamErrorCallback)(const void* stream, const void* error);
typedef void (*SCStreamCompletion)(const void* error);

const void* sc_stream_create(const void* filter, const void* config, const void* context, SCStreamErrorCallback errorCallback);
void sc_stream_release(const void* stream);
bool sc_stream_add_output(const void* stream, int32_t type, const void* context, SCStreamOutputCallback callback);
void sc_stream_start_capture(const void* stream, SCStreamCompletion completion);
void sc_stream_stop_capture(const void* stream, SCStreamCompletion completion);
void sc_stream_update_configuration(const void* stream, const void* config, SCStreamCompletion completion);

// Screenshot
typedef void (*SCScreenshotCompletion)(const void* image, const void* error);
void sc_screenshot_capture(const void* filter, const void* config, SCScreenshotCompletion completion);
void sc_cgimage_release(const void* image);

#ifdef __cplusplus
}
#endif

#endif // SCREENCAPTUREKIT_BRIDGE_H
