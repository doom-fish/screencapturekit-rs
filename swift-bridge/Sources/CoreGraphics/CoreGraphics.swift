// CoreGraphics Bridge - CGRect, CGSize, CGPoint, CGImage

import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

// MARK: - CGRect Bridge

public struct CGRectBridge {
    public var x: Double
    public var y: Double
    public var width: Double
    public var height: Double

    public init(x: Double, y: Double, width: Double, height: Double) {
        self.x = x
        self.y = y
        self.width = width
        self.height = height
    }

    public init(rect: CGRect) {
        x = Double(rect.origin.x)
        y = Double(rect.origin.y)
        width = Double(rect.size.width)
        height = Double(rect.size.height)
    }

    public func toCGRect() -> CGRect {
        CGRect(x: x, y: y, width: width, height: height)
    }
}

// MARK: - CGSize Bridge

public struct CGSizeBridge {
    public var width: Double
    public var height: Double

    public init(width: Double, height: Double) {
        self.width = width
        self.height = height
    }

    public init(size: CGSize) {
        width = Double(size.width)
        height = Double(size.height)
    }

    public func toCGSize() -> CGSize {
        CGSize(width: width, height: height)
    }
}

// MARK: - CGPoint Bridge

public struct CGPointBridge {
    public var x: Double
    public var y: Double

    public init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }

    public init(point: CGPoint) {
        x = Double(point.x)
        y = Double(point.y)
    }

    public func toCGPoint() -> CGPoint {
        CGPoint(x: x, y: y)
    }
}

// MARK: - CGImage Bridge

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

@_cdecl("cgimage_retain")
public func retainCGImage(_ image: OpaquePointer) -> OpaquePointer {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return OpaquePointer(Unmanaged.passRetained(cgImage).toOpaque())
}

@_cdecl("cgimage_get_data")
public func getCGImageData(_ image: OpaquePointer, _ outPtr: UnsafeMutablePointer<UnsafeRawPointer?>, _ outLength: UnsafeMutablePointer<Int>) -> Bool {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()

    let width = cgImage.width
    let height = cgImage.height
    let bytesPerPixel = 4 // RGBA
    let bytesPerRow = width * bytesPerPixel
    let totalBytes = height * bytesPerRow

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

    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))

    guard let data = context.data else {
        return false
    }

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

@_cdecl("cgimage_save_png")
public func saveCGImageToPNG(_ image: OpaquePointer, _ pathPtr: UnsafePointer<CChar>) -> Bool {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    let path = String(cString: pathPtr)
    let url = URL(fileURLWithPath: path)

    guard let destination = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else {
        return false
    }

    CGImageDestinationAddImage(destination, cgImage, nil)
    return CGImageDestinationFinalize(destination)
}

@_cdecl("cgimage_hash")
public func cgimageHash(_ image: OpaquePointer) -> Int {
    let cgImage = Unmanaged<CGImage>.fromOpaque(UnsafeRawPointer(image)).takeUnretainedValue()
    return cgImage.hashValue
}
