// Screenshot Manager APIs (macOS 14.0+)

import CoreGraphics
import CoreMedia
import Foundation
import ScreenCaptureKit

// MARK: - Screenshot Manager (macOS 14.0+)

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_image")
public func captureScreenshot(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let image = try await SCScreenshotManager.captureImage(
                contentFilter: filter,
                configuration: configuration
            )
            callback(retain(image), nil, userData)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0, userData) }
        }
    }
}

@available(macOS 14.0, *)
@_cdecl("sc_screenshot_manager_capture_sample_buffer")
public func captureScreenshotSampleBuffer(
    _ contentFilter: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    let filter: SCContentFilter = unretained(contentFilter)
    let configuration: SCStreamConfiguration = unretained(config)

    Task {
        do {
            let sampleBuffer = try await SCScreenshotManager.captureSampleBuffer(
                contentFilter: filter,
                configuration: configuration
            )
            let retained = Unmanaged.passRetained(sampleBuffer as AnyObject)
            callback(OpaquePointer(retained.toOpaque()), nil, userData)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(nil, $0, userData) }
        }
    }
}
