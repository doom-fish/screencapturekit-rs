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
/// - Parameters:
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get")
public func getShareableContent(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    let userDataValue = userData
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                false,
                onScreenWindowsOnly: false
            )
            callback(retain(content), nil, userDataValue)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userDataValue) }
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

/// Gets shareable content with windows below a reference window
/// - Parameters:
///   - excludeDesktopWindows: Whether to exclude desktop windows
///   - referenceWindow: The reference window pointer
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get_below_window")
public func getShareableContentBelowWindow(
    excludeDesktopWindows: Bool,
    referenceWindow: OpaquePointer,
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    let userDataValue = userData
    let window: SCWindow = unretained(referenceWindow)
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnlyBelow: window
            )
            callback(retain(content), nil, userDataValue)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userDataValue) }
        }
    }
}

/// Gets shareable content with windows above a reference window
/// - Parameters:
///   - excludeDesktopWindows: Whether to exclude desktop windows
///   - referenceWindow: The reference window pointer
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get_above_window")
public func getShareableContentAboveWindow(
    excludeDesktopWindows: Bool,
    referenceWindow: OpaquePointer,
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    let userDataValue = userData
    let window: SCWindow = unretained(referenceWindow)
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                excludeDesktopWindows,
                onScreenWindowsOnlyAbove: window
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
/// - Parameters:
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get_current_process_displays")
public func getShareableContentCurrentProcessDisplays(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    let userDataValue = userData
    if #available(macOS 14.4, *) {
        SCShareableContent.getCurrentProcessShareableContent { content, error in
            if let content = content {
                callback(retain(content), nil, userDataValue)
            } else {
                let bridgeError = SCBridgeError.contentUnavailable(error?.localizedDescription ?? "Unknown error")
                bridgeError.description.withCString { callback(nil, $0, userDataValue) }
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
                callback(retain(content), nil, userDataValue)
            } catch {
                let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
                bridgeError.description.withCString { callback(nil, $0, userDataValue) }
            }
        }
    }
}
#else
/// Gets shareable content for the current process (fallback for older compilers)
/// - Parameters:
///   - callback: Called with content pointer or error message
///   - userData: User data passed through to callback
@_cdecl("sc_shareable_content_get_current_process_displays")
public func getShareableContentCurrentProcessDisplays(
    callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    userData: UnsafeMutableRawPointer?
) {
    // Fallback for older compilers (macOS < 14.4 SDK)
    let userDataValue = userData
    Task {
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(
                false,
                onScreenWindowsOnly: true
            )
            callback(retain(content), nil, userDataValue)
        } catch {
            let bridgeError = SCBridgeError.contentUnavailable(error.localizedDescription)
            bridgeError.description.withCString { callback(nil, $0, userDataValue) }
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

// MARK: - SCShareableContentInfo (macOS 14.0+)

@_cdecl("sc_shareable_content_info_for_filter")
public func getShareableContentInfoForFilter(_ filter: OpaquePointer) -> OpaquePointer? {
    let f: SCContentFilter = unretained(filter)
    if #available(macOS 14.0, *) {
        let info = SCShareableContent.info(for: f)
        return retain(info)
    }
    return nil
}

@_cdecl("sc_shareable_content_info_get_style")
public func getShareableContentInfoStyle(_ info: OpaquePointer) -> Int32 {
    if #available(macOS 14.0, *) {
        let i: SCShareableContentInfo = unretained(info)
        switch i.style {
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

@_cdecl("sc_shareable_content_info_get_point_pixel_scale")
public func getShareableContentInfoPointPixelScale(_ info: OpaquePointer) -> Float {
    if #available(macOS 14.0, *) {
        let i: SCShareableContentInfo = unretained(info)
        return i.pointPixelScale
    }
    return 1.0
}

@_cdecl("sc_shareable_content_info_get_content_rect")
public func getShareableContentInfoContentRect(
    _ info: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    if #available(macOS 14.0, *) {
        let i: SCShareableContentInfo = unretained(info)
        let rect = i.contentRect
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

@_cdecl("sc_shareable_content_info_retain")
public func retainShareableContentInfo(_ info: OpaquePointer) -> OpaquePointer {
    if #available(macOS 14.0, *) {
        let i: SCShareableContentInfo = unretained(info)
        return retain(i)
    }
    return info
}

@_cdecl("sc_shareable_content_info_release")
public func releaseShareableContentInfo(_ info: OpaquePointer) {
    release(info)
}

// MARK: - Batch Data Retrieval (Optimized FFI)

/// Get all displays as packed data in a single call
/// Returns: number of displays written, or -1 on error
/// The buffer should be pre-allocated by Rust with enough space for maxDisplays * sizeof(FFIDisplayData)
@_cdecl("sc_shareable_content_get_displays_batch")
public func getDisplaysBatch(
    _ content: OpaquePointer,
    _ rawBuffer: UnsafeMutableRawPointer,
    _ maxDisplays: Int
) -> Int {
    let buffer = rawBuffer.assumingMemoryBound(to: FFIDisplayData.self)
    let sc: SCShareableContent = unretained(content)
    let displays = sc.displays
    let count = min(displays.count, maxDisplays)
    
    for i in 0..<count {
        let d = displays[i]
        buffer[i] = FFIDisplayData(
            displayId: d.displayID,
            width: Int32(d.width),
            height: Int32(d.height),
            frame: FFIRect(d.frame)
        )
    }
    
    return count
}

/// Get all applications as packed data with strings in a separate buffer
/// Returns: number of applications written
/// stringBuffer receives null-terminated strings packed together
/// stringBufferUsed receives actual bytes used in stringBuffer
@_cdecl("sc_shareable_content_get_applications_batch")
public func getApplicationsBatch(
    _ content: OpaquePointer,
    _ rawBuffer: UnsafeMutableRawPointer,
    _ maxApps: Int,
    _ stringBuffer: UnsafeMutablePointer<CChar>,
    _ stringBufferSize: Int,
    _ stringBufferUsed: UnsafeMutablePointer<Int>
) -> Int {
    let buffer = rawBuffer.assumingMemoryBound(to: FFIApplicationData.self)
    let sc: SCShareableContent = unretained(content)
    let apps = sc.applications
    let count = min(apps.count, maxApps)
    var stringOffset: UInt32 = 0
    
    for i in 0..<count {
        let app = apps[i]
        
        // Write bundle ID
        let bundleId = app.bundleIdentifier
        let bundleIdStart = stringOffset
        if let cStr = bundleId.cString(using: .utf8) {
            let len = cStr.count
            if Int(stringOffset) + len <= stringBufferSize {
                for (j, c) in cStr.enumerated() {
                    stringBuffer[Int(stringOffset) + j] = c
                }
                stringOffset += UInt32(len)
            }
        }
        let bundleIdLen = stringOffset - bundleIdStart
        
        // Write app name
        let appName = app.applicationName
        let appNameStart = stringOffset
        if let cStr = appName.cString(using: .utf8) {
            let len = cStr.count
            if Int(stringOffset) + len <= stringBufferSize {
                for (j, c) in cStr.enumerated() {
                    stringBuffer[Int(stringOffset) + j] = c
                }
                stringOffset += UInt32(len)
            }
        }
        let appNameLen = stringOffset - appNameStart
        
        buffer[i] = FFIApplicationData(
            processId: app.processID,
            _padding: 0,
            bundleIdOffset: bundleIdStart,
            bundleIdLength: bundleIdLen > 0 ? bundleIdLen - 1 : 0,  // exclude null terminator from length
            appNameOffset: appNameStart,
            appNameLength: appNameLen > 0 ? appNameLen - 1 : 0
        )
    }
    
    stringBufferUsed.pointee = Int(stringOffset)
    return count
}

/// Get all windows as packed data with strings in a separate buffer
/// Also provides application pointers for ownership lookup
@_cdecl("sc_shareable_content_get_windows_batch")
public func getWindowsBatch(
    _ content: OpaquePointer,
    _ rawBuffer: UnsafeMutableRawPointer,
    _ maxWindows: Int,
    _ stringBuffer: UnsafeMutablePointer<CChar>,
    _ stringBufferSize: Int,
    _ stringBufferUsed: UnsafeMutablePointer<Int>,
    _ appPointers: UnsafeMutablePointer<OpaquePointer?>,
    _ maxApps: Int,
    _ appCount: UnsafeMutablePointer<Int>
) -> Int {
    let buffer = rawBuffer.assumingMemoryBound(to: FFIWindowData.self)
    let sc: SCShareableContent = unretained(content)
    let windows = sc.windows
    let apps = sc.applications
    let count = min(windows.count, maxWindows)
    var stringOffset: UInt32 = 0
    
    // Build app lookup map and populate app pointers
    var appIndexMap: [ObjectIdentifier: Int32] = [:]
    let actualAppCount = min(apps.count, maxApps)
    for i in 0..<actualAppCount {
        appIndexMap[ObjectIdentifier(apps[i])] = Int32(i)
        appPointers[i] = retain(apps[i])
    }
    appCount.pointee = actualAppCount
    
    for i in 0..<count {
        let w = windows[i]
        
        // Write title
        let titleStart = stringOffset
        if let title = w.title, let cStr = title.cString(using: .utf8) {
            let len = cStr.count
            if Int(stringOffset) + len <= stringBufferSize {
                for (j, c) in cStr.enumerated() {
                    stringBuffer[Int(stringOffset) + j] = c
                }
                stringOffset += UInt32(len)
            }
        } else {
            // Write empty string
            if Int(stringOffset) < stringBufferSize {
                stringBuffer[Int(stringOffset)] = 0
                stringOffset += 1
            }
        }
        let titleLen = stringOffset - titleStart
        
        // Find owning app index
        var owningAppIndex: Int32 = -1
        if let owningApp = w.owningApplication {
            owningAppIndex = appIndexMap[ObjectIdentifier(owningApp)] ?? -1
        }
        
        var isActive = false
        if #available(macOS 13.1, *) {
            isActive = w.isActive
        }
        
        buffer[i] = FFIWindowData(
            windowId: w.windowID,
            windowLayer: Int32(w.windowLayer),
            isOnScreen: w.isOnScreen,
            isActive: isActive,
            frame: FFIRect(w.frame),
            titleOffset: titleStart,
            titleLength: titleLen > 0 ? titleLen - 1 : 0,
            owningAppIndex: owningAppIndex,
            _padding: 0
        )
    }
    
    stringBufferUsed.pointee = Int(stringOffset)
    return count
}

// MARK: - Packed Return Types for Simple Getters

/// Get display frame as packed struct (single call instead of 4 out params)
/// Uses out parameters since Swift @_cdecl can't return structs
@_cdecl("sc_display_get_frame_packed")
public func getDisplayFramePacked(
    _ display: OpaquePointer,
    _ outX: UnsafeMutablePointer<Double>,
    _ outY: UnsafeMutablePointer<Double>,
    _ outW: UnsafeMutablePointer<Double>,
    _ outH: UnsafeMutablePointer<Double>
) {
    let d: SCDisplay = unretained(display)
    let frame = d.frame
    outX.pointee = frame.origin.x
    outY.pointee = frame.origin.y
    outW.pointee = frame.size.width
    outH.pointee = frame.size.height
}

/// Get window frame as packed struct
@_cdecl("sc_window_get_frame_packed")
public func getWindowFramePacked(
    _ window: OpaquePointer,
    _ outX: UnsafeMutablePointer<Double>,
    _ outY: UnsafeMutablePointer<Double>,
    _ outW: UnsafeMutablePointer<Double>,
    _ outH: UnsafeMutablePointer<Double>
) {
    let w: SCWindow = unretained(window)
    let frame = w.frame
    outX.pointee = frame.origin.x
    outY.pointee = frame.origin.y
    outW.pointee = frame.size.width
    outH.pointee = frame.size.height
}

/// Get content filter content rect as packed struct (macOS 14.0+)
@_cdecl("sc_content_filter_get_content_rect_packed")
public func getContentFilterContentRectPacked(
    _ filter: OpaquePointer,
    _ outX: UnsafeMutablePointer<Double>,
    _ outY: UnsafeMutablePointer<Double>,
    _ outW: UnsafeMutablePointer<Double>,
    _ outH: UnsafeMutablePointer<Double>
) {
    if #available(macOS 14.0, *) {
        let f: SCContentFilter = unretained(filter)
        let rect = f.contentRect
        outX.pointee = rect.origin.x
        outY.pointee = rect.origin.y
        outW.pointee = rect.size.width
        outH.pointee = rect.size.height
    } else {
        outX.pointee = 0
        outY.pointee = 0
        outW.pointee = 0
        outH.pointee = 0
    }
}

