// Media APIs - CMSampleBuffer, CVPixelBuffer

import CoreGraphics
import CoreMedia
import CoreVideo
import Foundation
import IOSurface

// MARK: - Media: CMSampleBuffer

@_cdecl("cm_sample_buffer_get_image_buffer")
public func getSampleBufferImageBuffer(_ sampleBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    guard let imageBuffer = CMSampleBufferGetImageBuffer(buffer) else { return nil }
    return OpaquePointer(Unmanaged.passRetained(imageBuffer as AnyObject).toOpaque())
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp")
public func getSampleBufferPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_get_decode_timestamp")
public func getSampleBufferDecodeTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetDecodeTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_get_output_presentation_timestamp")
public func getSampleBufferOutputPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMSampleBufferGetOutputPresentationTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_set_output_presentation_timestamp")
public func setSampleBufferOutputPresentationTimestamp(
    _ sampleBuffer: OpaquePointer,
    _ value: Int64,
    _ timescale: Int32,
    _ flags: UInt32,
    _ epoch: Int64
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    let time = CMTime(value: CMTimeValue(value), timescale: timescale, flags: CMTimeFlags(rawValue: flags), epoch: epoch)
    return CMSampleBufferSetOutputPresentationTimeStamp(buffer, newValue: time)
}

@_cdecl("cm_sample_buffer_get_format_description")
public func getSampleBufferFormatDescription(_ sampleBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    guard let formatDesc = CMSampleBufferGetFormatDescription(buffer) else { return nil }
    return OpaquePointer(Unmanaged.passRetained(formatDesc as AnyObject).toOpaque())
}

@_cdecl("cm_sample_buffer_get_sample_size")
public func getSampleBufferSampleSize(_ sampleBuffer: OpaquePointer, _ sampleIndex: Int) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferGetSampleSize(buffer, at: sampleIndex)
}

@_cdecl("cm_sample_buffer_get_total_sample_size")
public func getSampleBufferTotalSampleSize(_ sampleBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferGetTotalSampleSize(buffer)
}

@_cdecl("cm_sample_buffer_is_ready_for_data_access")
public func isSampleBufferReadyForDataAccess(_ sampleBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferDataIsReady(buffer)
}

@_cdecl("cm_sample_buffer_make_data_ready")
public func makeSampleBufferDataReady(_ sampleBuffer: OpaquePointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(UnsafeRawPointer(sampleBuffer)).takeUnretainedValue()
    return CMSampleBufferMakeDataReady(buffer)
}

@_cdecl("cm_sample_buffer_release")
public func releaseSampleBuffer(_ sampleBuffer: OpaquePointer) {
    // Release the retained CMSampleBuffer that was passed from didOutputSampleBuffer
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(sampleBuffer)).release()
}

// MARK: - Media: CVPixelBuffer

@_cdecl("cv_pixel_buffer_get_width")
public func getPixelBufferWidth(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetWidth(buffer)
}

@_cdecl("cv_pixel_buffer_get_height")
public func getPixelBufferHeight(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetHeight(buffer)
}

@_cdecl("cv_pixel_buffer_get_pixel_format_type")
public func getPixelBufferPixelFormatType(_ pixelBuffer: OpaquePointer) -> UInt32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetPixelFormatType(buffer)
}

@_cdecl("cv_pixel_buffer_lock_base_address")
public func lockPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer, _ flags: UInt64) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: flags))
}

@_cdecl("cv_pixel_buffer_unlock_base_address")
public func unlockPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer, _ flags: UInt64) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: flags))
}

@_cdecl("cv_pixel_buffer_get_base_address")
public func getPixelBufferBaseAddress(_ pixelBuffer: OpaquePointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBaseAddress(buffer)
}

@_cdecl("cv_pixel_buffer_get_bytes_per_row")
public func getPixelBufferBytesPerRow(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBytesPerRow(buffer)
}

