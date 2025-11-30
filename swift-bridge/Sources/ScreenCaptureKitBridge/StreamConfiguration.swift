// Stream Configuration APIs - SCStreamConfiguration

import CoreGraphics
import CoreMedia
import Foundation
import ScreenCaptureKit

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
public func setStreamConfigurationMinimumFrameInterval(_ config: OpaquePointer, _ value: Int64, _ timescale: Int32, _ flags: UInt32, _ epoch: Int64) {
    let scConfig: SCStreamConfiguration = unretained(config)
    scConfig.minimumFrameInterval = CMTime(value: value, timescale: timescale, flags: CMTimeFlags(rawValue: flags), epoch: epoch)
}

@_cdecl("sc_stream_configuration_get_minimum_frame_interval")
public func getStreamConfigurationMinimumFrameInterval(_ config: OpaquePointer, _ value: UnsafeMutablePointer<Int64>, _ timescale: UnsafeMutablePointer<Int32>, _ flags: UnsafeMutablePointer<UInt32>, _ epoch: UnsafeMutablePointer<Int64>) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let cmTime = scConfig.minimumFrameInterval
    value.pointee = cmTime.value
    timescale.pointee = cmTime.timescale
    flags.pointee = cmTime.flags.rawValue
    epoch.pointee = cmTime.epoch
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

// Shadow display configuration
@_cdecl("sc_stream_configuration_set_ignores_shadow_display_configuration")
public func setStreamConfigurationIgnoresShadowDisplayConfiguration(_ config: OpaquePointer, _ ignores: Bool) {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        scConfig.ignoreShadowsDisplay = ignores
    }
}

@_cdecl("sc_stream_configuration_get_ignores_shadow_display_configuration")
public func getStreamConfigurationIgnoresShadowDisplayConfiguration(_ config: OpaquePointer) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return scConfig.ignoreShadowsDisplay
    }
    return false
}

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
public func setStreamConfigurationIgnoreGlobalClipboard(_: OpaquePointer, _: Bool) {}

@_cdecl("sc_stream_configuration_get_ignore_global_clipboard")
public func getStreamConfigurationIgnoreGlobalClipboard(_: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_capture_resolution_type")
public func setStreamConfigurationCaptureResolutionType(_ config: OpaquePointer, _ resolution: Int32) {
    if #available(macOS 14.0, *) {
        let scConfig: SCStreamConfiguration = unretained(config)
        switch resolution {
        case 0: scConfig.captureResolution = .automatic
        case 1: scConfig.captureResolution = .best
        case 2: scConfig.captureResolution = .nominal
        default: break
        }
    }
}

@_cdecl("sc_stream_configuration_get_capture_resolution_type")
public func getStreamConfigurationCaptureResolutionType(_ config: OpaquePointer) -> Int32 {
    if #available(macOS 14.0, *) {
        let scConfig: SCStreamConfiguration = unretained(config)
        switch scConfig.captureResolution {
        case .automatic: return 0
        case .best: return 1
        case .nominal: return 2
        @unknown default: return 0
        }
    }
    return 0
}

@_cdecl("sc_stream_configuration_set_color_matrix")
public func setStreamConfigurationColorMatrix(_ config: OpaquePointer, _ matrix: UnsafePointer<CChar>) {
    let scConfig: SCStreamConfiguration = unretained(config)
    let matrixStr = String(cString: matrix)
    scConfig.colorMatrix = matrixStr as CFString
}

@_cdecl("sc_stream_configuration_get_color_matrix")
public func getStreamConfigurationColorMatrix(_ config: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let scConfig: SCStreamConfiguration = unretained(config)
    let matrix = scConfig.colorMatrix as String
    return matrix.withCString { src in
        strlcpy(buffer, src, bufferSize)
        return true
    }
}

@_cdecl("sc_stream_configuration_set_increase_resolution_for_retina_displays")
public func setStreamConfigurationIncreaseResolutionForRetinaDisplays(_: OpaquePointer, _: Bool) {}

@_cdecl("sc_stream_configuration_get_increase_resolution_for_retina_displays")
public func getStreamConfigurationIncreaseResolutionForRetinaDisplays(_: OpaquePointer) -> Bool { false }

