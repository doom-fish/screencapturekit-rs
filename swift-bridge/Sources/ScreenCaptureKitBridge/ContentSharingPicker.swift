// Content Sharing Picker APIs (macOS 14.0+)

import AppKit
import CoreFoundation
import Foundation
import ScreenCaptureKit

@_cdecl("sc_content_sharing_picker_is_available")
public func contentSharingPickerIsAvailable() -> Bool {
    if #available(macOS 14.0, *) {
        return true
    }
    return false
}

#if SCREENCAPTUREKIT_HAS_MACOS14_SDK

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
    _ buffer: UnsafeMutablePointer<CChar>?,
    _ bufferSize: Int
) -> Bool {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    guard index >= 0, index < box.value.excludedBundleIDs.count else { return false }
    return writeCString(box.value.excludedBundleIDs[index], into: buffer, bufferSize: bufferSize)
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

/// Produce a **new, independent** configuration holding a copy of `config`'s
/// current values.
///
/// `sc_content_sharing_picker_configuration_retain` bumps the refcount of the
/// shared `Box`, so two handles to it observe each other's mutations. Rust
/// exposes `&mut self` setters and marks the wrapper `Send + Sync`, which is
/// only sound if each handle owns its box exclusively — hence this thunk backs
/// `Clone` instead.
///
/// `SCContentSharingPickerConfiguration` is a Swift value type, so assigning
/// `box.value` into a fresh `Box` copies every field, including any Apple adds
/// in a later SDK.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_configuration_copy")
public func copyContentSharingPickerConfiguration(_ config: OpaquePointer) -> OpaquePointer {
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    return retain(Box(box.value))
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

        // `includedWindows` / `includedDisplays` / `includedApplications` are
        // macOS 15.2 additions; older systems only expose them through KVC.
        #if SCREENCAPTUREKIT_HAS_MACOS15_2_SDK
            if #available(macOS 15.2, *) {
                windows = filter.includedWindows
                displays = filter.includedDisplays
                applications = filter.includedApplications
            } else {
                windows = (filter.value(forKey: "includedWindows") as? [SCWindow]) ?? []
                displays = (filter.value(forKey: "includedDisplays") as? [SCDisplay]) ?? []
                applications = (filter.value(forKey: "includedApplications") as? [SCRunningApplication]) ?? []
            }
        #else
            windows = (filter.value(forKey: "includedWindows") as? [SCWindow]) ?? []
            displays = (filter.value(forKey: "includedDisplays") as? [SCDisplay]) ?? []
            applications = (filter.value(forKey: "includedApplications") as? [SCRunningApplication]) ?? []
        #endif
    }
}
// MARK: - Callback ABI

/// One-shot completion ABI used by the `show*()` trampolines.
/// `(code, resultPtr, userData)` where code is 1 = picked, 0 = cancelled, -1 = error.
public typealias PickerOneShotCallback = @convention(c) (Int32, OpaquePointer?, UnsafeMutableRawPointer?) -> Void

/// Repeating observer ABI used by `sc_content_sharing_picker_add_observer`.
/// `(event, resultPtr, message, streamPtr, userData)` where event is
/// 1 = updated, 0 = cancelled, -1 = start-failed (message non-nil).
public typealias PickerEventCallback =
    @convention(c) (
        Int32,
        OpaquePointer?,
        UnsafePointer<CChar>?,
        OpaquePointer?,
        UnsafeMutableRawPointer?
    ) -> Void

/// Releases the Rust-side boxed context backing an observer. Invoked exactly
/// once, after the observer has been detached from `SCContentSharingPicker`.
public typealias PickerContextRelease = @convention(c) (UnsafeMutableRawPointer?) -> Void

@available(macOS 14.0, *)
private func pickerContentStyle(from raw: Int32) -> SCShareableContentStyle {
    switch raw {
    case 1: .window
    case 2: .display
    case 3: .application
    default: .none
    }
}

// MARK: - Activation scope

