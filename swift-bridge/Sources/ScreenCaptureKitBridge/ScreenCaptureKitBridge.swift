// MARK: - ScreenCaptureKit Bridge
//
// Professional modular FFI bridge for ScreenCaptureKit to Rust
// Organized by functional domains with clear separation of concerns
//
// Modules:
// - Core: Memory management utilities
// - ShareableContent: Display, Window, Application APIs
// - Configuration: Stream configuration management
// - Stream: Stream lifecycle and control
// - Media: Sample buffer and pixel buffer handling
// - Output: IOSurface support

import CoreGraphics
import CoreMedia
import Foundation
import IOSurface
import ScreenCaptureKit

// MARK: - Core: Memory Management

/// Helper class to box value types for retain/release
private class Box<T> {
    var value: T
    init(_ value: T) {
        self.value = value
    }
}

/// Retains and returns an opaque pointer to a Swift object
private func retain<T: AnyObject>(_ obj: T) -> OpaquePointer {
    OpaquePointer(Unmanaged.passRetained(obj).toOpaque())
}

/// Gets an unretained reference to a Swift object from an opaque pointer
private func unretained<T: AnyObject>(_ ptr: OpaquePointer) -> T {
    Unmanaged<T>.fromOpaque(UnsafeRawPointer(ptr)).takeUnretainedValue()
}

/// Releases a retained Swift object
private func release(_ ptr: OpaquePointer) {
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(ptr)).release()
}

// MARK: - ShareableContent: Content Discovery

@_cdecl("sc_shareable_content_get")
public func getShareableContent(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                false,
                onScreenWindowsOnly: true
            )
            callback(retain(content), nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0) }
        }
    }
}

@_cdecl("sc_shareable_content_get_with_options")
public func getShareableContentWithOptions(
    excludeDesktopWindows: Bool,
    onScreenWindowsOnly: Bool,
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnly: onScreenWindowsOnly
            )
            callback(retain(content), nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0) }
        }
    }
}

@_cdecl("sc_shareable_content_get_current_process_displays")
public func getShareableContentCurrentProcessDisplays(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    if #available(macOS 14.4, *) {
        SCShareableContent.getCurrentProcessShareableContent { content, error in
            if let content = content {
                callback(retain(content), nil)
            } else {
                let errorMsg = error?.localizedDescription ?? "Unknown error"
                errorMsg.withCString { callback(nil, $0) }
            }
        }
    } else {
        // Fallback for older macOS
        Task {
            do {
                let content = try await SCShareableContent.excludingDesktopWindows(
                    false,
                    onScreenWindowsOnly: true
                )
                callback(retain(content), nil)
            } catch {
                let errorMsg = error.localizedDescription
                errorMsg.withCString { callback(nil, $0) }
            }
        }
    }
}

@_cdecl("sc_shareable_content_retain")
public func retainShareableContent(_ content: OpaquePointer) -> OpaquePointer {
    let sc: SCShareableContent = unretained(content)
    return retain(sc)
}

@_cdecl("sc_shareable_content_release")
public func releaseShareableContent(_ content: OpaquePointer) {
    release(content)
}

@_cdecl("sc_shareable_content_get_displays_count")
public func getShareableContentDisplaysCount(_ content: OpaquePointer) -> Int {
    let sc: SCShareableContent = unretained(content)
    return sc.displays.count
}

@_cdecl("sc_shareable_content_get_display_at")
public func getShareableContentDisplayAt(_ content: OpaquePointer, _ index: Int) -> OpaquePointer {
    let sc: SCShareableContent = unretained(content)
    return retain(sc.displays[index])
}

@_cdecl("sc_shareable_content_get_windows_count")
public func getShareableContentWindowsCount(_ content: OpaquePointer) -> Int {
    let sc: SCShareableContent = unretained(content)
    return sc.windows.count
}

@_cdecl("sc_shareable_content_get_window_at")
public func getShareableContentWindowAt(_ content: OpaquePointer, _ index: Int) -> OpaquePointer {
    let sc: SCShareableContent = unretained(content)
    return retain(sc.windows[index])
}

@_cdecl("sc_shareable_content_get_applications_count")
public func getShareableContentApplicationsCount(_ content: OpaquePointer) -> Int {
    let sc: SCShareableContent = unretained(content)
    return sc.applications.count
}

@_cdecl("sc_shareable_content_get_application_at")
public func getShareableContentApplicationAt(_ content: OpaquePointer, _ index: Int) -> OpaquePointer {
    let sc: SCShareableContent = unretained(content)
    return retain(sc.applications[index])
}

// MARK: - ShareableContent: SCDisplay

@_cdecl("sc_display_retain")
public func retainDisplay(_ display: OpaquePointer) -> OpaquePointer {
    let d: SCDisplay = unretained(display)
    return retain(d)
}

@_cdecl("sc_display_release")
public func releaseDisplay(_ display: OpaquePointer) {
    release(display)
}

@_cdecl("sc_display_get_display_id")
public func getDisplayID(_ display: OpaquePointer) -> UInt32 {
    let scDisplay: SCDisplay = unretained(display)
    return scDisplay.displayID
}

@_cdecl("sc_display_get_width")
public func getDisplayWidth(_ display: OpaquePointer) -> Int {
    let scDisplay: SCDisplay = unretained(display)
    return scDisplay.width
}

@_cdecl("sc_display_get_height")
public func getDisplayHeight(_ display: OpaquePointer) -> Int {
    let scDisplay: SCDisplay = unretained(display)
    return scDisplay.height
}

@_cdecl("sc_display_get_frame")
public func getDisplayFrame(
    _ display: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    let scDisplay: SCDisplay = unretained(display)
    let frame = scDisplay.frame
    x.pointee = frame.origin.x
    y.pointee = frame.origin.y
    width.pointee = frame.size.width
    height.pointee = frame.size.height
}

// MARK: - ShareableContent: SCWindow

@_cdecl("sc_window_retain")
public func retainWindow(_ window: OpaquePointer) -> OpaquePointer {
    let w: SCWindow = unretained(window)
    return retain(w)
}

@_cdecl("sc_window_release")
public func releaseWindow(_ window: OpaquePointer) {
    release(window)
}

@_cdecl("sc_window_get_window_id")
public func getWindowID(_ window: OpaquePointer) -> UInt32 {
    let scWindow: SCWindow = unretained(window)
    return scWindow.windowID
}

