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

@_cdecl("sc_content_filter_get_content_rect")
public func getContentFilterContentRect(
    _ filter: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 14.0, *) {
            let rect = f.contentRect
            x.pointee = rect.origin.x
            y.pointee = rect.origin.y
            width.pointee = rect.size.width
            height.pointee = rect.size.height
            return
        }
    #endif
    x.pointee = 0.0
    y.pointee = 0.0
    width.pointee = 0.0
    height.pointee = 0.0
}

@_cdecl("sc_content_filter_get_style")
public func getContentFilterStyle(_ filter: OpaquePointer) -> Int32 {
    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
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
    #endif
    return 0
}

@_cdecl("sc_content_filter_get_stream_type")
public func getContentFilterStreamType(_ filter: OpaquePointer) -> Int32 {
    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
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
    #endif
    return -1
}

@_cdecl("sc_content_filter_get_point_pixel_scale")
public func getContentFilterPointPixelScale(_ filter: OpaquePointer) -> Float {
    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
        let f: SCContentFilter = unretained(filter)
        if #available(macOS 14.0, *) {
            return f.pointPixelScale
        }
    #endif
    return 1.0
}

// macOS 14.2+ - includeMenuBar property
#if SCREENCAPTUREKIT_HAS_MACOS14_2_SDK
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
#else
    @_cdecl("sc_content_filter_set_include_menu_bar")
    public func setContentFilterIncludeMenuBar(_: OpaquePointer, _: Bool) {}

    @_cdecl("sc_content_filter_get_include_menu_bar")
    public func getContentFilterIncludeMenuBar(_: OpaquePointer) -> Bool { false }
#endif

// macOS 15.2+ - readonly arrays
#if SCREENCAPTUREKIT_HAS_MACOS15_2_SDK
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

// Delegate lifecycle event codes. Mirrored by `delegate_event_callback` in
// src/stream/sc_stream.rs; the two must be changed together.
private let kStreamEventDidBecomeActive: Int32 = 0
private let kStreamEventDidBecomeInactive: Int32 = 1
private let kStreamEventVideoEffectDidStart: Int32 = 2
private let kStreamEventVideoEffectDidStop: Int32 = 3

private class StreamDelegateWrapper: NSObject, SCStreamDelegate {
    let contextPtr: UnsafeMutableRawPointer
    let errorCallback: @convention(c) (UnsafeMutableRawPointer, Int32, UnsafePointer<CChar>) -> Void
    let contextRelease: @convention(c) (UnsafeMutableRawPointer) -> Void

    private let eventLock = NSLock()
    private var eventCallback: (@convention(c) (UnsafeMutableRawPointer, Int32) -> Void)?

    init(
        contextPtr: UnsafeMutableRawPointer,
        errorCallback: @escaping @convention(c) (UnsafeMutableRawPointer, Int32, UnsafePointer<CChar>) -> Void,
        contextRetain: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void,
        contextRelease: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void
    ) {
        self.contextPtr = contextPtr
        self.errorCallback = errorCallback
        self.contextRelease = contextRelease
        // Take a +1 on the Rust StreamContext for the lifetime of this object so
        // an in-flight delegate callback can never observe a freed context.
        contextRetain(contextPtr)
    }

    deinit {
        contextRelease(contextPtr)
    }

    func setEventCallback(_ callback: @escaping @convention(c) (UnsafeMutableRawPointer, Int32) -> Void) {
        eventLock.lock()
        defer { eventLock.unlock() }
        eventCallback = callback
    }

    private func emit(_ event: Int32) {
        eventLock.lock()
        let callback = eventCallback
        eventLock.unlock()
        callback?(contextPtr, event)
    }

    func stream(_: SCStream, didStopWithError error: Error) {
        let errorCode = extractStreamErrorCode(error)
        let errorMsg = error.localizedDescription
        errorMsg.withCString { errorCallback(contextPtr, errorCode, $0) }
    }

    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
        @available(macOS 14.0, *)
        func outputVideoEffectDidStart(for _: SCStream) {
            emit(kStreamEventVideoEffectDidStart)
        }

