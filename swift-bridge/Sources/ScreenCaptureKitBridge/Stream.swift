// Stream Control APIs - SCContentFilter, SCStream

import CoreGraphics
import CoreMedia
import Foundation
import ScreenCaptureKit

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
    if let windows {
        for i in 0 ..< windowsCount {
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
    if let windows {
        for i in 0 ..< windowsCount {
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
    if let apps {
        for i in 0 ..< appsCount {
            let app: SCRunningApplication = unretained(apps[i])
            includedApps.append(app)
        }
    }
    var exceptedWindows: [SCWindow] = []
    if let windows {
        for i in 0 ..< windowsCount {
            let window: SCWindow = unretained(windows[i])
            exceptedWindows.append(window)
        }
    }
    let filter = SCContentFilter(display: scDisplay, including: includedApps, exceptingWindows: exceptedWindows)
    return retain(filter)
}

@_cdecl("sc_content_filter_create_with_display_excluding_applications_excepting_windows")
public func createContentFilterWithDisplayExcludingApplicationsExceptingWindows(
    _ display: OpaquePointer,
    _ apps: UnsafePointer<OpaquePointer>?,
    _ appsCount: Int,
    _ windows: UnsafePointer<OpaquePointer>?,
    _ windowsCount: Int
) -> OpaquePointer {
    let scDisplay: SCDisplay = unretained(display)
    var excludedApps: [SCRunningApplication] = []
    if let apps {
        for i in 0 ..< appsCount {
            let app: SCRunningApplication = unretained(apps[i])
            excludedApps.append(app)
        }
    }
    var exceptedWindows: [SCWindow] = []
    if let windows {
        for i in 0 ..< windowsCount {
            let window: SCWindow = unretained(windows[i])
            exceptedWindows.append(window)
        }
    }
    let filter = SCContentFilter(display: scDisplay, excludingApplications: excludedApps, exceptingWindows: exceptedWindows)
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
public func setContentFilterContentRect(_: OpaquePointer, _: Double, _: Double, _: Double, _: Double) {}

@_cdecl("sc_content_filter_get_content_rect")
public func getContentFilterContentRect(
    _ filter: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.0, *) {
        let rect = f.contentRect
        x.pointee = rect.origin.x
        y.pointee = rect.origin.y
        width.pointee = rect.size.width
        height.pointee = rect.size.height
    } else {
        x.pointee = 0.0
        y.pointee = 0.0
        width.pointee = 0.0
        height.pointee = 0.0
    }
}

@_cdecl("sc_content_filter_get_style")
public func getContentFilterStyle(_ filter: OpaquePointer) -> Int32 {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.0, *) {
        switch f.style {
        case .none:
            return 0
        case .window:
            return 1
        case .display:
            return 2
        case .application:
            return 3
        @unknown default:
            return 0
        }
    }
    return 0
}

@_cdecl("sc_content_filter_get_stream_type")
public func getContentFilterStreamType(_ filter: OpaquePointer) -> Int32 {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.0, *) {
        switch f.streamType {
        case .window:
            return 0
        case .display:
            return 1
        @unknown default:
            return -1
        }
    }
    return -1
}

@_cdecl("sc_content_filter_get_point_pixel_scale")
public func getContentFilterPointPixelScale(_ filter: OpaquePointer) -> Float {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.0, *) {
        return f.pointPixelScale
    }
    return 1.0
}

// macOS 14.2+ - includeMenuBar property
@_cdecl("sc_content_filter_set_include_menu_bar")
public func setContentFilterIncludeMenuBar(_ filter: OpaquePointer, _ include: Bool) {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.2, *) {
        f.includeMenuBar = include
    }
}

@_cdecl("sc_content_filter_get_include_menu_bar")
public func getContentFilterIncludeMenuBar(_ filter: OpaquePointer) -> Bool {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.2, *) {
        return f.includeMenuBar
    }
    return false
}

