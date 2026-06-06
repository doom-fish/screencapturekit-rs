// CoreMedia Bridge - CMSampleBuffer, CMTime, CMFormatDescription, CMBlockBuffer

import CoreMedia
import CoreVideo
import Foundation
import ScreenCaptureKit
import VideoToolbox

// MARK: - Audio Buffer List Bridge Types

public struct AudioBufferBridge {
    public var number_channels: UInt32
    public var data_bytes_size: UInt32
    public var data_ptr: UnsafeMutableRawPointer?
}

public struct AudioBufferListRaw {
    public var num_buffers: UInt32
    public var buffers_ptr: UnsafeMutablePointer<AudioBufferBridge>?
    public var buffers_len: UInt
}

// MARK: - CMSampleBuffer Bridge


@_cdecl("cm_sample_buffer_get_frame_status")
public func cm_sample_buffer_get_frame_status(_ sampleBuffer: UnsafeMutableRawPointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let status = firstAttachment[SCStreamFrameInfo.status.rawValue as CFString] as? SCFrameStatus
    else {
        return -1
    }

    return Int32(status.rawValue)
}

@_cdecl("cm_sample_buffer_get_display_time")
public func cm_sample_buffer_get_display_time(_ sampleBuffer: UnsafeMutableRawPointer, _ outValue: UnsafeMutablePointer<UInt64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let displayTime = firstAttachment[SCStreamFrameInfo.displayTime.rawValue as CFString] as? UInt64
    else {
        return false
    }

    outValue.pointee = displayTime
    return true
}

@_cdecl("cm_sample_buffer_get_scale_factor")
public func cm_sample_buffer_get_scale_factor(_ sampleBuffer: UnsafeMutableRawPointer, _ outValue: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let scaleFactor = firstAttachment[SCStreamFrameInfo.scaleFactor.rawValue as CFString] as? Float64
    else {
        return false
    }

    outValue.pointee = scaleFactor
    return true
}

@_cdecl("cm_sample_buffer_get_content_scale")
public func cm_sample_buffer_get_content_scale(_ sampleBuffer: UnsafeMutableRawPointer, _ outValue: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let contentScale = firstAttachment[SCStreamFrameInfo.contentScale.rawValue as CFString] as? Float64
    else {
        return false
    }

    outValue.pointee = contentScale
    return true
}

@_cdecl("cm_sample_buffer_get_content_rect")
public func cm_sample_buffer_get_content_rect(_ sampleBuffer: UnsafeMutableRawPointer, _ outX: UnsafeMutablePointer<Float64>, _ outY: UnsafeMutablePointer<Float64>, _ outWidth: UnsafeMutablePointer<Float64>, _ outHeight: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let rectDict = firstAttachment[SCStreamFrameInfo.contentRect.rawValue as CFString] as? [String: Any],
          let rect = CGRect(dictionaryRepresentation: rectDict as CFDictionary)
    else {
        return false
    }

    outX.pointee = rect.origin.x
    outY.pointee = rect.origin.y
    outWidth.pointee = rect.size.width
    outHeight.pointee = rect.size.height
    return true
}

@available(macOS 14.0, *)
@_cdecl("cm_sample_buffer_get_bounding_rect")
public func cm_sample_buffer_get_bounding_rect(_ sampleBuffer: UnsafeMutableRawPointer, _ outX: UnsafeMutablePointer<Float64>, _ outY: UnsafeMutablePointer<Float64>, _ outWidth: UnsafeMutablePointer<Float64>, _ outHeight: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let rectDict = firstAttachment[SCStreamFrameInfo.boundingRect.rawValue as CFString] as? [String: Any],
          let rect = CGRect(dictionaryRepresentation: rectDict as CFDictionary)
    else {
        return false
    }

    outX.pointee = rect.origin.x
    outY.pointee = rect.origin.y
    outWidth.pointee = rect.size.width
    outHeight.pointee = rect.size.height
    return true
}