        @available(macOS 14.0, *)
        func outputVideoEffectDidStop(for _: SCStream) {
            emit(kStreamEventVideoEffectDidStop)
        }
    #endif

    #if SCREENCAPTUREKIT_HAS_MACOS15_2_SDK
        @available(macOS 15.2, *)
        func streamDidBecomeActive(_: SCStream) {
            emit(kStreamEventDidBecomeActive)
        }

        @available(macOS 15.2, *)
        func streamDidBecomeInactive(_: SCStream) {
            emit(kStreamEventDidBecomeInactive)
        }
    #endif
}

private class StreamOutputHandler: NSObject, SCStreamOutput {
    let contextPtr: UnsafeMutableRawPointer
    let sampleBufferCallback: @convention(c) (UnsafeMutableRawPointer, OpaquePointer, Int32) -> Void
    let contextRelease: @convention(c) (UnsafeMutableRawPointer) -> Void

    init(
        contextPtr: UnsafeMutableRawPointer,
        sampleBufferCallback: @escaping @convention(c) (UnsafeMutableRawPointer, OpaquePointer, Int32) -> Void,
        contextRetain: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void,
        contextRelease: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void
    ) {
        self.contextPtr = contextPtr
        self.sampleBufferCallback = sampleBufferCallback
        self.contextRelease = contextRelease
        // Take a +1 on the Rust StreamContext for the lifetime of this object so
        // an in-flight sample callback can never observe a freed context.
        contextRetain(contextPtr)
    }

    deinit {
        contextRelease(contextPtr)
    }

    func stream(_: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        // Use rawValue comparison to avoid SDK availability issues
        // .screen = 0, .audio = 1, .microphone = 2 (macOS 15+)
        let outputType: Int32 = if type == .screen {
            0
        } else if type.rawValue == 2 { // microphone (macOS 15+)
            2
        } else {
            1 // audio
        }
        // IMPORTANT: passRetained() is used here to retain the CMSampleBuffer for Rust
        // The Rust side will release it when CMSampleBuffer is dropped
        sampleBufferCallback(contextPtr, OpaquePointer(Unmanaged.passRetained(sampleBuffer as AnyObject).toOpaque()), outputType)
    }
}

// Per-stream storage for its delegate and output handlers
private class StreamState {
    let delegate: StreamDelegateWrapper
    let outputHandler: StreamOutputHandler
    /// Number of live Rust-side `SCStream` handles (the original plus every
    /// clone) referencing this state. Mutated only while holding
    /// `streamStatesLock`, so it needs no lock of its own.
    var refCount: Int = 1
    private var outputTypes: Set<Int32> = []
    private let lock = NSLock()

    init(delegate: StreamDelegateWrapper, outputHandler: StreamOutputHandler) {
        self.delegate = delegate
        self.outputHandler = outputHandler
    }

    func hasOutput(_ type: Int32) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return outputTypes.contains(type)
    }

    func addOutput(_ type: Int32) {
        lock.lock()
        defer { lock.unlock() }
        outputTypes.insert(type)
    }

    func removeOutput(_ type: Int32) {
        lock.lock()
        defer { lock.unlock() }
        outputTypes.remove(type)
    }
}

// Map from SCStream pointer → StreamState (kept alive while stream exists)
private var streamStates: [ObjectIdentifier: StreamState] = [:]
private let streamStatesLock = NSLock()

private func getStreamState(for stream: SCStream) -> StreamState? {
    streamStatesLock.lock()
    defer { streamStatesLock.unlock() }
    return streamStates[ObjectIdentifier(stream)]
}

private func setStreamState(_ state: StreamState, for stream: SCStream) {
    streamStatesLock.lock()
    defer { streamStatesLock.unlock() }
    streamStates[ObjectIdentifier(stream)] = state
}