// macOS 15.2+ - readonly arrays
#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    @_cdecl("sc_content_filter_get_included_displays_count")
    public func getContentFilterIncludedDisplaysCount(_ filter: OpaquePointer) -> Int {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            return f.includedDisplays.count
        }
        return 0
    }

    @_cdecl("sc_content_filter_get_included_display_at")
    public func getContentFilterIncludedDisplayAt(_ filter: OpaquePointer, _ index: Int) -> OpaquePointer? {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            guard index >= 0, index < f.includedDisplays.count else { return nil }
            return retain(f.includedDisplays[index])
        }
        return nil
    }

    @_cdecl("sc_content_filter_get_included_windows_count")
    public func getContentFilterIncludedWindowsCount(_ filter: OpaquePointer) -> Int {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            return f.includedWindows.count
        }
        return 0
    }

    @_cdecl("sc_content_filter_get_included_window_at")
    public func getContentFilterIncludedWindowAt(_ filter: OpaquePointer, _ index: Int) -> OpaquePointer? {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            guard index >= 0, index < f.includedWindows.count else { return nil }
            return retain(f.includedWindows[index])
        }
        return nil
    }

    @_cdecl("sc_content_filter_get_included_applications_count")
    public func getContentFilterIncludedApplicationsCount(_ filter: OpaquePointer) -> Int {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            return f.includedApplications.count
        }
        return 0
    }

    @_cdecl("sc_content_filter_get_included_application_at")
    public func getContentFilterIncludedApplicationAt(_ filter: OpaquePointer, _ index: Int) -> OpaquePointer? {
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 15.2, *) {
            guard index >= 0, index < f.includedApplications.count else { return nil }
            return retain(f.includedApplications[index])
        }
        return nil
    }
#else
    @_cdecl("sc_content_filter_get_included_displays_count")
    public func getContentFilterIncludedDisplaysCount(_: OpaquePointer) -> Int { 0 }

    @_cdecl("sc_content_filter_get_included_display_at")
    public func getContentFilterIncludedDisplayAt(_: OpaquePointer, _: Int) -> OpaquePointer? { nil }

    @_cdecl("sc_content_filter_get_included_windows_count")
    public func getContentFilterIncludedWindowsCount(_: OpaquePointer) -> Int { 0 }

    @_cdecl("sc_content_filter_get_included_window_at")
    public func getContentFilterIncludedWindowAt(_: OpaquePointer, _: Int) -> OpaquePointer? { nil }

    @_cdecl("sc_content_filter_get_included_applications_count")
    public func getContentFilterIncludedApplicationsCount(_: OpaquePointer) -> Int { 0 }

    @_cdecl("sc_content_filter_get_included_application_at")
    public func getContentFilterIncludedApplicationAt(_: OpaquePointer, _: Int) -> OpaquePointer? { nil }
#endif

// MARK: - Stream: SCStream Delegates and Handlers

private class StreamDelegateWrapper: NSObject, SCStreamDelegate {
    let errorCallback: @convention(c) (OpaquePointer, Int32, UnsafePointer<CChar>) -> Void
    let streamPtr: OpaquePointer
    var activeCallback: (@convention(c) (OpaquePointer) -> Void)?
    var inactiveCallback: (@convention(c) (OpaquePointer) -> Void)?

    init(streamPtr: OpaquePointer, errorCallback: @escaping @convention(c) (OpaquePointer, Int32, UnsafePointer<CChar>) -> Void) {
        self.streamPtr = streamPtr
        self.errorCallback = errorCallback
    }

    func stream(_: SCStream, didStopWithError error: Error) {
        let errorCode = extractStreamErrorCode(error)
        let errorMsg = error.localizedDescription
        errorMsg.withCString { errorCallback(streamPtr, errorCode, $0) }
    }

    #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
        @available(macOS 15.2, *)
        func streamDidBecomeActive(_: SCStream) {
            activeCallback?(streamPtr)
        }