/// Presenting `SCContentSharingPicker` requires the host process to be a
/// regular (Dock-visible) app. Pure-Rust hosts are usually `.prohibited`, so
/// the bridge has to promote them — but doing that permanently leaves a stray
/// Dock icon and menu bar behind for the rest of the process lifetime.
///
/// `PickerActivationScope` makes the promotion *scoped and reference counted*:
/// the original policy is captured on the first acquire and restored once the
/// last holder releases. All access is confined to the main queue, which is
/// where every `present*()` / completion path in this file already runs, so no
/// additional locking is required.
@available(macOS 14.0, *)
private enum PickerActivationScope {
    private nonisolated(unsafe) static var holders = 0
    private nonisolated(unsafe) static var standaloneActive = false
    private nonisolated(unsafe) static var savedPolicy: NSApplication.ActivationPolicy?

    /// Promote to `.regular` (remembering the previous policy) and focus the app.
    static func acquire() {
        dispatchPrecondition(condition: .onQueue(.main))
        let app = NSApplication.shared
        if holders == 0, app.activationPolicy() != .regular {
            savedPolicy = app.activationPolicy()
            app.setActivationPolicy(.regular)
        }
        holders += 1
        app.activate(ignoringOtherApps: true)
    }

    /// Drop one holder; restore the original policy when the count reaches zero.
    static func release() {
        dispatchPrecondition(condition: .onQueue(.main))
        guard holders > 0 else { return }
        holders -= 1
        if holders == 0 {
            restore()
        }
    }

    static func acquireStandalone() {
        guard !standaloneActive else { return }
        standaloneActive = true
        acquire()
    }

    static func releaseStandalone() {
        guard standaloneActive else { return }
        standaloneActive = false
        release()
    }

    /// Force-drop every holder and restore the original policy. Used by the
    /// explicit `deactivate()` entry point.
    static func releaseAll() {
        dispatchPrecondition(condition: .onQueue(.main))
        holders = 0
        standaloneActive = false
        restore()
    }

    private static func restore() {
        if let savedPolicy {
            NSApplication.shared.setActivationPolicy(savedPolicy)
            PickerActivationScope.savedPolicy = nil
        }
    }
}

@available(macOS 14.0, *)
private nonisolated(unsafe) var pendingPersistentActivationCleanup = false

// MARK: - Persistent (repeating) observers

/// Observer that forwards **every** picker event for as long as it is
/// registered. This is what makes
/// `SCContentSharingPickerConfiguration.allowsChangingSelectedContent` usable:
/// Apple re-invokes `didUpdateWith:` each time the user re-picks, and a
/// one-shot latch would swallow everything after the first selection.
///
/// Rust receives only an opaque registry token, so a late callback racing
/// `teardown()` is ignored after the token is removed.
@available(macOS 14.0, *)
final class PersistentPickerObserver: NSObject, SCContentSharingPickerObserver {
    let token: Int64
    private let callback: PickerEventCallback
    private let contextRelease: PickerContextRelease
    private let userData: UnsafeMutableRawPointer?
    private let lock = NSLock()
    private var torndown = false

    init(
        token: Int64,
        callback: @escaping PickerEventCallback,
        contextRelease: @escaping PickerContextRelease,
        userData: UnsafeMutableRawPointer?
    ) {
        self.token = token
        self.callback = callback
        self.contextRelease = contextRelease
        self.userData = userData
    }

    func teardown() {
        lock.lock()
        guard !torndown else {
            lock.unlock()
            return
        }
        torndown = true
        lock.unlock()
        contextRelease(userData)
    }

    private func deliver(
        _ event: Int32,
        filter: SCContentFilter?,
        message: UnsafePointer<CChar>?,
        stream: SCStream?
    ) {
        lock.lock()
        let active = !torndown
        lock.unlock()
        guard active else { return }
        // Retain the result only once we know it will be delivered, so a
        // dropped event cannot leak a PickerResult.
        let ptr = filter.map { ScreenCaptureKitBridge.retain(PickerResult(filter: $0)) }
        let streamPtr = stream.map { OpaquePointer(Unmanaged.passUnretained($0).toOpaque()) }
        callback(event, ptr, message, streamPtr, userData)
    }