@available(macOS 13.1, *)
@_cdecl("cm_sample_buffer_get_screen_rect")
public func cm_sample_buffer_get_screen_rect(_ sampleBuffer: UnsafeMutableRawPointer, _ outX: UnsafeMutablePointer<Float64>, _ outY: UnsafeMutablePointer<Float64>, _ outWidth: UnsafeMutablePointer<Float64>, _ outHeight: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let rectDict = firstAttachment[SCStreamFrameInfo.screenRect.rawValue as CFString] as? [String: Any],
          let rect = CGRect(dictionaryRepresentation: rectDict as CFDictionary)
    else {
        return false
    }

    outX.pointee = rect.origin.x
    outY.pointee = rect.origin.y
    outWidth.pointee = rect.size.width
    outHeight.pointee = rect.size.height
    return true
}

/// Read the `SCStreamFrameInfo.presenterOverlayContentRect` attachment off the
/// sample buffer (macOS 14.2+ Presenter Overlay). Returns false (and leaves the
/// out parameters untouched) if the attachment is missing — typical when the
/// stream was not configured with a presenter overlay.
@available(macOS 14.2, *)
@_cdecl("cm_sample_buffer_get_presenter_overlay_content_rect")
public func cm_sample_buffer_get_presenter_overlay_content_rect(_ sampleBuffer: UnsafeMutableRawPointer, _ outX: UnsafeMutablePointer<Float64>, _ outY: UnsafeMutablePointer<Float64>, _ outWidth: UnsafeMutablePointer<Float64>, _ outHeight: UnsafeMutablePointer<Float64>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let rectDict = firstAttachment[SCStreamFrameInfo.presenterOverlayContentRect.rawValue as CFString] as? [String: Any],
          let rect = CGRect(dictionaryRepresentation: rectDict as CFDictionary)
    else {
        return false
    }

    outX.pointee = rect.origin.x
    outY.pointee = rect.origin.y
    outWidth.pointee = rect.size.width
    outHeight.pointee = rect.size.height
    return true
}

@_cdecl("cm_sample_buffer_get_dirty_rects")
public func cm_sample_buffer_get_dirty_rects(_ sampleBuffer: UnsafeMutableRawPointer, _ outRects: UnsafeMutablePointer<UnsafeMutableRawPointer?>, _ outCount: UnsafeMutablePointer<UInt>) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let dirtyRects = firstAttachment[SCStreamFrameInfo.dirtyRects.rawValue as CFString] as? [Any]
    else {
        outRects.pointee = nil
        outCount.pointee = 0
        return false
    }

    var rects: [CGRect] = []
    for item in dirtyRects {
        if let rectDict = item as? [String: Any],
           let rect = CGRect(dictionaryRepresentation: rectDict as CFDictionary)
        {
            rects.append(rect)
        }
    }

    guard !rects.isEmpty else {
        outRects.pointee = nil
        outCount.pointee = 0
        return false
    }

    // Allocate array of 4 doubles per rect (x, y, width, height)
    let rectsPtr = UnsafeMutablePointer<Float64>.allocate(capacity: rects.count * 4)
    for (index, rect) in rects.enumerated() {
        rectsPtr[index * 4 + 0] = rect.origin.x
        rectsPtr[index * 4 + 1] = rect.origin.y
        rectsPtr[index * 4 + 2] = rect.size.width
        rectsPtr[index * 4 + 3] = rect.size.height
    }

    outRects.pointee = UnsafeMutableRawPointer(rectsPtr)
    outCount.pointee = UInt(rects.count)
    return true
}

@_cdecl("cm_sample_buffer_free_dirty_rects")
public func cm_sample_buffer_free_dirty_rects(_ rectsPtr: UnsafeMutableRawPointer) {
    rectsPtr.deallocate()
}

