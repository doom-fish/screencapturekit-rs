// Content Sharing Picker APIs (macOS 14.0+)

import AppKit
import Foundation
import ScreenCaptureKit

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
    var pickerModes: SCContentSharingPickerMode = []
    for mode in modesArray {
        switch mode {
        case 0: pickerModes.insert(.singleWindow)
        case 1: pickerModes.insert(.multipleWindows)
        case 2: pickerModes.insert(.singleDisplay)
        case 3: pickerModes.insert(.singleApplication)
        case 4: pickerModes.insert(.multipleApplications)
        default: break
        }
    }
    box.value.allowedPickerModes = pickerModes
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_allowed_picker_modes_mask")
public func getContentSharingPickerAllowedModesMask(_ config: OpaquePointer) -> UInt64 {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return UInt64(box.value.allowedPickerModes.rawValue)
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_set_allows_changing_selected_content")
public func setContentSharingPickerAllowsChangingSelectedContent(_ config: OpaquePointer, _ allows: Bool) {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    box.value.allowsChangingSelectedContent = allows
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_allows_changing_selected_content")
public func getContentSharingPickerAllowsChangingSelectedContent(_ config: OpaquePointer) -> Bool {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return box.value.allowsChangingSelectedContent
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_set_excluded_bundle_ids")
public func setContentSharingPickerExcludedBundleIDs(
    _ config: OpaquePointer,
    _ bundleIDs: UnsafePointer<UnsafePointer<CChar>?>?,
    _ count: Int
) {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    var ids: [String] = []
    if let bundleIDs {
        for i in 0 ..< count {
            if let ptr = bundleIDs[i] {
                ids.append(String(cString: ptr))
            }
        }
    }
    box.value.excludedBundleIDs = ids
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_excluded_bundle_ids_count")
public func getContentSharingPickerExcludedBundleIDsCount(_ config: OpaquePointer) -> Int {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return box.value.excludedBundleIDs.count
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_excluded_bundle_id_at")
public func getContentSharingPickerExcludedBundleIDAt(
    _ config: OpaquePointer,
    _ index: Int,
    _ buffer: UnsafeMutablePointer<CChar>,
    _ bufferSize: Int
) -> Bool {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    guard index >= 0, index < box.value.excludedBundleIDs.count else { return false }
    let bundleID = box.value.excludedBundleIDs[index]
    return bundleID.withCString { src in
        strlcpy(buffer, src, bufferSize)
        return true
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_set_excluded_window_ids")
public func setContentSharingPickerExcludedWindowIDs(
    _ config: OpaquePointer,
    _ windowIDs: UnsafePointer<UInt32>?,
    _ count: Int
) {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    var ids: [Int] = []
    if let windowIDs {
        for i in 0 ..< count {
            ids.append(Int(windowIDs[i]))
        }
    }
    box.value.excludedWindowIDs = ids
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_excluded_window_ids_count")
public func getContentSharingPickerExcludedWindowIDsCount(_ config: OpaquePointer) -> Int {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return box.value.excludedWindowIDs.count
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_get_excluded_window_id_at")
public func getContentSharingPickerExcludedWindowIDAt(_ config: OpaquePointer, _ index: Int) -> UInt32 {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    guard index >= 0, index < box.value.excludedWindowIDs.count else { return 0 }
    return UInt32(box.value.excludedWindowIDs[index])
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

// MARK: - Picker maximumStreamCount

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_set_maximum_stream_count")
public func setContentSharingPickerMaximumStreamCount(_ count: Int) {
    let picker = SCContentSharingPicker.shared
    if count > 0 {
        picker.maximumStreamCount = count
    } else {
        picker.maximumStreamCount = nil
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_get_maximum_stream_count")
public func getContentSharingPickerMaximumStreamCount() -> Int {
    let picker = SCContentSharingPicker.shared
    return picker.maximumStreamCount ?? 0
}

/// Return a boxed copy of the system's `defaultConfiguration` for the
/// shared content-sharing picker. Callers may then mutate the returned
/// `SCContentSharingPickerConfiguration` (via the existing setter
/// trampolines) and feed it back to `present(...)` — getting "system
/// defaults plus my one tweak" without having to reconstruct every
/// field.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_create_default_configuration")
public func createContentSharingPickerDefaultConfiguration() -> OpaquePointer {
    let picker = SCContentSharingPicker.shared
    let config = picker.defaultConfiguration
    let box = Box(config)
    return retain(box)
}

/// Read whether the shared content-sharing picker is currently marked
/// active. Apple requires `picker.isActive = true` before its UI can
/// appear; the `present*()` trampolines in this bridge always set it
/// implicitly, but consumers may want to query the flag (e.g. to avoid
/// presenting twice) or explicitly deactivate the picker between
/// sessions.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_get_active")
public func getContentSharingPickerActive() -> Bool {
    SCContentSharingPicker.shared.isActive
}

/// Mark the shared content-sharing picker active or inactive. Setting
/// this to `false` hides the Control Center picker UI between
/// sessions; setting to `true` is required before `present*()` can
/// surface the picker (the bridge does this implicitly inside its
/// `present*()` trampolines).
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_set_active")
public func setContentSharingPickerActive(_ active: Bool) {
    SCContentSharingPicker.shared.isActive = active
}

// MARK: - Picker Result with content info

/// Result structure returned by picker - contains filter and content metadata
@available(macOS 14.0, *)
class PickerResult {
    let filter: SCContentFilter
    let contentRect: CGRect
    let pointPixelScale: Double

    // Extracted content from filter
    let windows: [SCWindow]
    let displays: [SCDisplay]
    let applications: [SCRunningApplication]

    init(filter: SCContentFilter) {
        self.filter = filter
        contentRect = filter.contentRect
        pointPixelScale = Double(filter.pointPixelScale)

        // Use public APIs on macOS 15.2+, fall back to KVC on older versions
        #if SCREENCAPTUREKIT_HAS_MACOS15_SDK
            if #available(macOS 15.2, *) {
                windows = filter.includedWindows
                displays = filter.includedDisplays
                applications = filter.includedApplications
            } else {
                // Fallback to KVC for older macOS versions
                windows = (filter.value(forKey: "includedWindows") as? [SCWindow]) ?? []
                displays = (filter.value(forKey: "includedDisplays") as? [SCDisplay]) ?? []
                applications = (filter.value(forKey: "includedApplications") as? [SCRunningApplication]) ?? []
            }
        #else
            // Fallback for older compilers (< Swift 6)
            windows = (filter.value(forKey: "includedWindows") as? [SCWindow]) ?? []
            displays = (filter.value(forKey: "includedDisplays") as? [SCDisplay]) ?? []
            applications = (filter.value(forKey: "includedApplications") as? [SCRunningApplication]) ?? []
        #endif
    }
}

// Base class that owns the one-shot C callback and guarantees it fires at
// most once per observer.
//
// The picker's delegate callbacks are not documented to arrive on the main
// queue, while the replacement path (`fireCancelledIfPending`, invoked from
// the `show*()` trampolines) runs on the main queue. The once-guard is
// therefore protected by an `NSLock` (the same lock pattern used elsewhere
// in this bridge) so the completion can never race itself.
//
// Firing the callback exactly once is also what lets the Rust side reclaim
// the boxed closure context: every code path (success, cancel, error, and
// replacement-cancel) routes the C callback through `beginCompletion()`.
@available(macOS 14.0, *)
class BasePickerObserver: NSObject {
    let callback: @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void
    let userData: UnsafeMutableRawPointer?
    private let lock = NSLock()
    private var hasCompleted = false

    init(callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
         userData: UnsafeMutableRawPointer?)
    {
        self.callback = callback
        self.userData = userData
    }

    // Atomically claim the single completion slot. Returns `true` only for the
    // first caller; all later callers (duplicate delegate fires or a
    // replacement-cancel after the picker already resolved) get `false`.
    func beginCompletion() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        if hasCompleted { return false }
        hasCompleted = true
        return true
    }

    // Deliver a cancelled outcome if (and only if) this observer is still
    // pending. Called when a newer `show*()` request replaces this observer:
    // without this the Rust trampoline would never run and its boxed closure
    // context would leak.
    func fireCancelledIfPending() {
        guard beginCompletion() else { return }
        callback(0, nil, userData) // 0 = cancelled
    }
}

// Observer class to handle picker callbacks - returns filter directly
@available(macOS 14.0, *)
final class PickerObserver: BasePickerObserver, SCContentSharingPickerObserver {
    func contentSharingPicker(_: SCContentSharingPicker, didCancelFor _: SCStream?) {
        guard beginCompletion() else { return }
        callback(0, nil, userData) // 0 = cancelled
    }

    func contentSharingPicker(_: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for _: SCStream?) {
        guard beginCompletion() else { return }
        // Return the filter in the same format as other APIs
        let ptr = ScreenCaptureKitBridge.retain(filter)
        callback(1, ptr, userData) // 1 = success with filter
    }

    func contentSharingPickerStartDidFailWithError(_: Error) {
        guard beginCompletion() else { return }
        callback(-1, nil, userData) // -1 = error
    }
}

// Observer that returns PickerResult with metadata
@available(macOS 14.0, *)
final class PickerObserverWithResult: BasePickerObserver, SCContentSharingPickerObserver {
    func contentSharingPicker(_: SCContentSharingPicker, didCancelFor _: SCStream?) {
        guard beginCompletion() else { return }
        callback(0, nil, userData)
    }

    func contentSharingPicker(_: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for _: SCStream?) {
        guard beginCompletion() else { return }
        // Return PickerResult with metadata
        let result = PickerResult(filter: filter)
        let ptr = ScreenCaptureKitBridge.retain(result)
        callback(1, ptr, userData)
    }

    func contentSharingPickerStartDidFailWithError(_: Error) {
        guard beginCompletion() else { return }
        callback(-1, nil, userData)
    }
}

// Global to keep observer alive during picker
@available(macOS 14.0, *)
private var currentObserver: (BasePickerObserver & SCContentSharingPickerObserver)? = nil

/// Show picker and return SCContentFilter directly (simple API)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show")
public func showContentSharingPicker(
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)

    DispatchQueue.main.async {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)

        let picker = SCContentSharingPicker.shared

        if let old = currentObserver {
            picker.remove(old)
            // Deliver a cancelled outcome to the replaced observer so the Rust
            // trampoline reclaims its boxed closure context (avoids a leak).
            old.fireCancelledIfPending()
        }

        let observer = PickerObserver(callback: callback, userData: userData)
        currentObserver = observer

        picker.isActive = true
        picker.add(observer)
        picker.defaultConfiguration = configBox.value
        picker.present()
    }
}

/// Show picker and return PickerResult with metadata (advanced API)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_with_result")
public func showContentSharingPickerWithResult(
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)

    DispatchQueue.main.async {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)

        let picker = SCContentSharingPicker.shared

        if let old = currentObserver {
            picker.remove(old)
            // Deliver a cancelled outcome to the replaced observer so the Rust
            // trampoline reclaims its boxed closure context (avoids a leak).
            old.fireCancelledIfPending()
        }

        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        currentObserver = observer

        picker.isActive = true
        picker.add(observer)
        picker.defaultConfiguration = configBox.value
        picker.present()
    }
}

/// Show picker for an existing stream (to update filter while capturing)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_for_stream")
public func showContentSharingPickerForStream(
    _ config: OpaquePointer,
    _ streamPtr: OpaquePointer,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let scStream: SCStream = unretained(streamPtr)

    DispatchQueue.main.async {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)

        let picker = SCContentSharingPicker.shared

        if let old = currentObserver {
            picker.remove(old)
            // Deliver a cancelled outcome to the replaced observer so the Rust
            // trampoline reclaims its boxed closure context (avoids a leak).
            old.fireCancelledIfPending()
        }

        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        currentObserver = observer

        picker.isActive = true
        picker.add(observer)
        picker.setConfiguration(configBox.value, for: scStream)
        picker.present(for: scStream)
    }
}

/// Show picker with a specific content style
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_using_style")
public func showContentSharingPickerUsingStyle(
    _ config: OpaquePointer,
    _ style: Int32,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)

    let contentStyle: SCShareableContentStyle = switch style {
    case 1: .window
    case 2: .display
    case 3: .application
    default: .none
    }

    DispatchQueue.main.async {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)

        let picker = SCContentSharingPicker.shared

        if let old = currentObserver {
            picker.remove(old)
            // Deliver a cancelled outcome to the replaced observer so the Rust
            // trampoline reclaims its boxed closure context (avoids a leak).
            old.fireCancelledIfPending()
        }

        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        currentObserver = observer

        picker.isActive = true
        picker.add(observer)
        picker.defaultConfiguration = configBox.value
        picker.present(using: contentStyle)
    }
}

/// Show picker for an existing stream with a specific content style
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_for_stream_using_style")
public func showContentSharingPickerForStreamUsingStyle(
    _ config: OpaquePointer,
    _ streamPtr: OpaquePointer,
    _ style: Int32,
    _ callback: @escaping @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let scStream: SCStream = unretained(streamPtr)

    let contentStyle: SCShareableContentStyle = switch style {
    case 1: .window
    case 2: .display
    case 3: .application
    default: .none
    }

    DispatchQueue.main.async {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)

        let picker = SCContentSharingPicker.shared

        if let old = currentObserver {
            picker.remove(old)
            // Deliver a cancelled outcome to the replaced observer so the Rust
            // trampoline reclaims its boxed closure context (avoids a leak).
            old.fireCancelledIfPending()
        }

        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        currentObserver = observer

        picker.isActive = true
        picker.add(observer)
        picker.setConfiguration(configBox.value, for: scStream)
        picker.present(for: scStream, using: contentStyle)
    }
}

// MARK: - PickerResult accessors

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_filter")
public func getPickerResultFilter(_ result: OpaquePointer) -> OpaquePointer {
    let r: PickerResult = unretained(result)
    return ScreenCaptureKitBridge.retain(r.filter)
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_content_rect")
public func getPickerResultContentRect(
    _ result: OpaquePointer,
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ width: UnsafeMutablePointer<Double>,
    _ height: UnsafeMutablePointer<Double>
) {
    let r: PickerResult = unretained(result)
    x.pointee = r.contentRect.origin.x
    y.pointee = r.contentRect.origin.y
    width.pointee = r.contentRect.size.width
    height.pointee = r.contentRect.size.height
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_scale")
public func getPickerResultScale(_ result: OpaquePointer) -> Double {
    let r: PickerResult = unretained(result)
    return r.pointPixelScale
}

// MARK: - Picked content accessors

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_windows_count")
public func getPickerResultWindowsCount(_ result: OpaquePointer) -> Int {
    let r: PickerResult = unretained(result)
    return r.windows.count
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_window_at")
public func getPickerResultWindowAt(_ result: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let r: PickerResult = unretained(result)
    guard index >= 0, index < r.windows.count else { return nil }
    return ScreenCaptureKitBridge.retain(r.windows[index])
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_displays_count")
public func getPickerResultDisplaysCount(_ result: OpaquePointer) -> Int {
    let r: PickerResult = unretained(result)
    return r.displays.count
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_display_at")
public func getPickerResultDisplayAt(_ result: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let r: PickerResult = unretained(result)
    guard index >= 0, index < r.displays.count else { return nil }
    return ScreenCaptureKitBridge.retain(r.displays[index])
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_applications_count")
public func getPickerResultApplicationsCount(_ result: OpaquePointer) -> Int {
    let r: PickerResult = unretained(result)
    return r.applications.count
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_get_application_at")
public func getPickerResultApplicationAt(_ result: OpaquePointer, _ index: Int) -> OpaquePointer? {
    let r: PickerResult = unretained(result)
    guard index >= 0, index < r.applications.count else { return nil }
    return ScreenCaptureKitBridge.retain(r.applications[index])
}

@available(macOS 14.0, *)
@_cdecl("sc_picker_result_release")
public func releasePickerResult(_ result: OpaquePointer) {
    release(result)
}