    func contentSharingPicker(_: SCContentSharingPicker, didCancelFor stream: SCStream?) {
        deliver(0, filter: nil, message: nil, stream: stream)
        releaseStandaloneActivation()
    }

    func contentSharingPicker(
        _: SCContentSharingPicker,
        didUpdateWith filter: SCContentFilter,
        for stream: SCStream?
    ) {
        deliver(1, filter: filter, message: nil, stream: stream)
        releaseStandaloneActivation()
    }

    func contentSharingPickerStartDidFailWithError(_ error: Error) {
        error.localizedDescription.withCString {
            deliver(-1, filter: nil, message: $0, stream: nil)
        }
        releaseStandaloneActivation()
    }

    private func releaseStandaloneActivation() {
        DispatchQueue.main.async {
            PickerActivationScope.releaseStandalone()
        }
    }
}

private final class PersistentObserverInstallResult: @unchecked Sendable {
    private let lock = NSLock()
    private var installed = false

    func markInstalled() {
        lock.lock()
        installed = true
        lock.unlock()
    }

    var wasInstalled: Bool {
        lock.lock()
        defer { lock.unlock() }
        return installed
    }
}

@available(macOS 14.0, *)
private enum PersistentObserverRegistry {
    private static let lock = NSLock()
    private nonisolated(unsafe) static var observers: [Int64: PersistentPickerObserver] = [:]
    private nonisolated(unsafe) static var nextToken: Int64 = 1

    static func insert(
        callback: @escaping PickerEventCallback,
        contextRelease: @escaping PickerContextRelease,
        userData: UnsafeMutableRawPointer?
    ) -> PersistentPickerObserver {
        lock.lock()
        defer { lock.unlock() }
        var token = nextToken
        while token == 0 || observers[token] != nil {
            token = token == Int64.max ? 1 : token + 1
        }
        nextToken = token == Int64.max ? 1 : token + 1
        let observer = PersistentPickerObserver(
            token: token,
            callback: callback,
            contextRelease: contextRelease,
            userData: userData
        )
        observers[token] = observer
        return observer
    }

    static func take(_ token: Int64) -> PersistentPickerObserver? {
        lock.lock()
        defer { lock.unlock() }
        return observers.removeValue(forKey: token)
    }

    static func takeAll() -> [PersistentPickerObserver] {
        lock.lock()
        defer { lock.unlock() }
        let all = Array(observers.values)
        observers.removeAll()
        return all
    }

    static var isEmpty: Bool {
        lock.lock()
        defer { lock.unlock() }
        return observers.isEmpty
    }

    static func contains(_ token: Int64) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return observers[token] != nil
    }

}

/// Register a repeating observer. Returns a non-zero token used to remove it,
/// or 0 if registration failed.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_add_observer")
public func addContentSharingPickerObserver(
    _ callback: @escaping PickerEventCallback,
    _ contextRelease: @escaping PickerContextRelease,
    _ userData: UnsafeMutableRawPointer?
) -> Int64 {
    let canHopToMain =
        Thread.isMainThread || CFRunLoopCopyCurrentMode(CFRunLoopGetMain()) != nil
    guard canHopToMain else { return 0 }
    let observer = PersistentObserverRegistry.insert(
        callback: callback,
        contextRelease: contextRelease,
        userData: userData
    )
    let result = PersistentObserverInstallResult()
    let install = {
        guard PersistentObserverRegistry.contains(observer.token) else {
            observer.teardown()
            return
        }
        let picker = SCContentSharingPicker.shared
        picker.add(observer)
        picker.isActive = true
        pendingPersistentActivationCleanup = false
        result.markInstalled()
    }
    if Thread.isMainThread {
        install()
    } else {
        let completion = DispatchSemaphore(value: 0)
        DispatchQueue.main.async {
            install()
            completion.signal()
        }
        if completion.wait(timeout: .now() + 5) == .timedOut {
            PersistentObserverRegistry.take(observer.token)?.teardown()
            DispatchQueue.main.async {
                let picker = SCContentSharingPicker.shared
                picker.remove(observer)
                if PersistentObserverRegistry.isEmpty, currentObserver == nil {
                    picker.isActive = false
                    PickerActivationScope.releaseAll()
                }
            }
            return 0
        }
    }
    return result.wasInstalled ? observer.token : 0
}

