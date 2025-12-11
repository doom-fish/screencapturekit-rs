// IOSurface Bridge

import Foundation
import IOSurface

// MARK: - IOSurface Bridge

@_cdecl("io_surface_get_width")
public func io_surface_get_width(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetWidth(ioSurface)
}

@_cdecl("io_surface_get_height")
public func io_surface_get_height(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetHeight(ioSurface)
}

@_cdecl("io_surface_get_bytes_per_row")
public func io_surface_get_bytes_per_row(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBytesPerRow(ioSurface)
}

@_cdecl("io_surface_get_pixel_format")
public func io_surface_get_pixel_format(_ surface: UnsafeMutableRawPointer) -> UInt32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetPixelFormat(ioSurface)
}

@_cdecl("io_surface_get_base_address")
public func io_surface_get_base_address(_ surface: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBaseAddress(ioSurface)
}

@_cdecl("io_surface_lock")
public func io_surface_lock(_ surface: UnsafeMutableRawPointer, _ options: UInt32, _ seedOut: UnsafeMutablePointer<UInt32>?) -> Int32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceLock(ioSurface, IOSurfaceLockOptions(rawValue: options), seedOut)
}

@_cdecl("io_surface_unlock")
public func io_surface_unlock(_ surface: UnsafeMutableRawPointer, _ options: UInt32, _ seedOut: UnsafeMutablePointer<UInt32>?) -> Int32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceUnlock(ioSurface, IOSurfaceLockOptions(rawValue: options), seedOut)
}

@_cdecl("io_surface_is_in_use")
public func io_surface_is_in_use(_ surface: UnsafeMutableRawPointer) -> Bool {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceIsInUse(ioSurface)
}

@_cdecl("io_surface_release")
public func io_surface_release(_ surface: UnsafeMutableRawPointer) {
    Unmanaged<IOSurface>.fromOpaque(surface).release()
}

@_cdecl("io_surface_retain")
public func io_surface_retain(_ surface: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return Unmanaged.passRetained(ioSurface).toOpaque()
}

// MARK: - Compatibility Aliases (deprecated - use io_surface_* functions)

@_cdecl("iosurface_get_width")
public func iosurface_get_width(_ surface: UnsafeMutableRawPointer) -> Int {
    io_surface_get_width(surface)
}

@_cdecl("iosurface_get_height")
public func iosurface_get_height(_ surface: UnsafeMutableRawPointer) -> Int {
    io_surface_get_height(surface)
}

@_cdecl("iosurface_get_bytes_per_row")
public func iosurface_get_bytes_per_row(_ surface: UnsafeMutableRawPointer) -> Int {
    io_surface_get_bytes_per_row(surface)
}

@_cdecl("iosurface_get_pixel_format")
public func iosurface_get_pixel_format(_ surface: UnsafeMutableRawPointer) -> UInt32 {
    io_surface_get_pixel_format(surface)
}

@_cdecl("iosurface_get_base_address")
public func iosurface_get_base_address(_ surface: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    io_surface_get_base_address(surface)
}

@_cdecl("iosurface_lock")
public func iosurface_lock(_ surface: UnsafeMutableRawPointer, _ options: UInt32) -> Int32 {
    io_surface_lock(surface, options, nil)
}

@_cdecl("iosurface_unlock")
public func iosurface_unlock(_ surface: UnsafeMutableRawPointer, _ options: UInt32) -> Int32 {
    io_surface_unlock(surface, options, nil)
}

@_cdecl("iosurface_is_in_use")
public func iosurface_is_in_use(_ surface: UnsafeMutableRawPointer) -> Bool {
    io_surface_is_in_use(surface)
}

@_cdecl("iosurface_release")
public func iosurface_release(_ surface: UnsafeMutableRawPointer) {
    io_surface_release(surface)
}

// MARK: - Plane Functions (for multi-planar formats like YCbCr)

@_cdecl("io_surface_get_plane_count")
public func io_surface_get_plane_count(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetPlaneCount(ioSurface)
}