@_cdecl("cv_pixel_buffer_release")
public func releasePixelBuffer(_ pixelBuffer: OpaquePointer) {
    release(pixelBuffer)
}

// MARK: - Output: IOSurface

@_cdecl("cv_pixel_buffer_get_iosurface")
public func getPixelBufferIOSurface(_ pixelBuffer: OpaquePointer) -> OpaquePointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    guard let ioSurface = CVPixelBufferGetIOSurface(buffer)?.takeUnretainedValue() else {
        return nil
    }
    return OpaquePointer(Unmanaged.passRetained(ioSurface).toOpaque())
}

@_cdecl("cv_pixel_buffer_is_backed_by_iosurface")
public func isPixelBufferBackedByIOSurface(_ pixelBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetIOSurface(buffer) != nil
}

@_cdecl("cv_pixel_buffer_retain")
public func retainPixelBuffer(_ pixelBuffer: OpaquePointer) -> OpaquePointer {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue()
    return OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
}

// MARK: - CVPixelBuffer Creation

@_cdecl("cv_pixel_buffer_create")
public func createPixelBuffer(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ pixelBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    var pixelBuffer: CVPixelBuffer?
    let status = CVPixelBufferCreate(
        kCFAllocatorDefault,
        width,
        height,
        OSType(pixelFormatType),
        nil,
        &pixelBuffer
    )

    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        pixelBufferOut.pointee = nil
    }

    return status
}

@_cdecl("cv_pixel_buffer_create_with_bytes")
public func createPixelBufferWithBytes(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ baseAddress: UnsafeMutableRawPointer,
    _ bytesPerRow: Int,
    _ pixelBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    var pixelBuffer: CVPixelBuffer?
    let status = CVPixelBufferCreateWithBytes(
        kCFAllocatorDefault,
        width,
        height,
        OSType(pixelFormatType),
        baseAddress,
        bytesPerRow,
        nil,
        nil,
        nil,
        &pixelBuffer
    )

    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        pixelBufferOut.pointee = nil
    }

    return status
}

@_cdecl("cv_pixel_buffer_fill_extended_pixels")
public func fillPixelBufferExtendedPixels(_ pixelBuffer: OpaquePointer) -> Int32 {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferFillExtendedPixels(buffer)
}

@_cdecl("cv_pixel_buffer_get_data_size")
public func getPixelBufferDataSize(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetDataSize(buffer)
}

@_cdecl("cv_pixel_buffer_is_planar")
public func isPixelBufferPlanar(_ pixelBuffer: OpaquePointer) -> Bool {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferIsPlanar(buffer)
}

@_cdecl("cv_pixel_buffer_get_plane_count")
public func getPixelBufferPlaneCount(_ pixelBuffer: OpaquePointer) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetPlaneCount(buffer)
}

@_cdecl("cv_pixel_buffer_get_width_of_plane")
public func getPixelBufferWidthOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetWidthOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_height_of_plane")
public func getPixelBufferHeightOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetHeightOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_base_address_of_plane")
public func getPixelBufferBaseAddressOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBaseAddressOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_bytes_per_row_of_plane")
public func getPixelBufferBytesPerRowOfPlane(_ pixelBuffer: OpaquePointer, _ planeIndex: Int) -> Int {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    return CVPixelBufferGetBytesPerRowOfPlane(buffer, planeIndex)
}

@_cdecl("cv_pixel_buffer_get_extended_pixels")
public func getPixelBufferExtendedPixels(
    _ pixelBuffer: OpaquePointer,
    _ extraColumnsOnLeft: UnsafeMutablePointer<Int>,
    _ extraColumnsOnRight: UnsafeMutablePointer<Int>,
    _ extraRowsOnTop: UnsafeMutablePointer<Int>,
    _ extraRowsOnBottom: UnsafeMutablePointer<Int>
) {
    let buffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(pixelBuffer)).takeUnretainedValue() as! CVPixelBuffer
    CVPixelBufferGetExtendedPixels(buffer,
                                   extraColumnsOnLeft,
                                   extraColumnsOnRight,
                                   extraRowsOnTop,
                                   extraRowsOnBottom)
}