/// Remove a previously registered repeating observer. Returns `true` if the
/// token matched a live observer.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_remove_observer")
public func removeContentSharingPickerObserver(_ token: Int64) -> Bool {
    guard let observer = PersistentObserverRegistry.take(token) else { return false }
    observer.teardown()
    DispatchQueue.main.async {
        let picker = SCContentSharingPicker.shared
        picker.remove(observer)
        if PersistentObserverRegistry.isEmpty {
            if currentObserver == nil {
                picker.isActive = false
                PickerActivationScope.releaseAll()
            } else {
                pendingPersistentActivationCleanup = true
            }
        }
    }
    return true
}

/// Remove every repeating observer registered through this bridge.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_remove_all_observers")
public func removeAllContentSharingPickerObservers() -> Int {
    let all = PersistentObserverRegistry.takeAll()
    guard !all.isEmpty else { return 0 }
    for observer in all {
        observer.teardown()
    }
    DispatchQueue.main.async {
        let picker = SCContentSharingPicker.shared
        for observer in all {
            picker.remove(observer)
        }
        if PersistentObserverRegistry.isEmpty {
            if currentObserver == nil {
                picker.isActive = false
                PickerActivationScope.releaseAll()
            } else {
                pendingPersistentActivationCleanup = true
            }
        }
    }
    return all.count
}

// MARK: - Standalone configuration operations

/// Assign the picker's process-wide `defaultConfiguration`.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_set_default_configuration")
public func setContentSharingPickerDefaultConfiguration(_ config: OpaquePointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let box: Box<SCContentSharingPickerConfiguration> = unretained(config)
    SCContentSharingPicker.shared.defaultConfiguration = box.value
    return true
}

/// Assign (or clear, when `config` is nil) the per-stream picker configuration.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_set_configuration_for_stream")
public func setContentSharingPickerConfigurationForStream(
    _ config: OpaquePointer?,
    _ streamPtr: OpaquePointer
) -> Bool {
    guard Thread.isMainThread else { return false }
    let scStream: SCStream = unretained(streamPtr)
    let value: SCContentSharingPickerConfiguration? = config.map {
        let box: Box<SCContentSharingPickerConfiguration> = unretained($0)
        return box.value
    }
    SCContentSharingPicker.shared.setConfiguration(value, for: scStream)
    return true
}