// Bit flags identifying which fields the batched frame_info FFI managed to
// populate. Keep in sync with `FrameInfoFields` in src/cm/ffi.rs.
private struct FrameInfoFieldBits {
    static let status: UInt32 = 1 << 0
    static let displayTime: UInt32 = 1 << 1
    static let scaleFactor: UInt32 = 1 << 2
    static let contentScale: UInt32 = 1 << 3
    static let contentRect: UInt32 = 1 << 4
    static let boundingRect: UInt32 = 1 << 5
    static let screenRect: UInt32 = 1 << 6
    static let presenterOverlayRect: UInt32 = 1 << 7
}

// Single-call frame info fetch.
//
// Replaces 5+ separate `cm_sample_buffer_get_*` calls that each invoked
// `CMSampleBufferGetSampleAttachmentsArray` and re-bridged the dictionary
// from CF → Swift. Measured at ~11.4 µs/frame with the old per-attribute
// path; this collapses to one attachment fetch + one Swift bridging cast
// (~3 µs) by reading every documented `SCStreamFrameInfo` key into a single
// `repr(C)` out struct.
//
// `presenterOverlayContentRect` is read on macOS 14.2+ only; the field stays
// at its default value and the corresponding bit is left clear on older
// systems.
//
// Layout MUST match `FrameInfoRaw` in src/cm/ffi.rs.
@_cdecl("cm_sample_buffer_get_frame_info")
public func cm_sample_buffer_get_frame_info(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ outFields: UnsafeMutablePointer<UInt32>,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outDisplayTime: UnsafeMutablePointer<UInt64>,
    _ outScaleFactor: UnsafeMutablePointer<Float64>,
    _ outContentScale: UnsafeMutablePointer<Float64>,
    _ outContentRect: UnsafeMutablePointer<Float64>,        // [4]: x,y,w,h
    _ outBoundingRect: UnsafeMutablePointer<Float64>,       // [4]
    _ outScreenRect: UnsafeMutablePointer<Float64>,         // [4]
    _ outPresenterOverlayRect: UnsafeMutablePointer<Float64> // [4]
) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    outFields.pointee = 0

    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let attachment = attachments.first
    else {
        return false
    }

    var fields: UInt32 = 0

    if let status = attachment[SCStreamFrameInfo.status.rawValue as CFString] as? SCFrameStatus {
        outStatus.pointee = Int32(status.rawValue)
        fields |= FrameInfoFieldBits.status
    }
    if let displayTime = attachment[SCStreamFrameInfo.displayTime.rawValue as CFString] as? UInt64 {
        outDisplayTime.pointee = displayTime
        fields |= FrameInfoFieldBits.displayTime
    }
    if let scaleFactor = attachment[SCStreamFrameInfo.scaleFactor.rawValue as CFString] as? Float64 {
        outScaleFactor.pointee = scaleFactor
        fields |= FrameInfoFieldBits.scaleFactor
    }
    if let contentScale = attachment[SCStreamFrameInfo.contentScale.rawValue as CFString] as? Float64 {
        outContentScale.pointee = contentScale
        fields |= FrameInfoFieldBits.contentScale
    }
    if let dict = attachment[SCStreamFrameInfo.contentRect.rawValue as CFString] as? [String: Any],
       let rect = CGRect(dictionaryRepresentation: dict as CFDictionary) {
        outContentRect[0] = rect.origin.x
        outContentRect[1] = rect.origin.y
        outContentRect[2] = rect.size.width
        outContentRect[3] = rect.size.height
        fields |= FrameInfoFieldBits.contentRect
    }
    if #available(macOS 14.0, *),
       let dict = attachment[SCStreamFrameInfo.boundingRect.rawValue as CFString] as? [String: Any],
       let rect = CGRect(dictionaryRepresentation: dict as CFDictionary) {
        outBoundingRect[0] = rect.origin.x
        outBoundingRect[1] = rect.origin.y
        outBoundingRect[2] = rect.size.width
        outBoundingRect[3] = rect.size.height
        fields |= FrameInfoFieldBits.boundingRect
    }
    if #available(macOS 13.1, *),
       let dict = attachment[SCStreamFrameInfo.screenRect.rawValue as CFString] as? [String: Any],
       let rect = CGRect(dictionaryRepresentation: dict as CFDictionary) {
        outScreenRect[0] = rect.origin.x
        outScreenRect[1] = rect.origin.y
        outScreenRect[2] = rect.size.width
        outScreenRect[3] = rect.size.height
        fields |= FrameInfoFieldBits.screenRect
    }
    if #available(macOS 14.2, *),
       let dict = attachment[SCStreamFrameInfo.presenterOverlayContentRect.rawValue as CFString] as? [String: Any],
       let rect = CGRect(dictionaryRepresentation: dict as CFDictionary) {
        outPresenterOverlayRect[0] = rect.origin.x
        outPresenterOverlayRect[1] = rect.origin.y
        outPresenterOverlayRect[2] = rect.size.width
        outPresenterOverlayRect[3] = rect.size.height
        fields |= FrameInfoFieldBits.presenterOverlayRect
    }

    outFields.pointee = fields
    return fields != 0
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp_value")
public func cm_sample_buffer_get_presentation_timestamp_value(_ sampleBuffer: UnsafeMutableRawPointer) -> Int64 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    return time.value
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp_timescale")
public func cm_sample_buffer_get_presentation_timestamp_timescale(_ sampleBuffer: UnsafeMutableRawPointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    return time.timescale
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp_flags")
public func cm_sample_buffer_get_presentation_timestamp_flags(_ sampleBuffer: UnsafeMutableRawPointer) -> UInt32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    return time.flags.rawValue
}