@_cdecl("sc_window_get_frame")
public func getWindowFrame(
    _ window: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    let scWindow: SCWindow = unretained(window)
    let frame = scWindow.frame
    x.pointee = frame.origin.x
    y.pointee = frame.origin.y
    width.pointee = frame.size.width
    height.pointee = frame.size.height
}

@_cdecl("sc_window_get_title")
public func getWindowTitle(
    _ window: OpaquePointer,
    _ buffer: UnsafeMutablePointer<CChar>,
    _ bufferSize: Int
) -> Bool {
    let scWindow: SCWindow = unretained(window)
    guard let title = scWindow.title else { return false }
    guard let cString = title.cString(using: .utf8), cString.count < bufferSize else { return false }
    cString.withUnsafeBufferPointer { ptr in
        buffer.update(from: ptr.baseAddress!, count: ptr.count)
    }
    return true
}

@_cdecl("sc_window_get_window_layer")
public func getWindowLayer(_ window: OpaquePointer) -> Int {
    let scWindow: SCWindow = unretained(window)
    return scWindow.windowLayer
}

@_cdecl("sc_window_is_on_screen")
public func getWindowIsOnScreen(_ window: OpaquePointer) -> Bool {
    let scWindow: SCWindow = unretained(window)
    return scWindow.isOnScreen
}

@_cdecl("sc_window_get_owning_application")
public func getWindowOwningApplication(_ window: OpaquePointer) -> OpaquePointer? {
    let scWindow: SCWindow = unretained(window)
    guard let app = scWindow.owningApplication else { return nil }
    return retain(app)
}

@_cdecl("sc_window_is_active")
public func getWindowIsActive(_ window: OpaquePointer) -> Bool {
    let scWindow: SCWindow = unretained(window)
    if #available(macOS 14.0, *) {
        return scWindow.isActive
    }
    return false
}

// MARK: - ShareableContent: SCRunningApplication

@_cdecl("sc_running_application_retain")
public func retainRunningApplication(_ app: OpaquePointer) -> OpaquePointer {
    let a: SCRunningApplication = unretained(app)
    return retain(a)
}

@_cdecl("sc_running_application_release")
public func releaseRunningApplication(_ app: OpaquePointer) {
    release(app)
}

@_cdecl("sc_running_application_get_bundle_identifier")
public func getRunningApplicationBundleIdentifier(
    _ app: OpaquePointer,
    _ buffer: UnsafeMutablePointer<CChar>,
    _ bufferSize: Int
) -> Bool {
    let scApp: SCRunningApplication = unretained(app)
    guard let cString = scApp.bundleIdentifier.cString(using: .utf8), cString.count < bufferSize else { return false }
    cString.withUnsafeBufferPointer { ptr in
        buffer.update(from: ptr.baseAddress!, count: ptr.count)
    }
    return true
}

@_cdecl("sc_running_application_get_application_name")
public func getRunningApplicationName(
    _ app: OpaquePointer,
    _ buffer: UnsafeMutablePointer<CChar>,
    _ bufferSize: Int
) -> Bool {
    let scApp: SCRunningApplication = unretained(app)
    guard let cString = scApp.applicationName.cString(using: .utf8), cString.count < bufferSize else { return false }
    cString.withUnsafeBufferPointer { ptr in
        buffer.update(from: ptr.baseAddress!, count: ptr.count)
    }
    return true
}

@_cdecl("sc_running_application_get_process_id")
public func getRunningApplicationProcessID(_ app: OpaquePointer) -> Int32 {
    let scApp: SCRunningApplication = unretained(app)
    return scApp.processID
}

// MARK: - Configuration: SCStreamConfiguration

@_cdecl("sc_stream_configuration_create")
public func createStreamConfiguration() -> OpaquePointer {
    retain(SCStreamConfiguration())
}

@_cdecl("sc_stream_configuration_retain")
public func retainStreamConfiguration(_ config: OpaquePointer) -> OpaquePointer {
    let c: SCStreamConfiguration = unretained(config)
    return retain(c)
}

@_cdecl("sc_stream_configuration_release")
public func releaseStreamConfiguration(_ config: OpaquePointer) {
    release(config)
}

@_cdecl("sc_stream_configuration_set_width")
public func setStreamConfigurationWidth(_ config: OpaquePointer, _ width: Int) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.width = width
}

@_cdecl("sc_stream_configuration_get_width")
public func getStreamConfigurationWidth(_ config: OpaquePointer) -> Int {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.width
}

@_cdecl("sc_stream_configuration_set_height")
public func setStreamConfigurationHeight(_ config: OpaquePointer, _ height: Int) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.height = height
}

@_cdecl("sc_stream_configuration_get_height")
public func getStreamConfigurationHeight(_ config: OpaquePointer) -> Int {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.height
}

@_cdecl("sc_stream_configuration_set_shows_cursor")
public func setStreamConfigurationShowsCursor(_ config: OpaquePointer, _ showsCursor: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.showsCursor = showsCursor
}

@_cdecl("sc_stream_configuration_get_shows_cursor")
public func getStreamConfigurationShowsCursor(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.showsCursor
}

@_cdecl("sc_stream_configuration_set_scales_to_fit")
public func setStreamConfigurationScalesToFit(_ config: OpaquePointer, _ scalesToFit: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.scalesToFit = scalesToFit
}

@_cdecl("sc_stream_configuration_get_scales_to_fit")
public func getStreamConfigurationScalesToFit(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.scalesToFit
}

@_cdecl("sc_stream_configuration_set_captures_audio")
public func setStreamConfigurationCapturesAudio(_ config: OpaquePointer, _ capturesAudio: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.capturesAudio = capturesAudio
}

@_cdecl("sc_stream_configuration_get_captures_audio")
public func getStreamConfigurationCapturesAudio(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.capturesAudio
}

@_cdecl("sc_stream_configuration_set_sample_rate")
public func setStreamConfigurationSampleRate(_ config: OpaquePointer, _ sampleRate: Int) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.sampleRate = sampleRate
}

@_cdecl("sc_stream_configuration_get_sample_rate")
public func getStreamConfigurationSampleRate(_ config: OpaquePointer) -> Int {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.sampleRate
}

@_cdecl("sc_stream_configuration_set_channel_count")
public func setStreamConfigurationChannelCount(_ config: OpaquePointer, _ channelCount: Int) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.channelCount = channelCount
}

@_cdecl("sc_stream_configuration_get_channel_count")
public func getStreamConfigurationChannelCount(_ config: OpaquePointer) -> Int {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.channelCount
}

