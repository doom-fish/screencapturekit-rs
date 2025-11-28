// ShareableContent APIs - SCShareableContent, SCDisplay, SCWindow, SCRunningApplication

import CoreGraphics
import Foundation
import ScreenCaptureKit



// MARK: - Thread-safe result holder

private class ResultHolder<T> {
    private let lock = NSLock()
    private var _value: T?
    private var _error: String?
    
    var value: T? {
        get { lock.lock(); defer { lock.unlock() }; return _value }
        set { lock.lock(); defer { lock.unlock() }; _value = newValue }
    }
    
    var error: String? {
        get { lock.lock(); defer { lock.unlock() }; return _error }
        set { lock.lock(); defer { lock.unlock() }; _error = newValue }
    }
}

// MARK: - ShareableContent: Content Discovery

/// Synchronous blocking call to get shareable content
/// Uses DispatchSemaphore to block until async completes
/// Returns content pointer on success, or writes error message to errorBuffer
@_cdecl("sc_shareable_content_get_sync")
public func getShareableContentSync(
    excludeDesktopWindows: Bool,
    onScreenWindowsOnly: Bool,
    errorBuffer: UnsafeMutablePointer<CChar>,
    errorBufferSize: Int
) -> OpaquePointer? {
    // Force CoreGraphics initialization
    initializeCoreGraphics()
    
    let semaphore = DispatchSemaphore(value: 0)
    let holder = ResultHolder<SCShareableContent>()

    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnly: onScreenWindowsOnly
            )
            holder.value = content
        } catch {
            holder.error = SCBridgeError.contentUnavailable(error.localizedDescription).description
        }
        semaphore.signal()
    }

    // Wait with timeout (5 seconds)
    let timeout = semaphore.wait(timeout: .now() + 5.0)

    if timeout == .timedOut {
        "Timeout waiting for shareable content".withCString { ptr in
            strncpy(errorBuffer, ptr, errorBufferSize - 1)
            errorBuffer[errorBufferSize - 1] = 0
        }
        return nil
    }

    if let error = holder.error {
        error.withCString { ptr in
            strncpy(errorBuffer, ptr, errorBufferSize - 1)
            errorBuffer[errorBufferSize - 1] = 0
        }
        return nil
    }

    if let content = holder.value {
        return retain(content)
    }

    "Unknown error".withCString { ptr in
        strncpy(errorBuffer, ptr, errorBufferSize - 1)
        errorBuffer[errorBufferSize - 1] = 0
    }
    return nil
}

/// Gets shareable content asynchronously
/// - Parameter callback: Called with content pointer or error message
@_cdecl("sc_shareable_content_get")
public func getShareableContent(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                false,
                onScreenWindowsOnly: false
            )
            callback(retain(content), nil)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0) }
        }
    }
}

/// Gets shareable content with options asynchronously
/// - Parameters:
///   - excludeDesktopWindows: Whether to exclude desktop windows
///   - onScreenWindowsOnly: Whether to only include on-screen windows
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get_with_options")
public func getShareableContentWithOptions(
    excludeDesktopWindows: Bool,
    onScreenWindowsOnly: Bool,
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    // Capture userData as a raw value to avoid Sendable issues
    let userDataValue = userData
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnly: onScreenWindowsOnly
            )
            callback(retain(content), nil, userDataValue)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userDataValue) }
        }
    }
}

#if compiler(>=6.0)
/// Gets shareable content for the current process (macOS 14.4+)
/// - Parameter callback: Called with content pointer or error message
@_cdecl("sc_shareable_content_get_current_process_displays")
public func getShareableContentCurrentProcessDisplays(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    if #available(macOS 14.4, *) {
        SCShareableContent.getCurrentProcessShareableContent { content, error in
            if let content = content {
                callback(retain(content), nil)
            } else {
                let bridgeError = SCBridgeError.contentUnavailable(error?.localizedDescription ?? "Unknown error")
                bridgeError.description.withCString { callback(nil, $0) }
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
                let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
                bridgeError.description.withCString { callback(nil, $0) }
            }
        }
    }
}
#else
/// Gets shareable content for the current process (fallback for older compilers)
/// - Parameter callback: Called with content pointer or error message
@_cdecl("sc_shareable_content_get_current_process_displays")
public func getShareableContentCurrentProcessDisplays(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?) -> Void
) {
    // Fallback for older compilers (macOS < 14.4 SDK)
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                false,
                onScreenWindowsOnly: true
            )
            callback(retain(content), nil)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0) }
        }
    }
}
#endif

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
public func getShareableContentDisplay(_ content: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let sc: SCShareableContent = unretained(content)
    guard index >= 0 && index < sc.displays.count else { return nil }
    return retain(sc.displays[index])
}

@_cdecl("sc_shareable_content_get_windows_count")
public func getShareableContentWindowsCount(_ content: OpaquePointer) -> Int {
    let sc: SCShareableContent = unretained(content)
    return sc.windows.count
}

@_cdecl("sc_shareable_content_get_window_at")
public func getShareableContentWindow(_ content: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let sc: SCShareableContent = unretained(content)
    guard index >= 0 && index < sc.windows.count else { return nil }
    return retain(sc.windows[index])
}