        @available(macOS 15.2, *)
        func streamDidBecomeInactive(_: SCStream) {
            inactiveCallback?(streamPtr)
        }
    #endif
}

private class StreamOutputHandler: NSObject, SCStreamOutput {
    let sampleBufferCallback: @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void
    let streamPtr: OpaquePointer

    init(streamPtr: OpaquePointer, sampleBufferCallback: @escaping @convention(c) (OpaquePointer, OpaquePointer, Int32) -> Void) {
        self.streamPtr = streamPtr
        self.sampleBufferCallback = sampleBufferCallback
    }

    func stream(_: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        let outputType: Int32 = if type == .screen {
            0
        } else if #available(macOS 15.0, *), type == .microphone {
            2
        } else {
            1 // audio
        }
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
        "\(UInt(bitPattern: stream))_\(type)"
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
    _ errorCallback: @escaping @convention(c) (OpaquePointer, Int32, UnsafePointer<CChar>) -> Void
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
        #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
            if #available(macOS 15.0, *) {
                outputType = .microphone
            } else {
                outputType = .audio
            }
        #else
            outputType = .audio
        #endif
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
        #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
            if #available(macOS 15.0, *) {
                outputType = .microphone
            } else {
                outputType = .audio
            }
        #else
            outputType = .audio
        #endif
    } else {
        outputType = .audio
    }

    let queue: DispatchQueue = if let queuePtr = dispatchQueue {
        unretained(queuePtr)
    } else {
        DispatchQueue(label: "com.screencapturekit.output", qos: .userInteractive)
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
        #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
            if #available(macOS 15.0, *) {
                outputType = .microphone
            } else {
                outputType = .audio
            }
        #else
            outputType = .audio
        #endif
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

// MARK: - Stream Lifecycle

/// Starts capturing from the stream
/// - Parameters:
///   - stream: The stream to start
///   - context: Opaque context pointer passed back to callback
///   - callback: Called with context, success/failure and optional error message
@_cdecl("sc_stream_start_capture")
public func startStreamCapture(
    _ stream: OpaquePointer,
    _ context: UnsafeMutableRawPointer?,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.startCapture()
            callback(context, true, nil)
        } catch {
            let bridgeError = SCBridgeError.streamError(error.localizedDescription)
            bridgeError.description.withCString { callback(context, false, $0) }
        }
    }
}

/// Stops capturing from the stream
/// - Parameters:
///   - stream: The stream to stop
///   - context: Opaque context pointer passed back to callback
///   - callback: Called with context, success/failure and optional error message
@_cdecl("sc_stream_stop_capture")
public func stopStreamCapture(
    _ stream: OpaquePointer,
    _ context: UnsafeMutableRawPointer?,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.stopCapture()
            callback(context, true, nil)
        } catch {
            let bridgeError = SCBridgeError.streamError(error.localizedDescription)
            bridgeError.description.withCString { callback(context, false, $0) }
        }
    }
}

/// Updates the content filter for the stream
/// - Parameters:
///   - stream: The stream to update
///   - filter: The new content filter
///   - context: Opaque context pointer passed back to callback
///   - callback: Called with context, success/failure and optional error message
@_cdecl("sc_stream_update_content_filter")
public func updateStreamContentFilter(
    _ stream: OpaquePointer,
    _ filter: OpaquePointer,
    _ context: UnsafeMutableRawPointer?,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    let scFilter: SCContentFilter = unretained(filter)
    Task {
        do {
            try await scStream.updateContentFilter(scFilter)
            callback(context, true, nil)
        } catch {
            let bridgeError = SCBridgeError.streamError(error.localizedDescription)
            bridgeError.description.withCString { callback(context, false, $0) }
        }
    }
}