@_cdecl("sc_stream_configuration_set_minimum_frame_interval")
public func setStreamConfigurationMinimumFrameInterval(_ config: OpaquePointer, _ seconds: Double) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.minimumFrameInterval = CMTime(seconds: seconds, preferredTimescale: 1000)
}

@_cdecl("sc_stream_configuration_get_minimum_frame_interval")
public func getStreamConfigurationMinimumFrameInterval(_ config: OpaquePointer) -> Double {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.minimumFrameInterval.seconds
}

@_cdecl("sc_stream_configuration_set_queue_depth")
public func setStreamConfigurationQueueDepth(_ config: OpaquePointer, _ depth: Int) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.queueDepth = depth
}

@_cdecl("sc_stream_configuration_get_queue_depth")
public func getStreamConfigurationQueueDepth(_ config: OpaquePointer) -> Int {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.queueDepth
}

@_cdecl("sc_stream_configuration_set_pixel_format")
public func setStreamConfigurationPixelFormat(_ config: OpaquePointer, _ format: UInt32) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.pixelFormat = format
}

@_cdecl("sc_stream_configuration_get_pixel_format")
public func getStreamConfigurationPixelFormat(_ config: OpaquePointer) -> UInt32 {
    let scConfig: SCStreamConfiguration = unretained(config)
    return scConfig.pixelFormat
}

@_cdecl("sc_stream_configuration_set_background_color")
public func setStreamConfigurationBackgroundColor(_ config: OpaquePointer, _ r: Float, _ g: Float, _ b: Float) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let color = CGColor(red: CGFloat(r), green: CGFloat(g), blue: CGFloat(b), alpha: 1.0)
    scConfig.backgroundColor = color
}

@_cdecl("sc_stream_configuration_set_color_space_name")
public func setStreamConfigurationColorSpaceName(_ config: OpaquePointer, _ name: UnsafePointer<CChar>) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let colorSpaceName = String(cString: name)
    scConfig.colorSpaceName = colorSpaceName as CFString
}

@_cdecl("sc_stream_configuration_set_should_be_opaque")
public func setStreamConfigurationShouldBeOpaque(_ config: OpaquePointer, _ shouldBeOpaque: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        scConfig.shouldBeOpaque = shouldBeOpaque
    }
}

@_cdecl("sc_stream_configuration_get_should_be_opaque")
public func getStreamConfigurationShouldBeOpaque(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return scConfig.shouldBeOpaque
    }
    return false
}

// Placeholder getters/setters for unsupported properties
@_cdecl("sc_stream_configuration_set_ignores_shadow_display_configuration")
public func setStreamConfigurationIgnoresShadowDisplayConfiguration(_ config: OpaquePointer, _ ignores: Bool) {}

@_cdecl("sc_stream_configuration_get_ignores_shadow_display_configuration")
public func getStreamConfigurationIgnoresShadowDisplayConfiguration(_ config: OpaquePointer) -> Bool { false }

// MARK: - Source and Destination Rectangles

@_cdecl("sc_stream_configuration_set_source_rect")
public func setStreamConfigurationSourceRect(_ config: OpaquePointer, _ x: Double, _ y: Double, _ width: Double, _ height: Double) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.sourceRect = CGRect(x: x, y: y, width: width, height: height)
}

@_cdecl("sc_stream_configuration_get_source_rect")
public func getStreamConfigurationSourceRect(_ config: OpaquePointer, _ x: UnsafeMutablePointer<Double>, _ y: UnsafeMutablePointer<Double>, _ width: UnsafeMutablePointer<Double>, _ height: UnsafeMutablePointer<Double>) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let rect = scConfig.sourceRect
    x.pointee = rect.origin.x
    y.pointee = rect.origin.y
    width.pointee = rect.size.width
    height.pointee = rect.size.height
}

@_cdecl("sc_stream_configuration_set_destination_rect")
public func setStreamConfigurationDestinationRect(_ config: OpaquePointer, _ x: Double, _ y: Double, _ width: Double, _ height: Double) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.destinationRect = CGRect(x: x, y: y, width: width, height: height)
}

@_cdecl("sc_stream_configuration_get_destination_rect")
public func getStreamConfigurationDestinationRect(_ config: OpaquePointer, _ x: UnsafeMutablePointer<Double>, _ y: UnsafeMutablePointer<Double>, _ width: UnsafeMutablePointer<Double>, _ height: UnsafeMutablePointer<Double>) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let rect = scConfig.destinationRect
    x.pointee = rect.origin.x
    y.pointee = rect.origin.y
    width.pointee = rect.size.width
    height.pointee = rect.size.height
}

@_cdecl("sc_stream_configuration_set_preserves_aspect_ratio")
public func setStreamConfigurationPreservesAspectRatio(_ config: OpaquePointer, _ preserves: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        scConfig.preservesAspectRatio = preserves
    }
}

@_cdecl("sc_stream_configuration_get_preserves_aspect_ratio")
public func getStreamConfigurationPreservesAspectRatio(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return scConfig.preservesAspectRatio
    }
    return false
}

// MARK: - Other Configuration Properties

@_cdecl("sc_stream_configuration_set_preserve_aspect_ratio")
public func setStreamConfigurationPreserveAspectRatio(_ config: OpaquePointer, _ preserves: Bool) {
    // Legacy name - forward to new function
    setStreamConfigurationPreservesAspectRatio(config, preserves)
}

@_cdecl("sc_stream_configuration_get_preserve_aspect_ratio")
public func getStreamConfigurationPreserveAspectRatio(_ config: OpaquePointer) -> Bool {
    // Legacy name - forward to new function
    getStreamConfigurationPreservesAspectRatio(config)
}

@_cdecl("sc_stream_configuration_set_ignore_global_clipboard")
public func setStreamConfigurationIgnoreGlobalClipboard(_ config: OpaquePointer, _ ignores: Bool) {}

