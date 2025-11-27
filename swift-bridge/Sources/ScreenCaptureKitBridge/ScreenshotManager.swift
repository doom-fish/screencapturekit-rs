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

@_cdecl("cgimage_get_width")
public func getCGImageWidth(_ image: OpaquePointer) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return cgImage.width
}

@_cdecl("cgimage_get_height")
public func getCGImageHeight(_ image: OpaquePointer) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return cgImage.height
}

@_cdecl("cgimage_release")
public func releaseCGImage(_ image: OpaquePointer) {
    Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).release()
}

@_cdecl("cgimage_get_data")
public func getCGImageData(_ image: OpaquePointer, _ outPtr: UnsafeMutablePointer<UnsafeRawPointer?>, _ outLength: UnsafeMutablePointer<Int>) -> Bool {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()

    let width = cgImage.width
    let height = cgImage.height
    let bytesPerPixel = 4  // RGBA
    let bytesPerRow = width * bytesPerPixel
    let totalBytes = height * bytesPerRow

    // Create a bitmap context to draw the image
    let colorSpace = CGColorSpaceCreateDeviceRGB()
    let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue

    guard let context = CGContext(
        data: nil,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: bytesPerRow,
        space: colorSpace,
        bitmapInfo: bitmapInfo
    ) else {
        return false
    }

    // Draw the image into the context
    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))

    // Get the data
    guard let data = context.data else {
        return false
    }

    // Allocate memory for the data and copy it
    let buffer = UnsafeMutableRawPointer.allocate(byteCount: totalBytes, alignment: 1)
    buffer.copyMemory(from: data, byteCount: totalBytes)

    outPtr.pointee = UnsafeRawPointer(buffer)
    outLength.pointee = totalBytes

    return true
}

@_cdecl("cgimage_free_data")
public func freeCGImageData(_ ptr: UnsafeMutableRawPointer) {
    ptr.deallocate()
}
