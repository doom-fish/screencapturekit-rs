# screencapturekit-rs member coverage (vs macOS 26.5 SDK, with the 27.0 delta)

Checked for `screencapturekit` 11.0.0 on 2026-09-24. The previous version of
this file repeated the v3.1.1 declaration table (and cited an adapter type,
`SCPickerObserverAdapter`, that does not exist). Declarations are now counted
only in [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md); this file checks members.

**Methodology.** For every class and protocol in the macOS 26.5
ScreenCaptureKit headers, each property, instance method, class method and
initializer that is available on macOS and not deprecated was looked up in the
Swift bridge (`swift-bridge/Sources/`) under both its Swift name and its
Objective-C selector, then traced to the public Rust API that calls that
bridge export. BRIDGED means both exist. This is a static check: it does not
show that each member behaves correctly, and members behind a `macos_*` Cargo
feature need that feature.

MACOS_MEMBERS: 136
BRIDGED: 136
MISSING: 0
BRIDGED_ONLY: 0

## macOS 26.5

| Class / protocol | Members | Bridged | Rust surface |
| --- | --- | --- | --- |
| SCStream | 10 | 10 | `SCStream::{new, new_with_delegate, add_output_handler, add_output_handler_with_queue, remove_output_handler, start_capture, stop_capture, update_content_filter, update_configuration, add_recording_output, remove_recording_output, synchronization_clock}` |
| SCStreamConfiguration | 32 | 32 | `SCStreamConfiguration` getters, `set_*` and `with_*` builders for all 31 properties, plus `from_preset` |
| SCContentFilter | 12 | 12 | `SCContentFilterBuilder` for the five initializers and `includeMenuBar`; `SCContentFilter::{style, point_pixel_scale, content_rect, include_menu_bar, included_displays, included_windows, included_applications}` |
| SCStreamOutput | 1 | 1 | `SCStreamOutputTrait::did_output_sample_buffer` |
| SCStreamDelegate | 5 | 5 | `SCStreamDelegateTrait` (`did_stop_with_error`, `stream_did_become_active`, `stream_did_become_inactive`, `output_video_effect_did_start_for_stream`, `output_video_effect_did_stop_for_stream`) |
| SCShareableContent | 9 | 9 | `SCShareableContent::{get, create, current_process, displays, windows, applications}`, `SCShareableContentOptions::{get, below_window, above_window}`, `SCShareableContentInfo::for_filter` |
| SCShareableContentInfo | 3 | 3 | `SCShareableContentInfo::{style, point_pixel_scale, content_rect}` |
| SCDisplay | 4 | 4 | `SCDisplay::{display_id, width, height, frame}` |
| SCWindow | 7 | 7 | `SCWindow::{window_id, frame, title, window_layer, owning_application, is_on_screen, is_active}` |
| SCRunningApplication | 3 | 3 | `SCRunningApplication::{bundle_identifier, application_name, process_id}` |
| SCContentSharingPicker | 11 | 11 | `SCContentSharingPicker::{default_configuration, set_default_configuration, maximum_stream_count, set_maximum_stream_count, is_active, set_active, add_observer, set_configuration_for_stream, present, present_using_style, present_for_stream, present_for_stream_using_style}`; `removeObserver:` runs when an `SCPickerSubscription` is dropped or detached |
| SCContentSharingPickerConfiguration | 4 | 4 | `SCContentSharingPickerConfiguration` getters and setters for all four properties |
| SCContentSharingPickerObserver | 3 | 3 | `SCPickerEvent` through `add_observer`, and the one-shot `show*` helpers |
| SCRecordingOutput | 3 | 3 | `SCRecordingOutput::{new, new_with_delegate, recorded_duration, recorded_file_size}` |
| SCRecordingOutputConfiguration | 5 | 5 | `SCRecordingOutputConfiguration::{with_output_url, output_url, with_video_codec, video_codec, with_output_file_type, output_file_type, available_video_codecs, available_output_file_types}` |
| SCRecordingOutputDelegate | 3 | 3 | `SCRecordingOutputDelegate`, `RecordingCallbacks::{on_start, on_fail, on_finish}` |
| SCScreenshotManager | 5 | 5 | `SCScreenshotManager::{capture_sample_buffer, capture_image, capture_image_in_rect, capture_screenshot, capture_screenshot_in_rect}` |
| SCScreenshotConfiguration | 13 | 13 | `SCScreenshotConfiguration` getters and `with_*` builders for all twelve properties, plus `supported_content_types` |
| SCScreenshotOutput | 3 | 3 | `SCScreenshotOutput::{sdr_image, hdr_image, file_path}` |

Deprecated members that are not scored but still exposed: `SCStreamType` and
`SCContentFilter.streamType` (`SCContentFilter::stream_type`).

## macOS 27.0 additions (not wrapped)

The macOS 27.0 SDK adds the following macOS-available API. None of it is
wrapped in 11.0; it is out of scope for this release.

| Header | Addition |
| --- | --- |
| SCClipBufferingOutput.h | `SCClipBufferingOutput` (`initWithDelegate:`, `exportClipToURL:duration:completionHandler:`) and `SCClipBufferingOutputDelegate` (`clipBufferingOutputDidStartBuffering:`, `clipBufferingOutputDidStopBuffering:`, `clipBufferingOutput:didFailWithError:`) |
| SCStream.h | `SCStream addClipBufferingOutput:error:` / `removeClipBufferingOutput:error:`, `SCStream.capturing`, `SCContentFilter.microphoneEnabled`, `SCStreamFrameInfoVideoOrientation` |
| SCRecordingEditor.h | `SCRecordingEditor` (`initWithURL:`, `delegate`, `presentFromWindow:completionHandler:`) and `SCRecordingEditorDelegate` (`recordingEditorDidDismiss:`, `recordingEditor:didFailWithError:`) |
| SCRecordingOutput.h | `SCRecordingOutputConfiguration.mixesAudioWithMicrophone` |
| SCContentSharingPicker.h | `SCContentSharingPicker.available` |

The two new error codes, `SCStreamErrorInsufficientStorage` (-3822) and
`SCStreamErrorNotSupported` (-3823), are wrapped as
`SCStreamErrorCode::InsufficientStorage` and `SCStreamErrorCode::NotSupported`.
Additions that are unavailable on macOS (`SCVideoEffectOutput` and the
video-effect `SCStream` methods, camera and microphone picker controls,
`SCRecordingEditorMode`, `presentFromWindowScene:`, and
`SCStreamErrorMissingBackgroundMode`) are not listed.
