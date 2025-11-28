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

// MARK: - Capture image in rect (macOS 15.2+)

#if compiler(>=6.0)
@_cdecl("sc_screenshot_manager_capture_image_in_rect")
public func captureScreenshotInRect(
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    if #available(macOS 15.2, *) {
        let rect = CGRect(x: x, y: y, width: width, height: height)
        Task {
            do {
                let image = try await SCScreenshotManager.captureImage(in: rect)
                callback(retain(image), nil, userData)
            } catch {
                let errorMsg = error.localizedDescription
                errorMsg.withCString { callback(nil, $0, userData) }
            }
        }
    } else {
        "captureImageInRect requires macOS 15.2+".withCString { callback(nil, $0, userData) }
    }
}
#else
@_cdecl("sc_screenshot_manager_capture_image_in_rect")
public func captureScreenshotInRect(
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double,
    _ callback: @escaping @convention(c) (OpaquePointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    _ userData: UnsafeMutableRawPointer?
) {
    "captureImageInRect requires macOS 15.2+".withCString { callback(nil, $0, userData) }
}
#endif
