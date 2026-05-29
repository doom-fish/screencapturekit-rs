// Strided CGImage render helpers.
//
// The tightly-packed renderers (cgimage_render_rgba_into /
// cgimage_render_bgra_into) live in apple-cf's CoreGraphicsBridge and hard-code
// bytesPerRow = width * 4. Consumers with row-aligned destination buffers
// (GPU upload, wgpu, Metal) need to specify their own row stride. These
// variants accept an explicit destBytesPerRow and build the CGContext with it.

import CoreGraphics
import Foundation

// MARK: - Strided CGImage Bridge

@_cdecl("cgimage_render_rgba_into_strided")
public func renderCGImageRGBAIntoStrided(
    _ image: OpaquePointer,
    _ destBuffer: UnsafeMutableRawPointer,
    _ destCapacity: Int,
    _ destBytesPerRow: Int
) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()

    let width = cgImage.width
    let height = cgImage.height
    let bytesPerPixel = 4 // RGBA
    let minBytesPerRow = width * bytesPerPixel

    // The stride must hold at least one full row of pixels, and the caller's
    // buffer must span every (possibly padded) row. Mirror the bounds checks
    // performed Rust-side so a mismatch aborts rather than overflows.
    guard destBytesPerRow >= minBytesPerRow else {
        return 0
    }
    let (rowSpan, overflow) = height.multipliedReportingOverflow(by: destBytesPerRow)
    guard !overflow, rowSpan <= destCapacity else {
        return 0
    }
    let totalBytes = rowSpan

    let colorSpace = CGColorSpaceCreateDeviceRGB()
    let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue

    guard let context = CGContext(
        data: destBuffer,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: destBytesPerRow,
        space: colorSpace,
        bitmapInfo: bitmapInfo
    ) else {
        return 0
    }

    // See cgimage_render_rgba_into in apple-cf — .copy keeps output
    // deterministic regardless of the destination buffer's prior contents.
    context.setBlendMode(.copy)
    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))

    return totalBytes
}

@_cdecl("cgimage_render_bgra_into_strided")
public func renderCGImageBGRAIntoStrided(
    _ image: OpaquePointer,
    _ destBuffer: UnsafeMutableRawPointer,
    _ destCapacity: Int,
    _ destBytesPerRow: Int
) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()

    let width = cgImage.width
    let height = cgImage.height
    let bytesPerPixel = 4 // BGRA
    let minBytesPerRow = width * bytesPerPixel

    guard destBytesPerRow >= minBytesPerRow else {
        return 0
    }
    let (rowSpan, overflow) = height.multipliedReportingOverflow(by: destBytesPerRow)
    guard !overflow, rowSpan <= destCapacity else {
        return 0
    }
    let totalBytes = rowSpan

    // Native ScreenCaptureKit layout: premultipliedFirst + byteOrder32Little =
    // BGRA, matching kCVPixelFormatType_32BGRA and skipping the channel swap.
    let colorSpace = CGColorSpaceCreateDeviceRGB()
    let bitmapInfo = CGImageAlphaInfo.premultipliedFirst.rawValue
        | CGBitmapInfo.byteOrder32Little.rawValue

    guard let context = CGContext(
        data: destBuffer,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: destBytesPerRow,
        space: colorSpace,
        bitmapInfo: bitmapInfo
    ) else {
        return 0
    }

    context.setBlendMode(.copy)
    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))

    return totalBytes
}
