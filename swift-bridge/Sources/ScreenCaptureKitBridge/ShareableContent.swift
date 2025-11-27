// ShareableContent APIs - SCShareableContent, SCDisplay, SCWindow, SCRunningApplication

import CoreGraphics
import Foundation
@preconcurrency import ScreenCaptureKit

// MARK: - CoreGraphics Initialization

/// Force CoreGraphics initialization by calling CGMainDisplayID
/// This prevents CGS_REQUIRE_INIT crashes on headless systems
private func ensureCoreGraphicsInitialized() {
    _ = CGMainDisplayID()
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
    ensureCoreGraphicsInitialized()
    
    let semaphore = DispatchSemaphore(value: 0)
    var resultContent: SCShareableContent?
    var resultError: String?

    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnly: onScreenWindowsOnly
            )
            resultContent = content
        } catch {
            resultError = error.localizedDescription
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

    if let error = resultError {
        error.withCString { ptr in
            strncpy(errorBuffer, ptr, errorBufferSize - 1)
            errorBuffer[errorBufferSize - 1] = 0
        }
        return nil
    }

    if let content = resultContent {
        return retain(content)
    }

    "Unknown error".withCString { ptr in
        strncpy(errorBuffer, ptr, errorBufferSize - 1)
        errorBuffer[errorBufferSize - 1] = 0
    }
    return nil
}

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

/// Async callback-based shareable content retrieval with user_data
@_cdecl("sc_shareable_content_get_async")
public func getShareableContentAsync(
    excludeDesktopWindows: Bool,
    onScreenWindowsOnly: Bool,
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnly: onScreenWindowsOnly
            )
            callback(retain(content), nil, userData)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0, userData) }
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