@_cdecl("sc_stream_configuration_get_ignore_global_clipboard")
public func getStreamConfigurationIgnoreGlobalClipboard(_ config: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_capture_resolution")
public func setStreamConfigurationCaptureResolution(_ config: OpaquePointer, _ resolution: Int32) {}

@_cdecl("sc_stream_configuration_get_capture_resolution")
public func getStreamConfigurationCaptureResolution(_ config: OpaquePointer) -> Int32 { 0 }

@_cdecl("sc_stream_configuration_set_color_matrix")
public func setStreamConfigurationColorMatrix(_ config: OpaquePointer, _ matrix: UnsafePointer<CChar>) {}

@_cdecl("sc_stream_configuration_set_increase_resolution_for_retina_displays")
public func setStreamConfigurationIncreaseResolutionForRetinaDisplays(_ config: OpaquePointer, _ increases: Bool) {}

@_cdecl("sc_stream_configuration_get_increase_resolution_for_retina_displays")
public func getStreamConfigurationIncreaseResolutionForRetinaDisplays(_ config: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_ignore_fraction_of_screen")
public func setStreamConfigurationIgnoreFractionOfScreen(_ config: OpaquePointer, _ fraction: Double) {}

@_cdecl("sc_stream_configuration_get_ignore_fraction_of_screen")
public func getStreamConfigurationIgnoreFractionOfScreen(_ config: OpaquePointer) -> Double { 0.0 }

@_cdecl("sc_stream_configuration_set_ignores_shadows_single_window")
public func setStreamConfigurationIgnoresShadowsSingleWindow(_ config: OpaquePointer, _ ignoresShadows: Bool) {}

@_cdecl("sc_stream_configuration_get_ignores_shadows_single_window")
public func getStreamConfigurationIgnoresShadowsSingleWindow(_ config: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_includes_child_windows")
public func setStreamConfigurationIncludesChildWindows(_ config: OpaquePointer, _ includesChildWindows: Bool) {}

@_cdecl("sc_stream_configuration_get_includes_child_windows")
public func getStreamConfigurationIncludesChildWindows(_ config: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_presenter_overlay_privacy_alert_setting")
public func setStreamConfigurationPresenterOverlayPrivacyAlertSetting(_ config: OpaquePointer, _ setting: Int) {}

@_cdecl("sc_stream_configuration_get_presenter_overlay_privacy_alert_setting")
public func getStreamConfigurationPresenterOverlayPrivacyAlertSetting(_ config: OpaquePointer) -> Int { 1 }

@_cdecl("sc_stream_configuration_set_captures_shadows_only")
public func setStreamConfigurationCapturesShadowsOnly(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        cfg.capturesShadowsOnly = value
    }
}

@_cdecl("sc_stream_configuration_get_captures_shadows_only")
public func getStreamConfigurationCapturesShadowsOnly(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return cfg.capturesShadowsOnly
    }
    return false
}

@_cdecl("sc_stream_configuration_set_captures_microphone")
public func setStreamConfigurationCapturesMicrophone(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        cfg.captureMicrophone = value
    }
}

@_cdecl("sc_stream_configuration_get_captures_microphone")
public func getStreamConfigurationCapturesMicrophone(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        return cfg.captureMicrophone
    }
    return false
}

@_cdecl("sc_stream_configuration_set_excludes_current_process_audio")
public func setStreamConfigurationExcludesCurrentProcessAudio(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    cfg.excludesCurrentProcessAudio = value
}

@_cdecl("sc_stream_configuration_get_excludes_current_process_audio")
public func getStreamConfigurationExcludesCurrentProcessAudio(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    return cfg.excludesCurrentProcessAudio
}

@_cdecl("sc_stream_configuration_set_microphone_capture_device_id")
public func setStreamConfigurationMicrophoneCaptureDeviceId(_ config: OpaquePointer, _ deviceId: UnsafePointer<CChar>?) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        if let deviceId = deviceId {
            cfg.microphoneCaptureDeviceID = String(cString: deviceId)
        } else {
            cfg.microphoneCaptureDeviceID = nil
        }
    }
}

@_cdecl("sc_stream_configuration_get_microphone_capture_device_id")
public func getStreamConfigurationMicrophoneCaptureDeviceId(_ config: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        if let deviceId = cfg.microphoneCaptureDeviceID {
            guard let cString = deviceId.cString(using: .utf8), cString.count < bufferSize else {
                return false
            }
            buffer.initialize(from: cString, count: min(cString.count, bufferSize))
            return true
        }
    }
    return false
}

@_cdecl("sc_stream_configuration_set_stream_name")
public func setStreamConfigurationStreamName(_ config: OpaquePointer, _ name: UnsafePointer<CChar>?) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        if let name = name {
            cfg.streamName = String(cString: name)
        } else {
            cfg.streamName = nil
        }
    }
}

@_cdecl("sc_stream_configuration_get_stream_name")
public func getStreamConfigurationStreamName(_ config: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        if let streamName = cfg.streamName {
            guard let cString = streamName.cString(using: .utf8), cString.count < bufferSize else {
                return false
            }
            buffer.initialize(from: cString, count: min(cString.count, bufferSize))
            return true
        }
    }
    return false
}

@_cdecl("sc_stream_configuration_set_capture_dynamic_range")
public func setStreamConfigurationCaptureDynamicRange(_ config: OpaquePointer, _ value: Int32) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        switch value {
        case 0:
            cfg.captureDynamicRange = .SDR
        case 1:
            cfg.captureDynamicRange = .hdrLocalDisplay
        case 2:
            cfg.captureDynamicRange = .hdrCanonicalDisplay
        default:
            cfg.captureDynamicRange = .SDR
        }
    }
}

@_cdecl("sc_stream_configuration_get_capture_dynamic_range")
public func getStreamConfigurationCaptureDynamicRange(_ config: OpaquePointer) -> Int32 {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 15.0, *) {
        switch cfg.captureDynamicRange {
        case .SDR:
            return 0
        case .hdrLocalDisplay:
            return 1
        case .hdrCanonicalDisplay:
            return 2
        @unknown default:
            return 0
        }
    }
    return 0
}

// MARK: - Stream: SCContentFilter

@_cdecl("sc_content_filter_create_with_desktop_independent_window")
public func createContentFilterWithDesktopIndependentWindow(_ window: OpaquePointer) -> OpaquePointer {
    let scWindow: SCWindow = unretained(window)
    let filter = SCContentFilter(desktopIndependentWindow: scWindow)
    return retain(filter)
}

@_cdecl("sc_content_filter_create_with_display_excluding_windows")
public func createContentFilterWithDisplayExcludingWindows(
    _ display: OpaquePointer,
    _ windows: UnsafePointer<OpaquePointer>?,
    _ windowsCount: Int
) -> OpaquePointer {
    let scDisplay: SCDisplay = unretained(display)
    var excludedWindows: [SCWindow] = []
    if let windows = windows {
        for i in 0..<windowsCount {
            let window: SCWindow = unretained(windows[i])
            excludedWindows.append(window)
        }
    }
    let filter = SCContentFilter(display: scDisplay, excludingWindows: excludedWindows)
    return retain(filter)
}