// MARK: - CMSampleBuffer Creation

@_cdecl("cm_sample_buffer_create_for_image_buffer")
public func createSampleBufferForImageBuffer(
    _ imageBuffer: OpaquePointer,
    _ presentationTimeValue: Int64,
    _ presentationTimeScale: Int32,
    _ durationValue: Int64,
    _ durationScale: Int32,
    _ sampleBufferOut: UnsafeMutablePointer<OpaquePointer?>
) -> Int32 {
    let pixelBuffer = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(imageBuffer)).takeUnretainedValue() as! CVPixelBuffer

    var sampleBuffer: CMSampleBuffer?
    var timingInfo = CMSampleTimingInfo(
        duration: CMTime(value: CMTimeValue(durationValue), timescale: durationScale, flags: .valid, epoch: 0),
        presentationTimeStamp: CMTime(value: CMTimeValue(presentationTimeValue), timescale: presentationTimeScale, flags: .valid, epoch: 0),
        decodeTimeStamp: .invalid
    )

    var formatDescription: CMFormatDescription?
    let descStatus = CMVideoFormatDescriptionCreateForImageBuffer(
        allocator: kCFAllocatorDefault,
        imageBuffer: pixelBuffer,
        formatDescriptionOut: &formatDescription
    )

    guard descStatus == noErr, let format = formatDescription else {
        sampleBufferOut.pointee = nil
        return descStatus
    }

    let status = CMSampleBufferCreateReadyWithImageBuffer(
        allocator: kCFAllocatorDefault,
        imageBuffer: pixelBuffer,
        formatDescription: format,
        sampleTiming: &timingInfo,
        sampleBufferOut: &sampleBuffer
    )

    if status == noErr, let buffer = sampleBuffer {
        sampleBufferOut.pointee = OpaquePointer(Unmanaged.passRetained(buffer).toOpaque())
    } else {
        sampleBufferOut.pointee = nil
    }

    return status
}

@_cdecl("iosurface_get_width")
public func getIOSurfaceWidth(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetWidth(surface)
}

@_cdecl("iosurface_get_height")
public func getIOSurfaceHeight(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetHeight(surface)
}

@_cdecl("iosurface_get_bytes_per_row")
public func getIOSurfaceBytesPerRow(_ ioSurface: OpaquePointer) -> Int {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetBytesPerRow(surface)
}

@_cdecl("iosurface_get_pixel_format")
public func getIOSurfacePixelFormat(_ ioSurface: OpaquePointer) -> UInt32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetPixelFormat(surface)
}

@_cdecl("iosurface_get_base_address")
public func getIOSurfaceBaseAddress(_ ioSurface: OpaquePointer) -> UnsafeMutableRawPointer? {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceGetBaseAddress(surface)
}

@_cdecl("iosurface_lock")
public func lockIOSurface(_ ioSurface: OpaquePointer, options: UInt32) -> Int32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceLock(surface, IOSurfaceLockOptions(rawValue: options), nil)
}

@_cdecl("iosurface_unlock")
public func unlockIOSurface(_ ioSurface: OpaquePointer, options: UInt32) -> Int32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceUnlock(surface, IOSurfaceLockOptions(rawValue: options), nil)
}

@_cdecl("iosurface_is_in_use")
public func isIOSurfaceInUse(_ ioSurface: OpaquePointer) -> Bool {
    let surface = Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).takeUnretainedValue()
    return IOSurfaceIsInUse(surface)
}

@_cdecl("iosurface_release")
public func releaseIOSurface(_ ioSurface: OpaquePointer) {
    Unmanaged<IOSurface>.fromOpaque(UnsafeRawPointer(ioSurface)).release()
}