/// Get shareable content info rect as packed struct
@_cdecl("sc_shareable_content_info_get_content_rect_packed")
public func getShareableContentInfoContentRectPacked(
    _ info: OpaquePointer,
    _ outX: UnsafeMutablePointer<Double>,
    _ outY: UnsafeMutablePointer<Double>,
    _ outW: UnsafeMutablePointer<Double>,
    _ outH: UnsafeMutablePointer<Double>
) {
    if #available(macOS 14.0, *) {
        let i: SCShareableContentInfo = unretained(info)
        let rect = i.contentRect
        outX.pointee = rect.origin.x
        outY.pointee = rect.origin.y
        outW.pointee = rect.size.width
        outH.pointee = rect.size.height
    } else {
        outX.pointee = 0
        outY.pointee = 0
        outW.pointee = 0
        outH.pointee = 0
    }
}

// MARK: - Owned String Returns

/// Get window title as owned string (caller must free with sc_free_string)
@_cdecl("sc_window_get_title_owned")
public func getWindowTitleOwned(_ window: OpaquePointer) -> UnsafeMutablePointer<CChar>? {
    let w: SCWindow = unretained(window)
    guard let title = w.title else { return nil }
    return strdup(title)
}

/// Get application bundle identifier as owned string
@_cdecl("sc_running_application_get_bundle_identifier_owned")
public func getRunningApplicationBundleIdentifierOwned(_ app: OpaquePointer) -> UnsafeMutablePointer<CChar>? {
    let a: SCRunningApplication = unretained(app)
    return strdup(a.bundleIdentifier)
}

/// Get application name as owned string
@_cdecl("sc_running_application_get_application_name_owned")
public func getRunningApplicationNameOwned(_ app: OpaquePointer) -> UnsafeMutablePointer<CChar>? {
    let a: SCRunningApplication = unretained(app)
    return strdup(a.applicationName)
}

/// Free a string allocated by Swift (strdup)
@_cdecl("sc_free_string")
public func freeString(_ str: UnsafeMutablePointer<CChar>?) {
    if let str = str {
        free(str)
    }
}