@_cdecl("cm_sample_buffer_get_presentation_timestamp_epoch")
public func cm_sample_buffer_get_presentation_timestamp_epoch(_ sampleBuffer: UnsafeMutableRawPointer) -> Int64 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    return time.epoch
}



@_cdecl("cm_sample_buffer_get_output_presentation_timestamp")
public func cm_sample_buffer_get_output_presentation_timestamp(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetOutputPresentationTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_set_output_presentation_timestamp")
public func cm_sample_buffer_set_output_presentation_timestamp(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ value: Int64,
    _ timescale: Int32,
    _ flags: UInt32,
    _ epoch: Int64
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMTime(value: CMTimeValue(value), timescale: timescale, flags: CMTimeFlags(rawValue: flags), epoch: epoch)
    return CMSampleBufferSetOutputPresentationTimeStamp(buffer, newValue: time)
}

@_cdecl("cm_sample_buffer_get_duration_value")
public func cm_sample_buffer_get_duration_value(_ sampleBuffer: UnsafeMutableRawPointer) -> Int64 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let duration = CMSampleBufferGetDuration(buffer)
    return duration.value
}

@_cdecl("cm_sample_buffer_get_duration_timescale")
public func cm_sample_buffer_get_duration_timescale(_ sampleBuffer: UnsafeMutableRawPointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let duration = CMSampleBufferGetDuration(buffer)
    return duration.timescale
}

@_cdecl("cm_sample_buffer_get_duration_flags")
public func cm_sample_buffer_get_duration_flags(_ sampleBuffer: UnsafeMutableRawPointer) -> UInt32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let duration = CMSampleBufferGetDuration(buffer)
    return duration.flags.rawValue
}

@_cdecl("cm_sample_buffer_get_duration_epoch")
public func cm_sample_buffer_get_duration_epoch(_ sampleBuffer: UnsafeMutableRawPointer) -> Int64 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let duration = CMSampleBufferGetDuration(buffer)
    return duration.epoch
}






@_cdecl("cm_sample_buffer_get_sample_size")
public func cm_sample_buffer_get_sample_size(_ sampleBuffer: UnsafeMutableRawPointer, _ sampleIndex: Int) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferGetSampleSize(buffer, at: sampleIndex)
}

@_cdecl("cm_sample_buffer_get_total_sample_size")
public func cm_sample_buffer_get_total_sample_size(_ sampleBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferGetTotalSampleSize(buffer)
}