@_cdecl("sc_content_filter_create_with_display_including_windows")
public func createContentFilterWithDisplayIncludingWindows(
    _ display: OpaquePointer,
    _ windows: UnsafePointer<OpaquePointer>?,
    _ windowsCount: Int
) -> OpaquePointer {
    let scDisplay: SCDisplay = unretained(display)
    var includedWindows: [SCWindow] = []
    if let windows = windows {
        for i in 0..<windowsCount {
            let window: SCWindow = unretained(windows[i])
            includedWindows.append(window)
        }
    }
    let filter = SCContentFilter(display: scDisplay, including: includedWindows)
    return retain(filter)
}

@_cdecl("sc_content_filter_create_with_display_including_applications_excepting_windows")
public func createContentFilterWithDisplayIncludingApplicationsExceptingWindows(
    _ display: OpaquePointer,
    _ apps: UnsafePointer<OpaquePointer>?,
    _ appsCount: Int,
    _ windows: UnsafePointer<OpaquePointer>?,
    _ windowsCount: Int
) -> OpaquePointer {
    let scDisplay: SCDisplay = unretained(display)
    var includedApps: [SCRunningApplication] = []
    if let apps = apps {
        for i in 0..<appsCount {
            let app: SCRunningApplication = unretained(apps[i])
            includedApps.append(app)
        }
    }
    var exceptedWindows: [SCWindow] = []
    if let windows = windows {
        for i in 0..<windowsCount {
            let window: SCWindow = unretained(windows[i])
            exceptedWindows.append(window)
        }
    }
    let filter = SCContentFilter(display: scDisplay, including: includedApps, exceptingWindows: exceptedWindows)
    return retain(filter)
}

@_cdecl("sc_content_filter_retain")
public func retainContentFilter(_ filter: OpaquePointer) -> OpaquePointer {
    let f: SCContentFilter = unretained(filter)
    return retain(f)
}

@_cdecl("sc_content_filter_release")
public func releaseContentFilter(_ filter: OpaquePointer) {
    release(filter)
}

@_cdecl("sc_content_filter_set_content_rect")
public func setContentFilterContentRect(_ filter: OpaquePointer, _ x: Double, _ y: Double, _ width: Double, _ height: Double) {}

@_cdecl("sc_content_filter_get_content_rect")
public func getContentFilterContentRect(
    _ filter: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    x.pointee = 0.0
    y.pointee = 0.0
    width.pointee = 0.0
    height.pointee = 0.0
}

// MARK: - Stream: SCStream Delegates and Handlers

private class StreamDelegateWrapper: NSObject, SCStreamDelegate {
    let errorCallback: @convention(c) (OpaquePointer, UnsafePointer<CChar>) -> Void
    let streamPtr: OpaquePointer

    init(streamPtr: OpaquePointer, errorCallback: @escaping @convention(c) (OpaquePointer, UnsafePointer<CChar>) -> Void) {
        self.streamPtr = streamPtr
        self.errorCallback = errorCallback
    }

    func stream(_ stream: SCStream, didStopWithError error: Error) {
        let errorMsg = error.localizedDescription
        errorMsg.withCString { errorCallback(streamPtr, $0) }
    }
}

private class StreamOutputHandler: NSObject, SCStreamOutput {
    let sampleBufferCallback: @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void
    let streamPtr: OpaquePointer

    init(streamPtr: OpaquePointer, sampleBufferCallback: @escaping @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void) {
        self.streamPtr = streamPtr
        self.sampleBufferCallback = sampleBufferCallback
    }

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        let outputType: Int32 = type == .screen ? 0 : 1
        // IMPORTANT: passRetained() is used here to retain the CMSampleBuffer for Rust
        // The Rust side will release it when CMSampleBuffer is dropped
        sampleBufferCallback(streamPtr, OpaquePointer(Unmanaged.passRetained(sampleBuffer as AnyObject).toOpaque()), outputType)
    }
}

// Registry to store handlers associated with streams
private class HandlerRegistry {
    private var handlers: [String: StreamOutputHandler] = [:]
    private let lock = NSLock()
    
    private func key(for stream: OpaquePointer, type: Int32) -> String {
        return "\(UInt(bitPattern: stream))_\(type)"
    }

    func store(_ handler: StreamOutputHandler, for stream: OpaquePointer, type: Int32) {
        lock.lock()
        defer { lock.unlock() }
        handlers[key(for: stream, type: type)] = handler
    }

    func get(for stream: OpaquePointer, type: Int32) -> StreamOutputHandler? {
        lock.lock()
        defer { lock.unlock() }
        return handlers[key(for: stream, type: type)]
    }

    func remove(for stream: OpaquePointer, type: Int32) {
        lock.lock()
        defer { lock.unlock() }
        handlers.removeValue(forKey: key(for: stream, type: type))
    }
}

private let handlerRegistry = HandlerRegistry()

// MARK: - Stream: SCStream Control

@_cdecl("sc_stream_create")
public func createStream(
    _ filter: OpaquePointer,
    _ config: OpaquePointer,
    _ errorCallback: @escaping @convention(c) (OpaquePointer, UnsafePointer<CChar>) -> Void
) -> OpaquePointer? {
    let scFilter: SCContentFilter = unretained(filter)
    let scConfig: SCStreamConfiguration = unretained(config)

    let streamPtr = OpaquePointer(bitPattern: 1)!
    let delegate = StreamDelegateWrapper(streamPtr: streamPtr, errorCallback: errorCallback)

    let stream = SCStream(filter: scFilter, configuration: scConfig, delegate: delegate)
    let actualStreamPtr = retain(stream)

    return actualStreamPtr
}

@_cdecl("sc_stream_add_stream_output")
public func addStreamOutput(
    _ stream: OpaquePointer,
    _ type: Int32,
    _ sampleBufferCallback: @escaping @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void
) -> Bool {
    let scStream: SCStream = unretained(stream)
    let handler = StreamOutputHandler(streamPtr: stream, sampleBufferCallback: sampleBufferCallback)
    handlerRegistry.store(handler, for: stream, type: type)

    let outputType: SCStreamOutputType
    if type == 0 {
        outputType = .screen
    } else if type == 2 {
        if #available(macOS 15.0, *) {
            outputType = .microphone
        } else {
            outputType = .audio  // Fallback for older macOS
        }
    } else {
        outputType = .audio
    }

    // Use a dedicated queue instead of .main to avoid runloop dependency
    let queue = DispatchQueue(label: "com.screencapturekit.output", qos: .userInteractive)

    do {
        try scStream.addStreamOutput(handler, type: outputType, sampleHandlerQueue: queue)
        return true
    } catch {
        return false
    }
}