@_cdecl("sc_stream_configuration_set_ignore_fraction_of_screen")
public func setStreamConfigurationIgnoreFractionOfScreen(_: OpaquePointer, _: Double) {}

@_cdecl("sc_stream_configuration_get_ignore_fraction_of_screen")
public func getStreamConfigurationIgnoreFractionOfScreen(_: OpaquePointer) -> Double { 0.0 }

@_cdecl("sc_stream_configuration_set_ignores_shadows_single_window")
public func setStreamConfigurationIgnoresShadowsSingleWindow(_ config: OpaquePointer, _ ignoresShadows: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        cfg.ignoreShadowsSingleWindow = ignoresShadows
    }
}

@_cdecl("sc_stream_configuration_get_ignores_shadows_single_window")
public func getStreamConfigurationIgnoresShadowsSingleWindow(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return cfg.ignoreShadowsSingleWindow
    }
    return false
}

@_cdecl("sc_stream_configuration_set_includes_child_windows")
public func setStreamConfigurationIncludesChildWindows(_ config: OpaquePointer, _ includesChildWindows: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.2, *) {
        cfg.includeChildWindows = includesChildWindows
    }
}

@_cdecl("sc_stream_configuration_get_includes_child_windows")
public func getStreamConfigurationIncludesChildWindows(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.2, *) {
        return cfg.includeChildWindows
    }
    return false
}

@_cdecl("sc_stream_configuration_set_presenter_overlay_privacy_alert_setting")
public func setStreamConfigurationPresenterOverlayPrivacyAlertSetting(_ config: OpaquePointer, _ setting: Int) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        switch setting {
        case 0: cfg.presenterOverlayPrivacyAlertSetting = .system
        case 1: cfg.presenterOverlayPrivacyAlertSetting = .never
        case 2: cfg.presenterOverlayPrivacyAlertSetting = .always
        default: break
        }
    }
}

@_cdecl("sc_stream_configuration_get_presenter_overlay_privacy_alert_setting")
public func getStreamConfigurationPresenterOverlayPrivacyAlertSetting(_ config: OpaquePointer) -> Int {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        switch cfg.presenterOverlayPrivacyAlertSetting {
        case .system: return 0
        case .never: return 1
        case .always: return 2
        @unknown default: return 0
        }
    }
    return 0
}

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

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_set_captures_microphone")
    public func setStreamConfigurationCapturesMicrophone(_ config: OpaquePointer, _ value: Bool) {
        let cfg: SCStreamConfiguration = unretained(config)
        if #available(macOS 15.0, *) {
            cfg.captureMicrophone = value
        }
    }
#else
    @_cdecl("sc_stream_configuration_set_captures_microphone")
    public func setStreamConfigurationCapturesMicrophone(_: OpaquePointer, _: Bool) {}
#endif

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_get_captures_microphone")
    public func getStreamConfigurationCapturesMicrophone(_ config: OpaquePointer) -> Bool {
        let cfg: SCStreamConfiguration = unretained(config)
        if #available(macOS 15.0, *) {
            return cfg.captureMicrophone
        }
        return false
    }
#else
    @_cdecl("sc_stream_configuration_get_captures_microphone")
    public func getStreamConfigurationCapturesMicrophone(_: OpaquePointer) -> Bool {
        false
    }
#endif

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

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_set_microphone_capture_device_id")
    public func setStreamConfigurationMicrophoneCaptureDeviceId(_ config: OpaquePointer, _ deviceId: UnsafePointer<CChar>?) {
        let cfg: SCStreamConfiguration = unretained(config)
        if #available(macOS 15.0, *) {
            if let deviceId {
                cfg.microphoneCaptureDeviceID = String(cString: deviceId)
            } else {
                cfg.microphoneCaptureDeviceID = nil
            }
        }
    }
#else
    @_cdecl("sc_stream_configuration_set_microphone_capture_device_id")
    public func setStreamConfigurationMicrophoneCaptureDeviceId(_: OpaquePointer, _: UnsafePointer<CChar>?) {}
#endif

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
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
#else
    @_cdecl("sc_stream_configuration_get_microphone_capture_device_id")
    public func getStreamConfigurationMicrophoneCaptureDeviceId(_: OpaquePointer, _: UnsafeMutablePointer<CChar>, _: Int) -> Bool {
        false
    }
#endif

