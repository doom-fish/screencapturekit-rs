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
public func io_surface_lock(_ surface: UnsafeMutableRawPointer, _ options: UInt32) -> Int32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceLock(ioSurface, IOSurfaceLockOptions(rawValue: options), nil)
}

@_cdecl("io_surface_unlock")
public func io_surface_unlock(_ surface: UnsafeMutableRawPointer, _ options: UInt32) -> Int32 {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return IOSurfaceUnlock(ioSurface, IOSurfaceLockOptions(rawValue: options), nil)
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
    io_surface_lock(surface, options)
}

@_cdecl("iosurface_unlock")
public func iosurface_unlock(_ surface: UnsafeMutableRawPointer, _ options: UInt32) -> Int32 {
    io_surface_unlock(surface, options)
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