// MARK: - Standalone present operations (pair with repeating observers)

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_present")
public func presentContentSharingPicker(_ style: Int32) {
    DispatchQueue.main.async {
        PickerActivationScope.acquireStandalone()
        let picker = SCContentSharingPicker.shared
        picker.isActive = true
        if style < 0 {
            picker.present()
        } else {
            picker.present(using: pickerContentStyle(from: style))
        }
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_present_for_stream")
public func presentContentSharingPickerForStream(_ streamPtr: OpaquePointer, _ style: Int32) {
    let scStream: SCStream = unretained(streamPtr)
    DispatchQueue.main.async {
        PickerActivationScope.acquireStandalone()
        let picker = SCContentSharingPicker.shared
        picker.isActive = true
        if style < 0 {
            picker.present(for: scStream)
        } else {
            picker.present(for: scStream, using: pickerContentStyle(from: style))
        }
    }
}

/// Deactivate the picker and undo any activation-policy promotion this bridge
/// performed. Does **not** implicitly remove registered observers.
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_deactivate")
public func deactivateContentSharingPicker() {
    DispatchQueue.main.async {
        SCContentSharingPicker.shared.isActive = false
        PickerActivationScope.releaseAll()
    }
}

// MARK: - One-shot observers (`show*()` helpers)

/// Base class that owns the one-shot C callback and guarantees it fires at
/// most once per observer.
///
/// The picker's delegate callbacks are not documented to arrive on the main
/// queue, while the replacement path (`fireCancelledIfPending`, invoked from
/// the `show*()` trampolines) runs on the main queue. The once-guard is
/// therefore protected by an `NSLock` (the same lock pattern used elsewhere
/// in this bridge) so the completion can never race itself.
///
/// Firing the callback exactly once is also what lets the Rust side reclaim
/// the boxed closure context: every code path (success, cancel, error, and
/// replacement-cancel) routes the C callback through `beginCompletion()`.
@available(macOS 14.0, *)
class BasePickerObserver: NSObject {
    let callback: PickerOneShotCallback
    let userData: UnsafeMutableRawPointer?
    private let lock = NSLock()
    private var hasCompleted = false

    init(callback: @escaping PickerOneShotCallback, userData: UnsafeMutableRawPointer?) {
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
        callback(0, nil, userData)
        finishOneShotSession(self)
    }
}

/// Shared teardown for a resolved one-shot picker session: detach the
/// observer, drop the activation-policy promotion, and put the shared picker
/// back to `isActive = false` when no repeating observer still wants it.
///
/// Without this the previous implementation leaked the observer into a global
/// for the process lifetime and left `isActive == true` forever, which keeps
/// the Control Center "ready to share" entry lit.
@available(macOS 14.0, *)
func finishOneShotSession(_ observer: BasePickerObserver) {
    DispatchQueue.main.async {
        let picker = SCContentSharingPicker.shared
        if let typed = observer as? SCContentSharingPickerObserver {
            picker.remove(typed)
        }
        if currentObserver === observer {
            currentObserver = nil
        }
        PickerActivationScope.release()
        if PersistentObserverRegistry.isEmpty, currentObserver == nil {
            picker.isActive = false
            if pendingPersistentActivationCleanup {
                pendingPersistentActivationCleanup = false
                PickerActivationScope.releaseAll()
            }
        }
    }
}

// Observer class to handle picker callbacks - returns filter directly
@available(macOS 14.0, *)
final class PickerObserver: BasePickerObserver, SCContentSharingPickerObserver {
    func contentSharingPicker(_: SCContentSharingPicker, didCancelFor _: SCStream?) {
        guard beginCompletion() else { return }
        callback(0, nil, userData) // 0 = cancelled
        finishOneShotSession(self)
    }

    func contentSharingPicker(_: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for _: SCStream?) {
        guard beginCompletion() else { return }
        // Return the filter in the same format as other APIs
        let ptr = ScreenCaptureKitBridge.retain(filter)
        callback(1, ptr, userData) // 1 = success with filter
        finishOneShotSession(self)
    }

    func contentSharingPickerStartDidFailWithError(_: Error) {
        guard beginCompletion() else { return }
        callback(-1, nil, userData) // -1 = error
        finishOneShotSession(self)
    }
}

// Observer that returns PickerResult with metadata
@available(macOS 14.0, *)
final class PickerObserverWithResult: BasePickerObserver, SCContentSharingPickerObserver {
    func contentSharingPicker(_: SCContentSharingPicker, didCancelFor _: SCStream?) {
        guard beginCompletion() else { return }
        callback(0, nil, userData)
        finishOneShotSession(self)
    }

    func contentSharingPicker(_: SCContentSharingPicker, didUpdateWith filter: SCContentFilter, for _: SCStream?) {
        guard beginCompletion() else { return }
        // Return PickerResult with metadata
        let result = PickerResult(filter: filter)
        let ptr = ScreenCaptureKitBridge.retain(result)
        callback(1, ptr, userData)
        finishOneShotSession(self)
    }

    func contentSharingPickerStartDidFailWithError(_: Error) {
        guard beginCompletion() else { return }
        callback(-1, nil, userData)
        finishOneShotSession(self)
    }
}

// Global tracking the in-flight one-shot observer. Main-queue confined.
@available(macOS 14.0, *)
private nonisolated(unsafe) var currentObserver: (BasePickerObserver & SCContentSharingPickerObserver)?

/// Install a fresh one-shot observer, cancelling and detaching any previous
/// in-flight one. Must run on the main queue.
@available(macOS 14.0, *)
private func installOneShotObserver(
    _ observer: BasePickerObserver & SCContentSharingPickerObserver
) -> SCContentSharingPicker {
    let picker = SCContentSharingPicker.shared

    if let old = currentObserver {
        picker.remove(old)
        currentObserver = nil
        // Deliver a cancelled outcome to the replaced observer so the Rust
        // trampoline reclaims its boxed closure context (avoids a leak).
        old.fireCancelledIfPending()
    }

    PickerActivationScope.acquire()
    currentObserver = observer
    picker.isActive = true
    picker.add(observer)
    return picker
}

/// Presents `body` on the main queue, or reports an error when nothing will
/// ever drain that queue.
///
/// A process without a running main run loop — a plain CLI binary, or a
/// `cargo test` harness — never executes the hop, so the picker would neither
/// appear nor invoke the callback, stranding the caller's boxed context
/// forever. Reporting failure lets the Rust trampoline reclaim it. Any code
/// other than 0 or 1 decodes as an error on the Rust side.
@available(macOS 14.0, *)
private func presentOnMain(
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?,
    _ body: @escaping () -> Void
) {
    let canHopToMain =
        Thread.isMainThread || CFRunLoopCopyCurrentMode(CFRunLoopGetMain()) != nil
    guard canHopToMain else {
        callback(-1, nil, userData)
        return
    }
    DispatchQueue.main.async(execute: body)
}

/// Show picker and return SCContentFilter directly (simple API)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show")
public func showContentSharingPicker(
    _ config: OpaquePointer,
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let configValue = configBox.value

    presentOnMain(callback, userData) {
        let observer = PickerObserver(callback: callback, userData: userData)
        let picker = installOneShotObserver(observer)
        picker.defaultConfiguration = configValue
        picker.present()
    }
}

/// Show picker and return PickerResult with metadata (advanced API)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_with_result")
public func showContentSharingPickerWithResult(
    _ config: OpaquePointer,
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let configValue = configBox.value

    presentOnMain(callback, userData) {
        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        let picker = installOneShotObserver(observer)
        picker.defaultConfiguration = configValue
        picker.present()
    }
}

/// Show picker for an existing stream (to update filter while capturing)
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_for_stream")
public func showContentSharingPickerForStream(
    _ config: OpaquePointer,
    _ streamPtr: OpaquePointer,
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let configValue = configBox.value
    let scStream: SCStream = unretained(streamPtr)

    presentOnMain(callback, userData) {
        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        let picker = installOneShotObserver(observer)
        picker.setConfiguration(configValue, for: scStream)
        picker.present(for: scStream)
    }
}

/// Show picker with a specific content style
@available(macOS 14.0, *)
@_cdecl("sc_content_sharing_picker_show_using_style")
public func showContentSharingPickerUsingStyle(
    _ config: OpaquePointer,
    _ style: Int32,
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let configValue = configBox.value
    let contentStyle = pickerContentStyle(from: style)

    presentOnMain(callback, userData) {
        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        let picker = installOneShotObserver(observer)
        picker.defaultConfiguration = configValue
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
    _ callback: @escaping PickerOneShotCallback,
    _ userData: UnsafeMutableRawPointer?
) {
    let configBox: Box<SCContentSharingPickerConfiguration> = unretained(config)
    let configValue = configBox.value
    let scStream: SCStream = unretained(streamPtr)
    let contentStyle = pickerContentStyle(from: style)

    presentOnMain(callback, userData) {
        let observer = PickerObserverWithResult(callback: callback, userData: userData)
        let picker = installOneShotObserver(observer)
        picker.setConfiguration(configValue, for: scStream)
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

#endif
