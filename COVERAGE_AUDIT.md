# screencapturekit-rs coverage audit (vs macOS 27.0 and 26.5 SDKs)

Checked for `screencapturekit` 11.0.0 on 2026-09-24 against the
ScreenCaptureKit headers of the macOS 27.0 SDK (Command Line Tools) and the
macOS 26.5 SDK (Xcode 26.5, the default build SDK). The previous version of
this file was a v3.1.1 snapshot against `MacOSX26.2.sdk`, which is no longer
installed.

**What the numbers measure.** A symbol is a top-level `@interface`,
`@protocol`, `typedef NS_ENUM` / `NS_OPTIONS` / `NS_ERROR_ENUM`,
`NS_TYPED_ENUM` or exported constant that is available on macOS. VERIFIED
means the crate has a public Rust item for it; every path in the table below
was compile-checked against 11.0.0 with `--all-features`. COVERAGE_PCT is
VERIFIED divided by the non-exempt symbols. The count says nothing about
individual properties and methods, which are checked separately in
[`COVERAGE_AUDIT_V2.md`](COVERAGE_AUDIT_V2.md). Deprecated symbols are exempt
and not scored. Most types need a `macos_*` Cargo feature; see the feature
table in the README.

## macOS 27.0 SDK

SDK_PUBLIC_SYMBOLS: 47
VERIFIED: 41
GAPS: 5
EXEMPT: 1
COVERAGE_PCT: 89.1%

The five gaps are macOS 27 additions that 11.0 does not wrap; they are out of
scope for this release. `SCVideoEffectOutput` (iOS only) and
`SCRecordingEditorMode` (tvOS only) are not available on macOS and are not
counted.

## macOS 26.5 SDK

SDK_PUBLIC_SYMBOLS: 42
VERIFIED: 41
GAPS: 0
EXEMPT: 1
COVERAGE_PCT: 100.0%

The 26.5 headers declare the same 42 macOS symbols that the v3.1.1 audit
listed for `MacOSX26.2.sdk`.

## 🟢 VERIFIED

| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| SCContentSharingPickerMode | NS_OPTIONS | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerMode` |
| SCContentSharingPickerObserver | protocol | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPicker::add_observer` (repeating `SCPickerEvent`s through an `SCPickerSubscription`) and the one-shot `show*` helpers |
| SCContentSharingPickerConfiguration | class | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPickerConfiguration` |
| SCContentSharingPicker | class | SCContentSharingPicker.h | `content_sharing_picker::SCContentSharingPicker` |
| SCStreamErrorCode | NS_ERROR_ENUM | SCError.h | `error::SCStreamErrorCode` (every macOS code, -3801 to -3823) |
| SCStreamErrorDomain | extern const | SCError.h | `error::SC_STREAM_ERROR_DOMAIN` |
| SCRecordingOutputConfiguration | class | SCRecordingOutput.h | `recording_output::SCRecordingOutputConfiguration` |
| SCRecordingOutputDelegate | protocol | SCRecordingOutput.h | `recording_output::SCRecordingOutputDelegate` |
| SCRecordingOutput | class | SCRecordingOutput.h | `recording_output::SCRecordingOutput` |
| SCScreenshotDisplayIntent | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDisplayIntent` |
| SCScreenshotDynamicRange | NS_ENUM | SCScreenshotManager.h | `screenshot_manager::SCScreenshotDynamicRange` |
| SCScreenshotConfiguration | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotConfiguration` |
| SCScreenshotOutput | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotOutput` |
| SCScreenshotManager | class | SCScreenshotManager.h | `screenshot_manager::SCScreenshotManager` |
| SCShareableContentStyle | NS_ENUM | SCShareableContent.h | `stream::content_filter::SCShareableContentStyle` |
| SCRunningApplication | class | SCShareableContent.h | `shareable_content::SCRunningApplication` |
| SCWindow | class | SCShareableContent.h | `shareable_content::SCWindow` |
| SCDisplay | class | SCShareableContent.h | `shareable_content::SCDisplay` |
| SCShareableContentInfo | class | SCShareableContent.h | `shareable_content::SCShareableContentInfo` |
| SCShareableContent | class | SCShareableContent.h | `shareable_content::SCShareableContent` |
| SCStreamOutputType | NS_ENUM | SCStream.h | `stream::output_type::SCStreamOutputType` |
| SCFrameStatus | NS_ENUM | SCStream.h | `cm::SCFrameStatus` |
| SCPresenterOverlayAlertSetting | NS_ENUM | SCStream.h | `stream::configuration::SCPresenterOverlayAlertSetting` |
| SCCaptureResolutionType | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureResolutionType` |
| SCCaptureDynamicRange | NS_ENUM | SCStream.h | `stream::configuration::SCCaptureDynamicRange` |
| SCContentFilter | class | SCStream.h | `stream::content_filter::SCContentFilter` |
| SCStreamConfiguration | class | SCStream.h | `stream::configuration::SCStreamConfiguration` |
| SCStreamConfigurationPreset | NS_ENUM | SCStream.h | `stream::configuration::SCStreamConfigurationPreset` |
| SCStreamDelegate | protocol | SCStream.h | `stream::delegate_trait::SCStreamDelegateTrait` (alias `stream::SCStreamDelegate`) |
| SCStreamFrameInfo | NS_TYPED_ENUM | SCStream.h | `cm::CMSampleBufferSCExt` and `cm::FrameInfo` |
| SCStreamFrameInfoStatus | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{frame_status, frame_info}` |
| SCStreamFrameInfoDisplayTime | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{display_time, frame_info}` |
| SCStreamFrameInfoScaleFactor | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{scale_factor, frame_info}` |
| SCStreamFrameInfoContentScale | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{content_scale, frame_info}` |
| SCStreamFrameInfoContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{content_rect, frame_info}` |
| SCStreamFrameInfoDirtyRects | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{dirty_rects, frame_info}` |
| SCStreamFrameInfoScreenRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{screen_rect, frame_info}` |
| SCStreamFrameInfoBoundingRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{bounding_rect, frame_info}` |
| SCStreamFrameInfoPresenterOverlayContentRect | extern const | SCStream.h | `cm::CMSampleBufferSCExt::{presenter_overlay_content_rect, frame_info}` |
| SCStreamOutput | protocol | SCStream.h | `stream::output_trait::SCStreamOutputTrait` (alias `stream::SCStreamOutput`) |
| SCStream | class | SCStream.h | `stream::SCStream` |

## 🔴 GAPS (macOS 27.0 SDK only)

| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| SCClipBufferingOutput | class | SCClipBufferingOutput.h | Rolling replay buffer with `exportClipToURL:duration:completionHandler:`; also needs the unwrapped `SCStream addClipBufferingOutput:` / `removeClipBufferingOutput:` |
| SCClipBufferingOutputDelegate | protocol | SCClipBufferingOutput.h | Buffering started / stopped / failed callbacks |
| SCRecordingEditor | class | SCRecordingEditor.h | `initWithURL:`, `delegate`, `presentFromWindow:completionHandler:` |
| SCRecordingEditorDelegate | protocol | SCRecordingEditor.h | Dismiss and failure callbacks |
| SCStreamFrameInfoVideoOrientation | extern const | SCStream.h | Frame attachment not read by `CMSampleBufferSCExt` or `FrameInfo` |

## ⏭️ EXEMPT

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| SCStreamType | NS_ENUM | SCStream.h | Apple deprecated it in favor of `SCShareableContentStyle`, so it is not scored. The crate still exposes it as the deprecated `stream::content_filter::SCStreamType`. | `API_DEPRECATED("Use SCShareableContentStyle instead", macos(14.0, 15.0))` |