/// Updates the configuration for the stream
/// - Parameters:
///   - stream: The stream to update
///   - config: The new configuration
///   - context: Opaque context pointer passed back to callback
///   - callback: Called with context, success/failure and optional error message
@_cdecl("sc_stream_update_configuration")
public func updateStreamConfiguration(
    _ stream: OpaquePointer,
    _ config: OpaquePointer,
    _ context: UnsafeMutableRawPointer?,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void
) {
    if #available(macOS 14.0, *) {
        let scStream: SCStream = unretained(stream)
        let scConfig: SCStreamConfiguration = unretained(config)
        Task {
            do {
                try await scStream.updateConfiguration(scConfig)
                callback(context, true, nil)
            } catch {
                let bridgeError = SCBridgeError.configurationError(error.localizedDescription)
                bridgeError.description.withCString { callback(context, false, $0) }
            }
        }
    } else {
        let bridgeError = SCBridgeError.configurationError("updateConfiguration requires macOS 14.0 or later")
        bridgeError.description.withCString { callback(context, false, $0) }
    }
}

// MARK: - Stream Properties

/// Get the synchronization clock for the stream (macOS 13.0+)
@_cdecl("sc_stream_get_synchronization_clock")
public func getStreamSynchronizationClock(_ stream: OpaquePointer) -> OpaquePointer? {
    if #available(macOS 13.0, *) {
        let s: SCStream = unretained(stream)
        if let clock = s.synchronizationClock {
            // CMClock is a CoreFoundation type, retain and return it
            return OpaquePointer(Unmanaged.passRetained(clock as AnyObject).toOpaque())
        }
    }
    return nil
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

// MARK: - Recording Output (macOS 15.0+)

#if SCREENCAPTUREKIT_HAS_MACOS15_SDK
    // Full implementation for macOS 15 SDK

    @available(macOS 15.0, *)
    private func addRecordingOutputImpl(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer
    ) throws {
        let s: SCStream = unretained(stream)
        let rec: SCRecordingOutput = unretained(recordingOutput)
        try s.addRecordingOutput(rec)
    }

    @available(macOS 15.0, *)
    private func removeRecordingOutputImpl(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer
    ) throws {
        let s: SCStream = unretained(stream)
        let rec: SCRecordingOutput = unretained(recordingOutput)
        try s.removeRecordingOutput(rec)
    }

    @_cdecl("sc_stream_add_recording_output")
    public func addRecordingOutput(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        if #available(macOS 15.0, *) {
            do {
                try addRecordingOutputImpl(stream, recordingOutput)
                callback(context, true, nil)
            } catch {
                error.localizedDescription.withCString { callback(context, false, $0) }
            }
        } else {
            let bridgeError = SCBridgeError.configurationError("addRecordingOutput requires macOS 15.0 or later")
            bridgeError.description.withCString { callback(context, false, $0) }
        }
    }

    @_cdecl("sc_stream_remove_recording_output")
    public func removeRecordingOutput(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        if #available(macOS 15.0, *) {
            do {
                try removeRecordingOutputImpl(stream, recordingOutput)
                callback(context, true, nil)
            } catch {
                error.localizedDescription.withCString { callback(context, false, $0) }
            }
        } else {
            let bridgeError = SCBridgeError.configurationError("removeRecordingOutput requires macOS 15.0 or later")
            bridgeError.description.withCString { callback(context, false, $0) }
        }
    }

#else
    // Stub implementation for older SDKs (macOS < 15 SDK)

    @_cdecl("sc_stream_add_recording_output")
    public func addRecordingOutput(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.configurationError("addRecordingOutput requires macOS 15.0 SDK or later")
        bridgeError.description.withCString { callback(context, false, $0) }
    }

    @_cdecl("sc_stream_remove_recording_output")
    public func removeRecordingOutput(
        _ stream: OpaquePointer,
        _ recordingOutput: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.configurationError("removeRecordingOutput requires macOS 15.0 SDK or later")
        bridgeError.description.withCString { callback(context, false, $0) }
    }

#endif