/// Mirror a Rust-side `SCStream` clone: one more handle now refers to this
/// state, so it must outlive that handle too.
private func retainStreamState(for stream: SCStream) {
    streamStatesLock.lock()
    defer { streamStatesLock.unlock() }
    streamStates[ObjectIdentifier(stream)]?.refCount += 1
}

/// Drop one Rust-side handle's claim on the state, discarding it only when the
/// last handle goes away. Dropping it on the first release would detach the
/// delegate and output handler while surviving clones were still capturing.
private func releaseStreamState(for stream: SCStream) {
    streamStatesLock.lock()
    defer { streamStatesLock.unlock() }
    let key = ObjectIdentifier(stream)
    guard let state = streamStates[key] else { return }
    state.refCount -= 1
    if state.refCount <= 0 {
        streamStates.removeValue(forKey: key)
    }
}

// MARK: - Stream: SCStream Control

@_cdecl("sc_stream_create")
public func createStream(
    _ filter: OpaquePointer,
    _ config: OpaquePointer,
    _ context: UnsafeMutableRawPointer,
    _ errorCallback: @escaping @convention(c) (UnsafeMutableRawPointer, Int32, UnsafePointer<CChar>) -> Void,
    _ sampleCallback: @escaping @convention(c) (UnsafeMutableRawPointer, OpaquePointer, Int32) -> Void,
    _ contextRetain: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void,
    _ contextRelease: @escaping @convention(c) (UnsafeMutableRawPointer) -> Void
) -> OpaquePointer? {
    let scFilter: SCContentFilter = unretained(filter)
    let scConfig: SCStreamConfiguration = unretained(config)

    let delegate = StreamDelegateWrapper(
        contextPtr: context,
        errorCallback: errorCallback,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )
    let outputHandler = StreamOutputHandler(
        contextPtr: context,
        sampleBufferCallback: sampleCallback,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )

    let stream = SCStream(filter: scFilter, configuration: scConfig, delegate: delegate)
    let state = StreamState(delegate: delegate, outputHandler: outputHandler)
    setStreamState(state, for: stream)

    let actualStreamPtr = retain(stream)
    return actualStreamPtr
}

/// Register the trampoline that forwards `SCStreamDelegate` lifecycle events
/// (become active/inactive, video effect start/stop) to Rust.
///
/// The trampoline is called with the same opaque context pointer that was
/// passed to `sc_stream_create` and an event code (see the `kStreamEvent*`
/// constants above). Returns `false` when the stream has no registered state.
@_cdecl("sc_stream_set_delegate_event_callback")
public func setStreamDelegateEventCallback(
    _ stream: OpaquePointer,
    _ callback: @escaping @convention(c) (UnsafeMutableRawPointer, Int32) -> Void
) -> Bool {
    let scStream: SCStream = unretained(stream)
    guard let state = getStreamState(for: scStream) else { return false }
    state.delegate.setEventCallback(callback)
    return true
}

/// Map the Rust-side output type discriminant onto `SCStreamOutputType`.
/// 0 = screen, 1 = audio, 2 = microphone. Unsupported values return `nil`
/// instead of silently registering a different output type.
private func nativeOutputType(_ type: Int32) -> SCStreamOutputType? {
    if type == 0 {
        return .screen
    }
    if type == 1 {
        return .audio
    }
    if type == 2 {
        #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
            if #available(macOS 15.0, *) {
                return .microphone
            }
        #endif
    }
    return nil
}

@_cdecl("sc_stream_add_stream_output")
public func addStreamOutput(
    _ stream: OpaquePointer,
    _ type: Int32
) -> Bool {
    addStreamOutputWithQueue(stream, type, nil)
}