@_cdecl("sc_shareable_content_get_applications_count")
public func getShareableContentApplicationsCount(_ content: OpaquePointer) -> Int {
    let sc: SCShareableContent = unretained(content)
    return sc.applications.count
}

@_cdecl("sc_shareable_content_get_application_at")
public func getShareableContentApplication(_ content: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let sc: SCShareableContent = unretained(content)
    guard index >= 0 && index < sc.applications.count else { return nil }
    return retain(sc.applications[index])
}

// MARK: - SCDisplay

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
public func getDisplayId(_ display: OpaquePointer) -> UInt32 {
    let d: SCDisplay = unretained(display)
    return d.displayID
}

@_cdecl("sc_display_get_width")
public func getDisplayWidth(_ display: OpaquePointer) -> Int {
    let d: SCDisplay = unretained(display)
    return d.width
}

@_cdecl("sc_display_get_height")
public func getDisplayHeight(_ display: OpaquePointer) -> Int {
    let d: SCDisplay = unretained(display)
    return d.height
}

@_cdecl("sc_display_get_frame")
public func getDisplayFrame(_ display: OpaquePointer, _ outX: UnsafeMutablePointer<Double>, _ outY: UnsafeMutablePointer<Double>, _ outW: UnsafeMutablePointer<Double>, _ outH: UnsafeMutablePointer<Double>) {
    let d: SCDisplay = unretained(display)
    let frame = d.frame
    outX.pointee = frame.origin.x
    outY.pointee = frame.origin.y
    outW.pointee = frame.size.width
    outH.pointee = frame.size.height
}

// MARK: - SCWindow

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
public func getWindowId(_ window: OpaquePointer) -> UInt32 {
    let w: SCWindow = unretained(window)
    return w.windowID
}

@_cdecl("sc_window_get_title")
public func getWindowTitle(_ window: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let w: SCWindow = unretained(window)
    guard let title = w.title, let cString = title.cString(using: .utf8) else {
        return false
    }
    strncpy(buffer, cString, bufferSize - 1)
    buffer[bufferSize - 1] = 0
    return true
}

@_cdecl("sc_window_get_frame")
public func getWindowFrame(_ window: OpaquePointer, _ outX: UnsafeMutablePointer<Double>, _ outY: UnsafeMutablePointer<Double>, _ outW: UnsafeMutablePointer<Double>, _ outH: UnsafeMutablePointer<Double>) {
    let w: SCWindow = unretained(window)
    let frame = w.frame
    outX.pointee = frame.origin.x
    outY.pointee = frame.origin.y
    outW.pointee = frame.size.width
    outH.pointee = frame.size.height
}

@_cdecl("sc_window_is_on_screen")
public func getWindowIsOnScreen(_ window: OpaquePointer) -> Bool {
    let w: SCWindow = unretained(window)
    return w.isOnScreen
}

@_cdecl("sc_window_is_active")
public func getWindowIsActive(_ window: OpaquePointer) -> Bool {
    let w: SCWindow = unretained(window)
    if #available(macOS 13.1, *) { return w.isActive } else { return false }
}

@_cdecl("sc_window_get_window_layer")
public func getWindowLayer(_ window: OpaquePointer) -> Int {
    let w: SCWindow = unretained(window)
    return w.windowLayer
}

@_cdecl("sc_window_get_owning_application")
public func getWindowOwningApplication(_ window: OpaquePointer) -> OpaquePointer? {
    let w: SCWindow = unretained(window)
    guard let app = w.owningApplication else { return nil }
    return retain(app)
}

// MARK: - SCRunningApplication

@_cdecl("sc_running_application_retain")
public func retainRunningApplication(_ app: OpaquePointer) -> OpaquePointer {
    let a: SCRunningApplication = unretained(app)
    return retain(a)
}

@_cdecl("sc_running_application_release")
public func releaseRunningApplication(_ app: OpaquePointer) {
    release(app)
}

@_cdecl("sc_running_application_get_process_id")
public func getRunningApplicationProcessId(_ app: OpaquePointer) -> Int32 {
    let a: SCRunningApplication = unretained(app)
    return a.processID
}

@_cdecl("sc_running_application_get_bundle_identifier")
public func getRunningApplicationBundleIdentifier(_ app: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let a: SCRunningApplication = unretained(app)
    let bundleId = a.bundleIdentifier; guard let cString = bundleId.cString(using: .utf8) else {
        return false
    }
    strncpy(buffer, cString, bufferSize - 1)
    buffer[bufferSize - 1] = 0
    return true
}

@_cdecl("sc_running_application_get_application_name")
public func getRunningApplicationName(_ app: OpaquePointer, _ buffer: UnsafeMutablePointer<CChar>, _ bufferSize: Int) -> Bool {
    let a: SCRunningApplication = unretained(app)
    let name = a.applicationName; guard let cString = name.cString(using: .utf8) else {
        return false
    }
    strncpy(buffer, cString, bufferSize - 1)
    buffer[bufferSize - 1] = 0
    return true
}
