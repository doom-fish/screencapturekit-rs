# screencapturekit-rs coverage audit v2 (vs MacOSX26.2.sdk)

> Snapshot from the `screencapturekit` v3.1.1 audit pass — historical, not re-verified against 6.x.

SDK_PUBLIC_SYMBOLS: 42
VERIFIED: 41
GAPS: 0
EXEMPT: 1
COVERAGE_PCT: 100.0%

## Methodology

Enumerated all public-facing ScreenCaptureKit symbols from the macOS 26.2 SDK headers (7 headers total, ~1317 lines) via grep-extraction of `@interface`, `@protocol`, `typedef NS_ENUM`, `typedef NS_OPTIONS`, `typedef NS_ERROR_ENUM`, `typedef NS_TYPED_ENUM`, and `extern const` declarations. Cross-referenced each symbol against the crate's Rust source tree (44 Rust modules across src/), Swift bridge thunks, and public re-exports. Validated that protocol types are wrapped via trait adapters (e.g., `SCStreamDelegate` → `SCStreamDelegateTrait`), and that NS_TYPED_ENUM constants are accessible via extension traits (e.g., `SCStreamFrameInfo*` → `CMSampleBufferSCExt`). Re-verified the v1 EXEMPT entry (`SCStreamType`) against the SDK headers: confirmed API_DEPRECATED marker.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| SCContentSharingPickerMode | NS_OPTIONS | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerMode` |
| SCContentSharingPickerObserver | @protocol | SCContentSharingPicker.h | `content_sharing_picker::SCPickerObserverAdapter` (trait adapter) |
| SCContentSharingPickerConfiguration | @interface | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerConfiguration` |
| SCContentSharingPicker | @interface | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPicker` |
| SCStreamErrorDomain | extern const | SCError.h | `error::SC_STREAM_ERROR_DOMAIN` (re-export) |
| SCStreamErrorCode | NS_ERROR_ENUM | SCError.h | `error::SCStreamErrorCode` |
| SCRecordingOutputConfiguration | @interface | SCRecordingOutput.h | `recording_output::SCRecordingOutputConfiguration` |
| SCRecordingOutputDelegate | @protocol | SCRecordingOutput.h | `recording_output::SCRecordingOutputDelegate` (trait) |
| SCRecordingOutput | @interface | SCRecordingOutput.h | `recording_output::SCRecordingOutput` |
| SCScreenshotDisplayIntent | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDisplayIntent` |
| SCScreenshotDynamicRange | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDynamicRange` (macos_26_0) |
| SCScreenshotConfiguration | @interface | SCScreenshotManager.h | `screenshot_manager::SCScreenshotConfiguration` |
| SCScreenshotOutput | @interface | SCScreenshotManager.h | `screenshot_manager::SCScreenshotOutput` |
| SCScreenshotManager | @interface | SCScreenshotManager.h | `screenshot_manager::SCScreenshotManager` |
| SCShareableContentStyle | NS_ENUM | SCShareableContent.h | `stream::content_filter::SCShareableContentStyle` |
| SCRunningApplication | @interface | SCShareableContent.h | `shareable_content::SCRunningApplication` |
| SCWindow | @interface | SCShareableContent.h | `shareable_content::SCWindow` |
| SCDisplay | @interface | SCShareableContent.h | `shareable_content::SCDisplay` |
| SCShareableContentInfo | @interface | SCShareableContent.h | `shareable_content::SCShareableContentInfo` |
| SCShareableContent | @interface | SCShareableContent.h | `shareable_content::SCShareableContent` |
| SCStreamOutputType | NS_ENUM | SCStream.h | `stream::output_type::SCStreamOutputType` |
| SCFrameStatus | NS_ENUM | SCStream.h | `cm::SCFrameStatus` (frame_status module) |
| SCPresenterOverlayAlertSetting | NS_ENUM | SCStream.h | `stream::configuration::SCPresenterOverlayAlertSetting` |
| SCCaptureResolutionType | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureResolutionType` |
| SCCaptureDynamicRange | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureDynamicRange` |
| SCContentFilter | @interface | SCStream.h | `stream::content_filter::SCContentFilter` |
| SCStreamConfiguration | @interface | SCStream.h | `stream::configuration::SCStreamConfiguration` |
| SCStreamConfigurationPreset | NS_ENUM | SCStream.h | `stream::configuration::SCStreamConfigurationPreset` |
| SCStreamDelegate | @protocol | SCStream.h | `stream::delegate_trait::SCStreamDelegateTrait` (trait adapter) |
| SCStreamFrameInfo | NS_TYPED_ENUM | SCStream.h | `cm::FrameInfo` (via `CMSampleBufferSCExt`) |
| SCStreamFrameInfoStatus | extern const | SCStream.h | `cm::CMSampleBufferSCExt::frame_status()` |
| SCStreamFrameInfoDisplayTime | extern const | SCStream.h | `cm::CMSampleBufferSCExt::display_time()` |
| SCStreamFrameInfoScaleFactor | extern const | SCStream.h | `cm::CMSampleBufferSCExt::scale_factor()` |
| SCStreamFrameInfoContentScale | extern const | SCStream.h | `cm::CMSampleBufferSCExt::content_scale()` |
| SCStreamFrameInfoContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::content_rect()` |
| SCStreamFrameInfoDirtyRects | extern const | SCStream.h | `cm::CMSampleBufferSCExt::dirty_rects()` |
| SCStreamFrameInfoScreenRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::screen_rect()` |
| SCStreamFrameInfoBoundingRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::bounding_rect()` |
| SCStreamFrameInfoPresenterOverlayContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::presenter_overlay_content_rect()` |
| SCStreamOutput | @protocol | SCStream.h | `stream::output_trait::SCStreamOutputTrait` (trait adapter) |
| SCStream | @interface | SCStream.h | `stream::SCStream` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| SCStreamType | NS_ENUM | SCStream.h | Apple deprecated in favor of `SCShareableContentStyle` in macOS 15.0; crate still exposes via `stream::content_filter::SCStreamType` for backward compatibility. | `API_DEPRECATED("Use SCShareableContentStyle instead", macos(14.0, 15.0))` |