@_cdecl("sc_stream_add_stream_output_with_queue")
public func addStreamOutputWithQueue(
    _ stream: OpaquePointer,
    _ type: Int32,
    _ dispatchQueue: OpaquePointer?
) -> Bool {
    let scStream: SCStream = unretained(stream)
    guard let state = getStreamState(for: scStream) else { return false }
    guard let outputType = nativeOutputType(type) else { return false }

    // One shared `StreamOutputHandler` object serves every Rust handler, so
    // ScreenCaptureKit only ever needs one native registration per output
    // type. A repeat call is a no-op that reuses the queue established by the
    // first registration; the Rust side refuses conflicting explicit queues
    // before it gets here.
    if state.hasOutput(type) {
        return true
    }

    // INTENTIONAL DEVIATION FROM SCStream.addStreamOutput's `nil`→main-queue
    // contract: this bridge always uses a dedicated queue when the caller
    // didn't supply one. Apple's API treats `nil` as "deliver on the main
    // queue", which only works when the host process runs a Cocoa runloop
    // (RunLoop.main / app `NSRunLoop`). Pure-Rust hosts typically don't, in
    // which case main-queue dispatch never fires and samples are silently
    // dropped. A dedicated user-interactive queue gives correct delivery
    // out of the box; callers who need main-queue affinity (e.g. AppKit/
    // UIKit access) should pass their own DispatchQueue or hop to the main
    // queue from inside their handler.
    let queue: DispatchQueue = if let queuePtr = dispatchQueue {
        unretained(queuePtr)
    } else {
        DispatchQueue(label: "com.screencapturekit.output.\(type)", qos: .userInteractive)
    }

    do {
        try scStream.addStreamOutput(state.outputHandler, type: outputType, sampleHandlerQueue: queue)
        state.addOutput(type)
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
    guard let state = getStreamState(for: scStream) else { return false }
    guard state.hasOutput(type) else { return false }
    guard let outputType = nativeOutputType(type) else { return false }

    do {
        try scStream.removeStreamOutput(state.outputHandler, type: outputType)
        state.removeOutput(type)
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
    #if SCREENCAPTUREKIT_HAS_MACOS14_SDK
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
            return
        }
    #endif
    let bridgeError = SCBridgeError.configurationError("updateConfiguration requires macOS 14.0 or later")
    bridgeError.description.withCString { callback(context, false, $0) }
}

// MARK: - Stream Properties

/// Get the synchronization clock for the stream (macOS 13.0+)
@_cdecl("sc_stream_get_synchronization_clock")
public func getStreamSynchronizationClock(_ stream: OpaquePointer) -> OpaquePointer? {
    if #available(macOS 13.0, *) {
        let s: SCStream = unretained(stream)
        if let clock = s.synchronizationClock {
            // Return a +0 (unretained) reference; the Rust side wraps it via
            // CMClock::from_raw, which retains. The clock is owned by the live
            // stream for the duration of this synchronous call.
            return OpaquePointer(Unmanaged.passUnretained(clock as AnyObject).toOpaque())
        }
    }
    return nil
}

@_cdecl("sc_stream_retain")
public func retainStream(_ stream: OpaquePointer) -> OpaquePointer {
    let s: SCStream = unretained(stream)
    retainStreamState(for: s)
    return retain(s)
}

@_cdecl("sc_stream_release")
public func releaseStream(_ stream: OpaquePointer) {
    let s: SCStream = unretained(stream)
    releaseStreamState(for: s)
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
                let rec: SCRecordingOutput = unretained(recordingOutput)
                afterRecordingOutputTerminal(rec) {
                    callback(context, true, nil)
                }
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
        _: OpaquePointer,
        _: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.configurationError("addRecordingOutput requires macOS 15.0 SDK or later")
        bridgeError.description.withCString { callback(context, false, $0) }
    }

    @_cdecl("sc_stream_remove_recording_output")
    public func removeRecordingOutput(
        _: OpaquePointer,
        _: OpaquePointer,
        _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        _ context: UnsafeMutableRawPointer?
    ) {
        let bridgeError = SCBridgeError.configurationError("removeRecordingOutput requires macOS 15.0 SDK or later")
        bridgeError.description.withCString { callback(context, false, $0) }
    }

#endif
