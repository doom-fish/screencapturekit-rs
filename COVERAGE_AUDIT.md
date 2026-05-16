# screencapturekit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 42
VERIFIED: 41
GAPS: 0
EXEMPT: 1
COVERAGE_PCT: 100.0%

Scope: top-level public ScreenCaptureKit declarations from the SDK headers (`@interface`, `@protocol`, `typedef NS_ENUM/NS_OPTIONS/NS_ERROR_ENUM`, `NS_TYPED_ENUM`, and exported constants), excluding non-macOS declarations and listing Apple-deprecated symbols as exempt. Crate coverage was checked against the public Rust API with feature-gated modules enabled via `--all-features`, plus the Swift bridge thunks under `swift-bridge/Sources/ScreenCaptureKitBridge`.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| SCContentSharingPickerMode | NS_OPTIONS | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerMode` |
| SCContentSharingPickerObserver | protocol | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPicker::{show, show_for_stream, show_filter, show_using_style, show_for_stream_using_style}` |
| SCContentSharingPickerConfiguration | class | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerConfiguration` |
| SCContentSharingPicker | class | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPicker` |
| SCStreamErrorCode | NS_ERROR_ENUM | SCError.h | `error::SCStreamErrorCode` |
| SCStreamErrorDomain | extern NSString *const | SCError.h | `error::SC_STREAM_ERROR_DOMAIN` |
| SCRecordingOutputConfiguration | class | SCRecordingOutput.h | `recording_output::SCRecordingOutputConfiguration` |
| SCRecordingOutputDelegate | protocol | SCRecordingOutput.h | `recording_output::SCRecordingOutputDelegate` |
| SCRecordingOutput | class | SCRecordingOutput.h | `recording_output::SCRecordingOutput` |
| SCScreenshotDisplayIntent | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDisplayIntent` (`macos_26_0`) |
| SCScreenshotDynamicRange | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDynamicRange` (`macos_26_0`) |
| SCScreenshotConfiguration | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotConfiguration` (`macos_26_0`) |
| SCScreenshotOutput | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotOutput` (`macos_26_0`) |
| SCScreenshotManager | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotManager` (`macos_14_0`) |
| SCShareableContentStyle | NS_ENUM | SCShareableContent.h | `stream::content_filter::SCShareableContentStyle` (`macos_14_0`) |
| SCRunningApplication | class | SCShareableContent.h | `shareable_content::SCRunningApplication` |
| SCWindow | class | SCShareableContent.h | `shareable_content::SCWindow` |
| SCDisplay | class | SCShareableContent.h | `shareable_content::SCDisplay` |
| SCShareableContentInfo | class | SCShareableContent.h | `shareable_content::SCShareableContentInfo` (`macos_14_0`) |
| SCShareableContent | class | SCShareableContent.h | `shareable_content::SCShareableContent` |
| SCStreamOutputType | NS_ENUM | SCStream.h | `stream::output_type::SCStreamOutputType` |
| SCFrameStatus | NS_ENUM | SCStream.h | `cm::SCFrameStatus` |
| SCPresenterOverlayAlertSetting | NS_ENUM | SCStream.h | `stream::configuration::SCPresenterOverlayAlertSetting` (`macos_14_0`) |
| SCCaptureResolutionType | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureResolutionType` (`macos_14_0`) |
| SCCaptureDynamicRange | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureDynamicRange` (`macos_15_0`) |
| SCContentFilter | class | SCStream.h | `stream::content_filter::SCContentFilter` |
| SCStreamConfiguration | class | SCStream.h | `stream::configuration::SCStreamConfiguration` |
| SCStreamConfigurationPreset | NS_ENUM | SCStream.h | `stream::configuration::SCStreamConfigurationPreset` (`macos_15_0`) |
| SCStreamDelegate | protocol | SCStream.h | `stream::SCStreamDelegate` / `delegate_trait::SCStreamDelegateTrait` |
| SCStreamFrameInfo | NS_TYPED_ENUM | SCStream.h | `cm::CMSampleBufferSCExt` + `cm::FrameInfo` |
| SCStreamFrameInfoStatus | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{frame_status, frame_info}` |
| SCStreamFrameInfoDisplayTime | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{display_time, frame_info}` |
| SCStreamFrameInfoScaleFactor | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{scale_factor, frame_info}` |
| SCStreamFrameInfoContentScale | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{content_scale, frame_info}` |
| SCStreamFrameInfoContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{content_rect, frame_info}` |
| SCStreamFrameInfoDirtyRects | extern const | SCStream.h | `cm::CMSampleBufferSCExt::dirty_rects` |
| SCStreamFrameInfoScreenRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{screen_rect, frame_info}` |
| SCStreamFrameInfoBoundingRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{bounding_rect, frame_info}` |
| SCStreamFrameInfoPresenterOverlayContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{presenter_overlay_content_rect, frame_info}` (`macos_14_2`) |
| SCStreamOutput | protocol | SCStream.h | `stream::SCStreamOutput` / `output_trait::SCStreamOutputTrait` |
| SCStream | class | SCStream.h | `stream::SCStream` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| SCStreamType | NS_ENUM | SCStream.h | Apple deprecated it in favor of `SCShareableContentStyle`; per the audit instructions, deprecated macOS APIs are excluded from the score even if the crate still exposes `stream::content_filter::SCStreamType`. | `API_DEPRECATED("Use SCShareableContentStyle instead", macos(14.0, 15.0))` |