@_cdecl("sc_stream_add_stream_output_with_queue")
public func addStreamOutputWithQueue(
    _ stream: OpaquePointer,
    _ type: Int32,
    _ sampleBufferCallback: @escaping @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void,
    _ dispatchQueue: OpaquePointer?
) -> Bool {
    let scStream: SCStream = unretained(stream)
    let handler = StreamOutputHandler(streamPtr: stream, sampleBufferCallback: sampleBufferCallback)
    handlerRegistry.store(handler, for: stream, type: type)

    let outputType: SCStreamOutputType
    if type == 0 {
        outputType = .screen
    } else if type == 2 {
        if #available(macOS 15.0, *) {
            outputType = .microphone
        } else {
            outputType = .audio  // Fallback for older macOS
        }
    } else {
        outputType = .audio
    }

    let queue: DispatchQueue
    if let queuePtr = dispatchQueue {
        queue = unretained(queuePtr)
    } else {
        queue = DispatchQueue(label: "com.screencapturekit.output", qos: .userInteractive)
    }

    do {
        try scStream.addStreamOutput(handler, type: outputType, sampleHandlerQueue: queue)
        return true
    } catch {
        return false
    }
}

@_cdecl("sc_stream_remove_stream_output")
public func removeStreamOutput(
    _ stream: OpaquePointer,
    _ type: Int32
) -> Bool {
    let scStream: SCStream = unretained(stream)
    guard let handler = handlerRegistry.get(for: stream, type: type) else { return false }

    let outputType: SCStreamOutputType
    if type == 0 {
        outputType = .screen
    } else if type == 2 {
        if #available(macOS 15.0, *) {
            outputType = .microphone
        } else {
            outputType = .audio  // Fallback for older macOS
        }
    } else {
        outputType = .audio
    }

    do {
        try scStream.removeStreamOutput(handler, type: outputType)
        handlerRegistry.remove(for: stream, type: type)
        return true
    } catch {
        return false
    }
}

// MARK: - Dispatch Queue Management

@_cdecl("dispatch_queue_create")
public func createDispatchQueue(_ label: UnsafePointer<CChar>, _ qos: Int32) -> OpaquePointer {
    let labelStr = String(cString: label)
    let qosClass: DispatchQoS
    
    switch qos {
    case 0: qosClass = .background
    case 1: qosClass = .utility
    case 2: qosClass = .default
    case 3: qosClass = .userInitiated
    case 4: qosClass = .userInteractive
    default: qosClass = .default
    }
    
    let queue = DispatchQueue(label: labelStr, qos: qosClass)
    return retain(queue)
}

@_cdecl("dispatch_queue_release")
public func releaseDispatchQueue(_ queue: OpaquePointer) {
    release(queue)
}

@_cdecl("sc_stream_start_capture")
public func startStreamCapture(
    _ stream: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.startCapture()
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_stop_capture")
public func stopStreamCapture(
    _ stream: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.stopCapture()
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_update_content_filter")
public func updateStreamContentFilter(
    _ stream: OpaquePointer,
    _ filter: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    let scFilter: SCContentFilter = unretained(filter)
    Task {
        do {
            try await scStream.updateContentFilter(scFilter)
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_update_configuration")
public func updateStreamConfiguration(
    _ stream: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    let scConfig: SCStreamConfiguration = unretained(config)
    Task {
        do {
            try await scStream.updateConfiguration(scConfig)
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_retain")
public func retainStream(_ stream: OpaquePointer) -> OpaquePointer {
    let s: SCStream = unretained(stream)
    return retain(s)
}

@_cdecl("sc_stream_release")
public func releaseStream(_ stream: OpaquePointer) {
    release(stream)
}

// MARK: - Media: CMSampleBuffer

@_cdecl("cm_sample_buffer_get_image_buffer")
public func getSampleBufferImageBuffer(_ sampleBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    guard let imageBuffer = CMSampleBufferGetImageBuffer(buffer) else { return nil }
    return OpaquePointer(Unmanaged.passRetained(imageBuffer as AnyObject).toOpaque())
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp")
public func getSampleBufferPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_get_decode_timestamp")
public func getSampleBufferDecodeTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetDecodeTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_get_output_presentation_timestamp")
public func getSampleBufferOutputPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetOutputPresentationTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_set_output_presentation_timestamp")
public func setSampleBufferOutputPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: Int64,
    _ timescale: Int32,
    _ flags: UInt32,
    _ epoch: Int64
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMTime(value: CMTimeValue(value), timescale: timescale, flags: CMTimeFlags(rawValue: flags), epoch: epoch)
    return CMSampleBufferSetOutputPresentationTimeStamp(buffer, newValue: time)
}

@_cdecl("cm_sample_buffer_get_format_description")
public func getSampleBufferFormatDescription(_ sampleBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    guard let formatDesc = CMSampleBufferGetFormatDescription(buffer) else { return nil }
    return OpaquePointer(Unmanaged.passRetained(formatDesc as AnyObject).toOpaque())
}

@_cdecl("cm_sample_buffer_get_sample_size")
public func getSampleBufferSampleSize(_ sampleBuffer: OpaquePointer, _ sampleIndex: Int) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferGetSampleSize(buffer, at: sampleIndex)
}

@_cdecl("cm_sample_buffer_get_total_sample_size")
public func getSampleBufferTotalSampleSize(_ sampleBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferGetTotalSampleSize(buffer)
}

@_cdecl("cm_sample_buffer_is_ready_for_data_access")
public func isSampleBufferReadyForDataAccess(_ sampleBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferDataIsReady(buffer)
}

@_cdecl("cm_sample_buffer_make_data_ready")
public func makeSampleBufferDataReady(_ sampleBuffer: OpaquePointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferMakeDataReady(buffer)
}

@_cdecl("cm_sample_buffer_release")
public func releaseSampleBuffer(_ sampleBuffer: OpaquePointer) {
    // Release the retained CMSampleBuffer that was passed from didOutputSampleBuffer
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(sampleBuffer)).release()
}

// MARK: - Media: CVPixelBuffer

@_cdecl("cv_pixel_buffer_get_width")
public func getPixelBufferWidth(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetWidth(buffer)
}

@_cdecl("cv_pixel_buffer_get_height")
public func getPixelBufferHeight(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetHeight(buffer)
}

@_cdecl("cv_pixel_buffer_get_pixel_format_type")
public func getPixelBufferPixelFormatType(_ pixelBuffer: OpaquePointer) -> UInt32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetPixelFormatType(buffer)
}

@_cdecl("cv_pixel_buffer_lock_base_address")
public func lockPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer, _ flags: UInt64) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: flags))
}

@_cdecl("cv_pixel_buffer_unlock_base_address")
public func unlockPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer, _ flags: UInt64) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: flags))
}