@_cdecl("cm_sample_buffer_is_ready_for_data_access")
public func cm_sample_buffer_is_ready_for_data_access(_ sampleBuffer: UnsafeMutableRawPointer) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferDataIsReady(buffer)
}

@_cdecl("cm_sample_buffer_make_data_ready")
public func cm_sample_buffer_make_data_ready(_ sampleBuffer: UnsafeMutableRawPointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferMakeDataReady(buffer)
}

// MARK: - Audio Buffer List Bridge

@_cdecl("cm_sample_buffer_get_audio_buffer_list_num_buffers")
public func cm_sample_buffer_get_audio_buffer_list_num_buffers(_ sampleBuffer: UnsafeMutableRawPointer) -> UInt32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    var blockBuffer: CMBlockBuffer?
    var audioBufferList = AudioBufferList()

    let status = CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
        buffer,
        bufferListSizeNeededOut: nil,
        bufferListOut: &audioBufferList,
        bufferListSize: MemoryLayout<AudioBufferList>.size,
        blockBufferAllocator: nil,
        blockBufferMemoryAllocator: nil,
        flags: 0,
        blockBufferOut: &blockBuffer
    )

    return status == noErr ? audioBufferList.mNumberBuffers : 0
}

@_cdecl("cm_sample_buffer_get_audio_buffer_list")
public func cm_sample_buffer_get_audio_buffer_list(_ sampleBuffer: UnsafeMutableRawPointer, _ outNumBuffers: UnsafeMutablePointer<UInt32>, _ outBuffersPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>, _ outBuffersLen: UnsafeMutablePointer<UInt>, _ outBlockBuffer: UnsafeMutablePointer<UnsafeMutableRawPointer?>) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()

    // First, query the required buffer size
    var bufferListSizeNeeded = 0
    var status = CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
        buffer,
        bufferListSizeNeededOut: &bufferListSizeNeeded,
        bufferListOut: nil,
        bufferListSize: 0,
        blockBufferAllocator: nil,
        blockBufferMemoryAllocator: nil,
        flags: 0,
        blockBufferOut: nil
    )

    guard bufferListSizeNeeded > 0 else {
        outNumBuffers.pointee = 0
        outBuffersPtr.pointee = nil
        outBuffersLen.pointee = 0
        outBlockBuffer.pointee = nil
        return
    }

    // Allocate buffer of the required size
    let audioBufferListPtr = UnsafeMutablePointer<AudioBufferList>.allocate(capacity: bufferListSizeNeeded / MemoryLayout<AudioBufferList>.stride + 1)
    defer { audioBufferListPtr.deallocate() }

    var blockBuffer: CMBlockBuffer?
    status = CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
        buffer,
        bufferListSizeNeededOut: nil,
        bufferListOut: audioBufferListPtr,
        bufferListSize: bufferListSizeNeeded,
        blockBufferAllocator: nil,
        blockBufferMemoryAllocator: nil,
        flags: 0,
        blockBufferOut: &blockBuffer
    )

    guard status == noErr, let blockBuffer else {
        outNumBuffers.pointee = 0
        outBuffersPtr.pointee = nil
        outBuffersLen.pointee = 0
        outBlockBuffer.pointee = nil
        return
    }

    let numBuffers = Int(audioBufferListPtr.pointee.mNumberBuffers)
    guard numBuffers > 0 else {
        outNumBuffers.pointee = 0
        outBuffersPtr.pointee = nil
        outBuffersLen.pointee = 0
        outBlockBuffer.pointee = nil
        return
    }

    let buffers = UnsafeMutablePointer<AudioBufferBridge>.allocate(capacity: numBuffers)

    // Trust-boundary hardening: CoreMedia's reported mDataByteSize is copied across
    // the FFI boundary and used by Rust to build a slice via from_raw_parts. If
    // CoreMedia ever over-reports a buffer size, that slice would read out of bounds.
    // Clamp each reported mDataByteSize to the block buffer's actual backing length
    // so the Rust consumer can trust the value it receives. Per-buffer offsets into
    // the block buffer aren't tracked here, so we conservatively clamp every buffer
    // to the total block-buffer data length.
    let blockDataLength = UInt32(truncatingIfNeeded: CMBlockBufferGetDataLength(blockBuffer))

    withUnsafePointer(to: &audioBufferListPtr.pointee.mBuffers) { buffersPtr in
        let bufferArray = UnsafeBufferPointer(start: buffersPtr, count: numBuffers)
        for (index, audioBuffer) in bufferArray.enumerated() {
            let clampedSize = min(audioBuffer.mDataByteSize, blockDataLength)
            buffers[index] = AudioBufferBridge(
                number_channels: audioBuffer.mNumberChannels,
                data_bytes_size: clampedSize,
                data_ptr: audioBuffer.mData
            )
        }
    }

    outNumBuffers.pointee = UInt32(numBuffers)
    outBuffersPtr.pointee = UnsafeMutableRawPointer(buffers)
    outBuffersLen.pointee = UInt(numBuffers)
    // Retain the block buffer to keep data alive, caller must release
    outBlockBuffer.pointee = Unmanaged.passRetained(blockBuffer).toOpaque()
}









