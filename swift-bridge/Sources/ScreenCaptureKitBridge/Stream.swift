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
        #if compiler(>=6.0)
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
        #if compiler(>=6.0)
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
        #if compiler(>=6.0)
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