@_cdecl("cv_pixel_buffer_get_base_address")
public func getPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBaseAddress(buffer)
}

@_cdecl("cv_pixel_buffer_get_bytes_per_row")
public func getPixelBufferBytesPerRow(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBytesPerRow(buffer)
}

@_cdecl("cv_pixel_buffer_release")
public func releasePixelBuffer(_ pixelBuffer: OpaquePointer) {
    release(pixelBuffer)
}

// MARK: - Output: IOSurface

@_cdecl("cv_pixel_buffer_get_iosurface")
public func getPixelBufferIOSurface(_ pixelBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    guard let ioSurface = CVPixelBufferGetIOSurface(buffer)?.takeUnretainedValue() else {
        return nil
    }
    return OpaquePointer(Unmanaged.passRetained(ioSurface).toOpaque())
}

@_cdecl("cv_pixel_buffer_is_backed_by_iosurface")
public func isPixelBufferBackedByIOSurface(_ pixelBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetIOSurface(buffer) != nil
}

@_cdecl("cv_pixel_buffer_retain")
public func retainPixelBuffer(_ pixelBuffer: OpaquePointer) -> OpaquePointer {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue()
    return OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
}

// MARK: - CVPixelBuffer Creation

@_cdecl("cv_pixel_buffer_create")
public func createPixelBuffer(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ pixelBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    var pixelBuffer: CVPixelBuffer?
    let status = CVPixelBufferCreate(
        kCFAllocatorDefault,
        width,
        height,
        OSType(pixelFormatType),
        nil,
        &pixelBuffer
    )

    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        pixelBufferOut.pointee = nil
    }

    return status
}

@_cdecl("cv_pixel_buffer_create_with_bytes")
public func createPixelBufferWithBytes(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ baseAddress: UnsafeMutableRawPointer,
    _ bytesPerRow: Int,
    _ pixelBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    var pixelBuffer: CVPixelBuffer?
    let status = CVPixelBufferCreateWithBytes(
        kCFAllocatorDefault,
        width,
        height,
        OSType(pixelFormatType),
        baseAddress,
        bytesPerRow,
        nil,
        nil,
        nil,
        &pixelBuffer
    )

    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        pixelBufferOut.pointee = nil
    }

    return status
}

@_cdecl("cv_pixel_buffer_fill_extended_pixels")
public func fillPixelBufferExtendedPixels(_ pixelBuffer: OpaquePointer) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferFillExtendedPixels(buffer)
}

@_cdecl("cv_pixel_buffer_get_data_size")
public func getPixelBufferDataSize(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetDataSize(buffer)
}

@_cdecl("cv_pixel_buffer_is_planar")
public func isPixelBufferPlanar(_ pixelBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferIsPlanar(buffer)
}

@_cdecl("cv_pixel_buffer_get_plane_count")
public func getPixelBufferPlaneCount(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetPlaneCount(buffer)
}

@_cdecl("cv_pixel_buffer_get_width_of_plane")
public func getPixelBufferWidthOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetWidthOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_height_of_plane")
public func getPixelBufferHeightOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetHeightOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_base_address_of_plane")
public func getPixelBufferBaseAddressOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBaseAddressOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_bytes_per_row_of_plane")
public func getPixelBufferBytesPerRowOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBytesPerRowOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_extended_pixels")
public func getPixelBufferExtendedPixels(
    _ pixelBuffer: OpaquePointer,
    _ extraColumnsOnLeft: UnsafeMutablePointer<Int>,
    _ extraColumnsOnRight: UnsafeMutablePointer<Int>,
    _ extraRowsOnTop: UnsafeMutablePointer<Int>,
    _ extraRowsOnBottom: UnsafeMutablePointer<Int>
) {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    CVPixelBufferGetExtendedPixels(buffer,
                                   extraColumnsOnLeft,
                                   extraColumnsOnRight,
                                   extraRowsOnTop,
                                   extraRowsOnBottom)
}

// MARK: - CMSampleBuffer Creation

@_cdecl("cm_sample_buffer_create_for_image_buffer")
public func createSampleBufferForImageBuffer(
    _ imageBuffer: OpaquePointer,
    _ presentationTimeValue: Int64,
    _ presentationTimeScale: Int32,
    _ durationValue: Int64,
    _ durationScale: Int32,
    _ sampleBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    let pixelBuffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(imageBuffer)).takeUnretainedValue() as! CVPixelBuffer

    var sampleBuffer: CMSampleBuffer?
    var timingInfo = CMSampleTimingInfo(
        duration: CMTime(value: CMTimeValue(durationValue), timescale: durationScale, flags: .valid, epoch: 0),
        presentationTimeStamp: CMTime(value: CMTimeValue(presentationTimeValue), timescale: presentationTimeScale, flags: .valid, epoch: 0),
        decodeTimeStamp: .invalid
    )

    var formatDescription: CMFormatDescription?
    let descStatus = CMVideoFormatDescriptionCreateForImageBuffer(
        allocator: kCFAllocatorDefault,
        imageBuffer: pixelBuffer,
        formatDescriptionOut: &formatDescription
    )

    guard descStatus == noErr, let format = formatDescription else {
        sampleBufferOut.pointee = nil
        return descStatus
    }

    let status = CMSampleBufferCreateReadyWithImageBuffer(
        allocator: kCFAllocatorDefault,
        imageBuffer: pixelBuffer,
        formatDescription: format,
        sampleTiming: &timingInfo,
        sampleBufferOut: &sampleBuffer
    )

    if status == noErr, let buffer = sampleBuffer {
        sampleBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        sampleBufferOut.pointee = nil
    }

    return status
}

@_cdecl("iosurface_get_width")
public func getIOSurfaceWidth(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetWidth(surface)
}

@_cdecl("iosurface_get_height")
public func getIOSurfaceHeight(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetHeight(surface)
}

@_cdecl("iosurface_get_bytes_per_row")
public func getIOSurfaceBytesPerRow(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetBytesPerRow(surface)
}

@_cdecl("iosurface_get_pixel_format")
public func getIOSurfacePixelFormat(_ ioSurface: OpaquePointer) -> UInt32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetPixelFormat(surface)
}

@_cdecl("iosurface_get_base_address")
public func getIOSurfaceBaseAddress(_ ioSurface: OpaquePointer) -> UnsafeMutableRawPointer? {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetBaseAddress(surface)
}