@_cdecl("io_surface_get_width_of_plane")
public func io_surface_get_width_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetWidthOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_height_of_plane")
public func io_surface_get_height_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetHeightOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_bytes_per_row_of_plane")
public func io_surface_get_bytes_per_row_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBytesPerRowOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_bytes_per_element_of_plane")
public func io_surface_get_bytes_per_element_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBytesPerElementOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_element_width_of_plane")
public func io_surface_get_element_width_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetElementWidthOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_element_height_of_plane")
public func io_surface_get_element_height_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetElementHeightOfPlane(ioSurface, plane)
}

@_cdecl("io_surface_get_base_address_of_plane")
public func io_surface_get_base_address_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> UnsafeMutableRawPointer? {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBaseAddressOfPlane(ioSurface, plane)
}

// Compatibility aliases for plane functions
@_cdecl("iosurface_get_plane_count")
public func iosurface_get_plane_count(_ surface: UnsafeMutableRawPointer) -> Int {
    io_surface_get_plane_count(surface)
}

@_cdecl("iosurface_get_width_of_plane")
public func iosurface_get_width_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    io_surface_get_width_of_plane(surface, plane)
}

@_cdecl("iosurface_get_height_of_plane")
public func iosurface_get_height_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    io_surface_get_height_of_plane(surface, plane)
}

@_cdecl("iosurface_get_bytes_per_row_of_plane")
public func iosurface_get_bytes_per_row_of_plane(_ surface: UnsafeMutableRawPointer, _ plane: Int) -> Int {
    io_surface_get_bytes_per_row_of_plane(surface, plane)
}

// MARK: - Hash Functions

@_cdecl("io_surface_hash")
public func io_surface_hash(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return ioSurface.hashValue
}

// MARK: - Additional IOSurface Functions

@_cdecl("io_surface_get_alloc_size")
public func io_surface_get_alloc_size(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetAllocSize(ioSurface)
}

@_cdecl("io_surface_get_id")
public func io_surface_get_id(_ surface: UnsafeMutableRawPointer) -> UInt32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetID(ioSurface)
}

@_cdecl("io_surface_get_seed")
public func io_surface_get_seed(_ surface: UnsafeMutableRawPointer) -> UInt32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetSeed(ioSurface)
}

@_cdecl("io_surface_get_bytes_per_element")
public func io_surface_get_bytes_per_element(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetBytesPerElement(ioSurface)
}

@_cdecl("io_surface_get_element_width")
public func io_surface_get_element_width(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetElementWidth(ioSurface)
}

@_cdecl("io_surface_get_element_height")
public func io_surface_get_element_height(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceGetElementHeight(ioSurface)
}

@_cdecl("io_surface_increment_use_count")
public func io_surface_increment_use_count(_ surface: UnsafeMutableRawPointer) {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    IOSurfaceIncrementUseCount(ioSurface)
}

@_cdecl("io_surface_decrement_use_count")
public func io_surface_decrement_use_count(_ surface: UnsafeMutableRawPointer) {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    IOSurfaceDecrementUseCount(ioSurface)
}

// MARK: - IOSurface Creation (for testing)

/// Create an IOSurface with the given dimensions and pixel format
@_cdecl("io_surface_create")
public func io_surface_create(
    _ width: Int,
    _ height: Int,
    _ pixelFormat: UInt32,
    _ bytesPerElement: Int,
    _ surfaceOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let bytesPerRow = width * bytesPerElement
    
    let properties: [IOSurfacePropertyKey: Any] = [
        .width: width,
        .height: height,
        .bytesPerElement: bytesPerElement,
        .bytesPerRow: bytesPerRow,
        .allocSize: bytesPerRow * height,
        .pixelFormat: pixelFormat,
    ]
    
    guard let surface = IOSurface(properties: properties) else {
        surfaceOut.pointee = nil
        return -1
    }
    
    surfaceOut.pointee = Unmanaged.passRetained(surface).toOpaque()
    return 0
}