// MARK: - CMFormatDescription APIs


@_cdecl("cm_sample_buffer_get_sample_timing_info")
public func cm_sample_buffer_get_sample_timing_info(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ sampleIndex: Int,
    _ outDurationValue: UnsafeMutablePointer<Int64>,
    _ outDurationTimescale: UnsafeMutablePointer<Int32>,
    _ outDurationFlags: UnsafeMutablePointer<UInt32>,
    _ outDurationEpoch: UnsafeMutablePointer<Int64>,
    _ outPtsValue: UnsafeMutablePointer<Int64>,
    _ outPtsTimescale: UnsafeMutablePointer<Int32>,
    _ outPtsFlags: UnsafeMutablePointer<UInt32>,
    _ outPtsEpoch: UnsafeMutablePointer<Int64>,
    _ outDtsValue: UnsafeMutablePointer<Int64>,
    _ outDtsTimescale: UnsafeMutablePointer<Int32>,
    _ outDtsFlags: UnsafeMutablePointer<UInt32>,
    _ outDtsEpoch: UnsafeMutablePointer<Int64>
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    var timingInfo = CMSampleTimingInfo()
    let status = CMSampleBufferGetSampleTimingInfo(buffer, at: sampleIndex, timingInfoOut: &timingInfo)

    if status == noErr {
        outDurationValue.pointee = timingInfo.duration.value
        outDurationTimescale.pointee = timingInfo.duration.timescale
        outDurationFlags.pointee = timingInfo.duration.flags.rawValue
        outDurationEpoch.pointee = timingInfo.duration.epoch

        outPtsValue.pointee = timingInfo.presentationTimeStamp.value
        outPtsTimescale.pointee = timingInfo.presentationTimeStamp.timescale
        outPtsFlags.pointee = timingInfo.presentationTimeStamp.flags.rawValue
        outPtsEpoch.pointee = timingInfo.presentationTimeStamp.epoch

        outDtsValue.pointee = timingInfo.decodeTimeStamp.value
        outDtsTimescale.pointee = timingInfo.decodeTimeStamp.timescale
        outDtsFlags.pointee = timingInfo.decodeTimeStamp.flags.rawValue
        outDtsEpoch.pointee = timingInfo.decodeTimeStamp.epoch
    }

    return status
}

@_cdecl("cm_sample_buffer_invalidate")
public func cm_sample_buffer_invalidate(_ sampleBuffer: UnsafeMutableRawPointer) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    CMSampleBufferInvalidate(buffer)
    return 0
}