@_cdecl("iosurface_lock")
public func lockIOSurface(_ ioSurface: OpaquePointer, options: UInt32) -> Int32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceLock(surface, IOSurfaceLockOptions(rawValue: options), nil)
}

@_cdecl("iosurface_unlock")
public func unlockIOSurface(_ ioSurface: OpaquePointer, options: UInt32) -> Int32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceUnlock(surface, IOSurfaceLockOptions(rawValue: options), nil)
}

@_cdecl("iosurface_is_in_use")
public func isIOSurfaceInUse(_ ioSurface: OpaquePointer) -> Bool {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceIsInUse(surface)
}

@_cdecl("iosurface_release")
public func releaseIOSurface(_ ioSurface: OpaquePointer) {
    Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).release()
}

// MARK: - Content Sharing Picker (macOS 14.0+)

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_create")
public func createContentSharingPickerConfiguration() -> OpaquePointer {
    let config = SCContentSharingPickerConfiguration()
    let box = Box(config)
    return retain(box)
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_set_allowed_picker_modes")
public func setContentSharingPickerAllowedModes(
    _ config: OpaquePointer,
    _ modes: UnsafePointer<Int32>,
    _ count: Int
) {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let modesArray = Array(UnsafeBufferPointer(start: modes, count: count))
    var pickerModes: [SCContentSharingPickerMode] = []
    for mode in modesArray {
        switch mode {
        case 0: pickerModes.append(.singleWindow)
        case 1: pickerModes.append(.multipleWindows)
        case 2: pickerModes.append(.singleDisplay)
        default: break
        }
    }
    box.value.allowedPickerModes = pickerModes.first ?? .singleWindow
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_retain")
public func retainContentSharingPickerConfiguration(_ config: OpaquePointer) -> OpaquePointer {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return retain(box)
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_release")
public func releaseContentSharingPickerConfiguration(_ config: OpaquePointer) {
    release(config)
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show")
public func showContentSharingPicker(
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    // Note: SCContentSharingPicker API requires specific integration
    // For now, return cancelled. Full implementation would need proper UI handling
    callback(0, nil, userData)
}

// MARK: - Recording Output (macOS 15.0+)

@available(macOS 15.0, *)
private class RecordingDelegate: NSObject, SCRecordingOutputDelegate {
    func recordingOutput(_ recordingOutput: SCRecordingOutput, didFailWithError error: Error) {
    }

    func recordingOutputDidFinishRecording(_ recordingOutput: SCRecordingOutput) {
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_create")
public func createRecordingOutputConfiguration() -> OpaquePointer {
    let config = SCRecordingOutputConfiguration()
    let box = Box(config)
    return retain(box)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_output_url")
public func setRecordingOutputURL(_ config: OpaquePointer, _ path: UnsafePointer<CChar>) {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    let pathString = String(cString: path)
    box.value.outputURL = URL(fileURLWithPath: pathString)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_video_codec")
public func setRecordingOutputVideoCodec(_ config: OpaquePointer, _ codec: Int32) {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    switch codec {
    case 0: box.value.videoCodecType = .h264
    case 1: box.value.videoCodecType = .hevc
    default: break
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_average_bitrate")
public func setRecordingOutputAverageBitrate(_ config: OpaquePointer, _ bitrate: Int64) {
    // Note: bitrate control may be done through outputURL with codec settings
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_retain")
public func retainRecordingOutputConfiguration(_ config: OpaquePointer) -> OpaquePointer {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    return retain(box)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_release")
public func releaseRecordingOutputConfiguration(_ config: OpaquePointer) {
    release(config)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_create")
public func createRecordingOutput(_ config: OpaquePointer) -> OpaquePointer? {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    let delegate = RecordingDelegate()
    let output = SCRecordingOutput(configuration: box.value, delegate: delegate)
    return retain(output)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_retain")
public func retainRecordingOutput(_ output: OpaquePointer) -> OpaquePointer {
    let o: SCRecordingOutput = unretained(output)
    return retain(o)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_release")
public func releaseRecordingOutput(_ output: OpaquePointer) {
    release(output)
}

// MARK: - Screenshot Manager (macOS 14.0+)

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_image")
public func captureScreenshot(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let image = try await SCScreenshotManager.captureImage(
                contentFilter: filter,
                configuration: configuration
            )
            callback(retain(image), nil, userData)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0, userData) }
        }
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_sample_buffer")
public func captureScreenshotSampleBuffer(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let sampleBuffer = try await SCScreenshotManager.captureSampleBuffer(
                contentFilter: filter,
                configuration: configuration
            )
            let retained = Unmanaged.passRetained(sampleBuffer as AnyObject)
            callback(OpaquePointer(retained.toOpaque()), nil, userData)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0, userData) }
        }
    }
}

@_cdecl("cgimage_get_width")
public func getCGImageWidth(_ image: OpaquePointer) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return cgImage.width
}

@_cdecl("cgimage_get_height")
public func getCGImageHeight(_ image: OpaquePointer) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return cgImage.height
}

@_cdecl("cgimage_release")
public func releaseCGImage(_ image: OpaquePointer) {
    Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).release()
}

@_cdecl("cgimage_get_data")
public func getCGImageData(_ image: OpaquePointer, _ outPtr: UnsafeMutablePointer<UnsafeRawPointer?>, _ outLength: UnsafeMutablePointer<Int>) -> Bool {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    
    let width = cgImage.width
    let height = cgImage.height
    let bytesPerPixel = 4  // RGBA
    let bytesPerRow = width * bytesPerPixel
    let totalBytes = height * bytesPerRow
    
    // Create a bitmap context to draw the image
    let colorSpace = CGColorSpaceCreateDeviceRGB()
    let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue
    
    guard let context = CGContext(
        data: nil,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: bytesPerRow,
        space: colorSpace,
        bitmapInfo: bitmapInfo
    ) else {
        return false
    }
    
    // Draw the image into the context
    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))
    
    // Get the data
    guard let data = context.data else {
        return false
    }
    
    // Allocate memory for the data and copy it
    let buffer = UnsafeMutableRawPointer.allocate(byteCount: totalBytes, alignment: 1)
    buffer.copyMemory(from: data, byteCount: totalBytes)
    
    outPtr.pointee = UnsafeRawPointer(buffer)
    outLength.pointee = totalBytes
    
    return true
}

@_cdecl("cgimage_free_data")
public func freeCGImageData(_ ptr: UnsafeMutableRawPointer) {
    ptr.deallocate()
}