@_cdecl("sc_stream_configuration_set_stream_name")
public func setStreamConfigurationStreamName(_ config: OpaquePointer, _ name: UnsafePointer<CChar>?) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        if let name {
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

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
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
#else
    @_cdecl("sc_stream_configuration_set_capture_dynamic_range")
    public func setStreamConfigurationCaptureDynamicRange(_: OpaquePointer, _: Int32) {
        // Not available on macOS < 15.0
    }
#endif

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
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
#else
    @_cdecl("sc_stream_configuration_get_capture_dynamic_range")
    public func getStreamConfigurationCaptureDynamicRange(_: OpaquePointer) -> Int32 {
        0 // Not available on macOS < 15.0
    }
#endif

// MARK: - macOS 15.0+ Properties

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_set_shows_mouse_clicks")
    public func setStreamConfigurationShowsMouseClicks(_ config: OpaquePointer, _ value: Bool) {
        let cfg: SCStreamConfiguration = unretained(config)
        if #available(macOS 15.0, *) {
            cfg.showMouseClicks = value
        }
    }
#else
    @_cdecl("sc_stream_configuration_set_shows_mouse_clicks")
    public func setStreamConfigurationShowsMouseClicks(_: OpaquePointer, _: Bool) {}
#endif

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_get_shows_mouse_clicks")
    public func getStreamConfigurationShowsMouseClicks(_ config: OpaquePointer) -> Bool {
        let cfg: SCStreamConfiguration = unretained(config)
        if #available(macOS 15.0, *) {
            return cfg.showMouseClicks
        }
        return false
    }
#else
    @_cdecl("sc_stream_configuration_get_shows_mouse_clicks")
    public func getStreamConfigurationShowsMouseClicks(_: OpaquePointer) -> Bool {
        false
    }
#endif

// MARK: - macOS 14.0+ Properties

@_cdecl("sc_stream_configuration_set_ignores_shadows_display")
public func setStreamConfigurationIgnoresShadowsDisplay(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        cfg.ignoreShadowsDisplay = value
    }
}

@_cdecl("sc_stream_configuration_get_ignores_shadows_display")
public func getStreamConfigurationIgnoresShadowsDisplay(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return cfg.ignoreShadowsDisplay
    }
    return false
}

@_cdecl("sc_stream_configuration_set_ignore_global_clip_display")
public func setStreamConfigurationIgnoreGlobalClipDisplay(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        cfg.ignoreGlobalClipDisplay = value
    }
}

@_cdecl("sc_stream_configuration_get_ignore_global_clip_display")
public func getStreamConfigurationIgnoreGlobalClipDisplay(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return cfg.ignoreGlobalClipDisplay
    }
    return false
}

@_cdecl("sc_stream_configuration_set_ignore_global_clip_single_window")
public func setStreamConfigurationIgnoreGlobalClipSingleWindow(_ config: OpaquePointer, _ value: Bool) {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        cfg.ignoreGlobalClipSingleWindow = value
    }
}

@_cdecl("sc_stream_configuration_get_ignore_global_clip_single_window")
public func getStreamConfigurationIgnoreGlobalClipSingleWindow(_ config: OpaquePointer) -> Bool {
    let cfg: SCStreamConfiguration = unretained(config)
    if #available(macOS 14.0, *) {
        return cfg.ignoreGlobalClipSingleWindow
    }
    return false
}

// MARK: - Preset-based configuration (macOS 15.0+)

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_stream_configuration_create_with_preset")
    public func createStreamConfigurationWithPreset(_ preset: Int32) -> OpaquePointer? {
        if #available(macOS 15.0, *) {
            let scPreset: SCStreamConfiguration.Preset = switch preset {
            case 0:
                .captureHDRStreamLocalDisplay
            case 1:
                .captureHDRStreamCanonicalDisplay
            case 2:
                .captureHDRScreenshotLocalDisplay
            case 3:
                .captureHDRScreenshotCanonicalDisplay
            default:
                .captureHDRStreamLocalDisplay
            }
            return retain(SCStreamConfiguration(preset: scPreset))
        }
        return retain(SCStreamConfiguration())
    }
#else
    @_cdecl("sc_stream_configuration_create_with_preset")
    public func createStreamConfigurationWithPreset(_: Int32) -> OpaquePointer? {
        retain(SCStreamConfiguration())
    }
#endif