@_cdecl("cm_sample_buffer_create_copy_with_new_timing")
public func cm_sample_buffer_create_copy_with_new_timing(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ numTimingInfos: Int,
    _ timingInfoArray: UnsafeRawPointer,
    _ sampleBufferOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let timingInfos = timingInfoArray.bindMemory(to: CMSampleTimingInfo.self, capacity: numTimingInfos)
    let timingArray = Array(UnsafeBufferPointer(start: timingInfos, count: numTimingInfos))

    var newBuffer: CMSampleBuffer?
    let status = CMSampleBufferCreateCopyWithNewTiming(
        allocator: kCFAllocatorDefault,
        sampleBuffer: buffer,
        sampleTimingEntryCount: numTimingInfos,
        sampleTimingArray: timingArray,
        sampleBufferOut: &newBuffer
    )

    if status == noErr, let newBuf = newBuffer {
        sampleBufferOut.pointee = Unmanaged.passRetained(newBuf).toOpaque()
    } else {
        sampleBufferOut.pointee = nil
    }

    return status
}

@_cdecl("cm_sample_buffer_copy_pcm_data_into_audio_buffer_list")
public func cm_sample_buffer_copy_pcm_data_into_audio_buffer_list(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ frameOffset: Int32,
    _ numFrames: Int32,
    _ bufferList: UnsafeMutableRawPointer
) -> Int32 {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let audioBufferList = bufferList.bindMemory(to: AudioBufferList.self, capacity: 1)

    let status = CMSampleBufferCopyPCMDataIntoAudioBufferList(
        buffer,
        at: frameOffset,
        frameCount: numFrames,
        into: audioBufferList
    )

    return status
}











// MARK: - CMSampleBuffer Creation

@_cdecl("cm_sample_buffer_create_for_image_buffer")
public func cm_sample_buffer_create_for_image_buffer(
    _ imageBuffer: UnsafeMutableRawPointer,
    _ presentationTimeValue: Int64,
    _ presentationTimeScale: Int32,
    _ durationValue: Int64,
    _ durationScale: Int32,
    _ sampleBufferOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let pixelBuffer = Unmanaged<CVPixelBuffer>.fromOpaque(imageBuffer).takeUnretainedValue()

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
        sampleBufferOut.pointee = Unmanaged.passRetained(buffer).toOpaque()
    } else {
        sampleBufferOut.pointee = nil
    }

    return status
}

// MARK: - Hash Functions




// MARK: - CMBlockBuffer Creation (for testing)

/// Create a CMBlockBuffer with the given data for testing purposes

/// Create an empty CMBlockBuffer for testing

// MARK: - CMSampleBuffer → CGImage

/// Build a CGImage from the CVImageBuffer (typically CVPixelBuffer) attached
/// to a CMSampleBuffer.
///
/// Backed by `VTCreateCGImageFromCVPixelBuffer`, which handles every pixel
/// format ScreenCaptureKit can deliver (BGRA, 420v YCbCr 8-bit bi-planar
/// video range, l10r 10-bit ARGB, etc.) and uses Apple's hardware-accelerated
/// path when one exists for the input. The returned CGImage is IOSurface-
/// backed where the source was, so downstream consumers (ImageIO encoders,
/// Metal sampling) can avoid host-side pixel copies entirely.
///
/// Returns a retained CGImage pointer on success (caller must release via
/// `cgimage_release`), or NULL on failure with the OSStatus copied into
/// `outStatus`. `outStatus = noErr` on success.
@_cdecl("cm_sample_buffer_create_cg_image")
public func cm_sample_buffer_create_cg_image(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ outStatus: UnsafeMutablePointer<Int32>
) -> OpaquePointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    guard let imageBuffer = CMSampleBufferGetImageBuffer(buffer) else {
        outStatus.pointee = -12731 // kCMSampleBufferError_NoSampleBufferContent
        return nil
    }

    var cgImage: CGImage?
    let status = VTCreateCGImageFromCVPixelBuffer(imageBuffer, options: nil, imageOut: &cgImage)
    outStatus.pointee = status
    guard status == noErr, let image = cgImage else {
        return nil
    }
    return OpaquePointer(Unmanaged.passRetained(image).toOpaque())
}
