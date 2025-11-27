// Content Sharing Picker APIs (macOS 14.0+)

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
