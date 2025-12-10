// CoreMedia Bridge - CMSampleBuffer, CMTime, CMFormatDescription, CMBlockBuffer

import CoreMedia
import CoreVideo
import Foundation
import ScreenCaptureKit

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

@_cdecl("cm_sample_buffer_get_image_buffer")
public func cm_sample_buffer_get_image_buffer(_ sampleBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    guard let imageBuffer = CMSampleBufferGetImageBuffer(buffer) else {
        return nil
    }
    return Unmanaged.passRetained(imageBuffer).toOpaque()
}

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

@_cdecl("cm_sample_buffer_get_dirty_rects")
public func cm_sample_buffer_get_dirty_rects(_ sampleBuffer: UnsafeMutableRawPointer, _ outRects: UnsafeMutablePointer<UnsafeMutableRawPointer?>, _ outCount: UnsafeMutablePointer<Int>) -> Bool {
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
    outCount.pointee = rects.count
    return true
}

@_cdecl("cm_sample_buffer_free_dirty_rects")
public func cm_sample_buffer_free_dirty_rects(_ rectsPtr: UnsafeMutableRawPointer) {
    rectsPtr.deallocate()
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

@_cdecl("cm_sample_buffer_get_presentation_timestamp")
public func cm_sample_buffer_get_presentation_timestamp(_ sampleBuffer: UnsafeMutableRawPointer, _ outValue: UnsafeMutablePointer<Int64>, _ outTimescale: UnsafeMutablePointer<Int32>, _ outFlags: UnsafeMutablePointer<UInt32>, _ outEpoch: UnsafeMutablePointer<Int64>) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetPresentationTimeStamp(buffer)
    outValue.pointee = time.value
    outTimescale.pointee = time.timescale
    outFlags.pointee = time.flags.rawValue
    outEpoch.pointee = time.epoch
}

@_cdecl("cm_sample_buffer_get_decode_timestamp")
public func cm_sample_buffer_get_decode_timestamp(
    _ sampleBuffer: UnsafeMutableRawPointer,
    _ value: UnsafeMutablePointer<Int64>,
    _ timescale: UnsafeMutablePointer<Int32>,
    _ flags: UnsafeMutablePointer<UInt32>,
    _ epoch: UnsafeMutablePointer<Int64>
) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let time = CMSampleBufferGetDecodeTimeStamp(buffer)
    value.pointee = time.value
    timescale.pointee = time.timescale
    flags.pointee = time.flags.rawValue
    epoch.pointee = time.epoch
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

@_cdecl("cm_sample_buffer_get_duration")
public func cm_sample_buffer_get_duration(_ sampleBuffer: UnsafeMutableRawPointer, _ outValue: UnsafeMutablePointer<Int64>, _ outTimescale: UnsafeMutablePointer<Int32>, _ outFlags: UnsafeMutablePointer<UInt32>, _ outEpoch: UnsafeMutablePointer<Int64>) {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    let duration = CMSampleBufferGetDuration(buffer)
    outValue.pointee = duration.value
    outTimescale.pointee = duration.timescale
    outFlags.pointee = duration.flags.rawValue
    outEpoch.pointee = duration.epoch
}

@_cdecl("cm_sample_buffer_release")
public func cm_sample_buffer_release(_ sampleBuffer: UnsafeMutableRawPointer) {
    Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).release()
}

@_cdecl("cm_sample_buffer_retain")
public func cm_sample_buffer_retain(_ sampleBuffer: UnsafeMutableRawPointer) {
    _ = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).retain()
}

@_cdecl("cm_sample_buffer_is_valid")
public func cm_sample_buffer_is_valid(_ sampleBuffer: UnsafeMutableRawPointer) -> Bool {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferIsValid(buffer)
}

@_cdecl("cm_sample_buffer_get_num_samples")
public func cm_sample_buffer_get_num_samples(_ sampleBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return CMSampleBufferGetNumSamples(buffer)
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

@_cdecl("cm_sample_buffer_get_audio_buffer_number_channels")
public func cm_sample_buffer_get_audio_buffer_number_channels(_ sampleBuffer: UnsafeMutableRawPointer, _ index: UInt32) -> UInt32 {
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

    guard status == noErr, index < audioBufferList.mNumberBuffers else {
        return 0
    }

    return withUnsafePointer(to: &audioBufferList.mBuffers) { buffersPtr in
        let buffersArray = UnsafeBufferPointer(start: buffersPtr, count: Int(index + 1))
        return buffersArray[Int(index)].mNumberChannels
    }
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

    withUnsafePointer(to: &audioBufferListPtr.pointee.mBuffers) { buffersPtr in
        let bufferArray = UnsafeBufferPointer(start: buffersPtr, count: numBuffers)
        for (index, audioBuffer) in bufferArray.enumerated() {
            buffers[index] = AudioBufferBridge(
                number_channels: audioBuffer.mNumberChannels,
                data_bytes_size: audioBuffer.mDataByteSize,
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

@_cdecl("cm_block_buffer_release")
public func cm_block_buffer_release(_ blockBuffer: UnsafeMutableRawPointer) {
    _ = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeRetainedValue()
}

@_cdecl("cm_block_buffer_retain")
public func cm_block_buffer_retain(_ blockBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return Unmanaged.passRetained(buffer).toOpaque()
}

@_cdecl("cm_block_buffer_get_data_length")
public func cm_block_buffer_get_data_length(_ blockBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return CMBlockBufferGetDataLength(buffer)
}

@_cdecl("cm_block_buffer_is_empty")
public func cm_block_buffer_is_empty(_ blockBuffer: UnsafeMutableRawPointer) -> Bool {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return CMBlockBufferIsEmpty(buffer)
}

@_cdecl("cm_block_buffer_is_range_contiguous")
public func cm_block_buffer_is_range_contiguous(_ blockBuffer: UnsafeMutableRawPointer, _ offset: Int, _ length: Int) -> Bool {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return CMBlockBufferIsRangeContiguous(buffer, atOffset: offset, length: length)
}

@_cdecl("cm_block_buffer_get_data_pointer")
public func cm_block_buffer_get_data_pointer(
    _ blockBuffer: UnsafeMutableRawPointer,
    _ offset: Int,
    _ outLengthAtOffset: UnsafeMutablePointer<Int>,
    _ outTotalLength: UnsafeMutablePointer<Int>,
    _ outDataPointer: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    var lengthAtOffset: Int = 0
    var totalLength: Int = 0
    var dataPointer: UnsafeMutablePointer<CChar>?
    
    let status = CMBlockBufferGetDataPointer(
        buffer,
        atOffset: offset,
        lengthAtOffsetOut: &lengthAtOffset,
        totalLengthOut: &totalLength,
        dataPointerOut: &dataPointer
    )
    
    outLengthAtOffset.pointee = lengthAtOffset
    outTotalLength.pointee = totalLength
    outDataPointer.pointee = dataPointer.map { UnsafeMutableRawPointer($0) }
    
    return status
}

@_cdecl("cm_block_buffer_copy_data_bytes")
public func cm_block_buffer_copy_data_bytes(
    _ blockBuffer: UnsafeMutableRawPointer,
    _ offsetToData: Int,
    _ dataLength: Int,
    _ destination: UnsafeMutableRawPointer
) -> Int32 {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return CMBlockBufferCopyDataBytes(
        buffer,
        atOffset: offsetToData,
        dataLength: dataLength,
        destination: destination
    )
}

@_cdecl("cm_sample_buffer_get_audio_buffer_data_byte_size")
public func cm_sample_buffer_get_audio_buffer_data_byte_size(_ sampleBuffer: UnsafeMutableRawPointer, _ index: UInt32) -> UInt32 {
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

    guard status == noErr, index < audioBufferList.mNumberBuffers else {
        return 0
    }

    return withUnsafePointer(to: &audioBufferList.mBuffers) { buffersPtr in
        let buffersArray = UnsafeBufferPointer(start: buffersPtr, count: Int(index + 1))
        return buffersArray[Int(index)].mDataByteSize
    }
}

@_cdecl("cm_sample_buffer_get_audio_buffer_data")
public func cm_sample_buffer_get_audio_buffer_data(_ sampleBuffer: UnsafeMutableRawPointer, _ index: UInt32) -> UnsafeMutableRawPointer? {
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

    guard status == noErr, index < audioBufferList.mNumberBuffers else {
        return nil
    }

    return withUnsafePointer(to: &audioBufferList.mBuffers) { buffersPtr in
        let buffersArray = UnsafeBufferPointer(start: buffersPtr, count: Int(index + 1))
        return buffersArray[Int(index)].mData
    }
}

@_cdecl("cm_sample_buffer_get_data_buffer")
public func cm_sample_buffer_get_data_buffer(_ sampleBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    guard let dataBuffer = CMSampleBufferGetDataBuffer(buffer) else {
        return nil
    }
    return Unmanaged.passRetained(dataBuffer).toOpaque()
}

// MARK: - CMFormatDescription APIs

@_cdecl("cm_sample_buffer_get_format_description")
public func cm_sample_buffer_get_format_description(_ sampleBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    guard let formatDesc = CMSampleBufferGetFormatDescription(buffer) else {
        return nil
    }
    return Unmanaged.passRetained(formatDesc).toOpaque()
}

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

@_cdecl("cm_format_description_get_media_type")
public func cm_format_description_get_media_type(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    return CMFormatDescriptionGetMediaType(desc)
}

@_cdecl("cm_format_description_get_media_subtype")
public func cm_format_description_get_media_subtype(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    return CMFormatDescriptionGetMediaSubType(desc)
}

@_cdecl("cm_format_description_get_extensions")
public func cm_format_description_get_extensions(_ formatDescription: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let extensions = CMFormatDescriptionGetExtensions(desc) else {
        return nil
    }
    return UnsafeRawPointer(Unmanaged.passUnretained(extensions).toOpaque())
}

@_cdecl("cm_format_description_retain")
public func cm_format_description_retain(_ formatDescription: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    return Unmanaged.passRetained(desc).toOpaque()
}

@_cdecl("cm_format_description_release")
public func cm_format_description_release(_ formatDescription: UnsafeMutableRawPointer) {
    Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).release()
}

@_cdecl("cm_format_description_get_audio_sample_rate")
public func cm_format_description_get_audio_sample_rate(_ formatDescription: UnsafeMutableRawPointer) -> Double {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc) else {
        return 0.0
    }
    return asbd.pointee.mSampleRate
}

@_cdecl("cm_format_description_get_audio_channel_count")
public func cm_format_description_get_audio_channel_count(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc) else {
        return 0
    }
    return asbd.pointee.mChannelsPerFrame
}

@_cdecl("cm_format_description_get_audio_bits_per_channel")
public func cm_format_description_get_audio_bits_per_channel(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc) else {
        return 0
    }
    return asbd.pointee.mBitsPerChannel
}

@_cdecl("cm_format_description_get_audio_bytes_per_frame")
public func cm_format_description_get_audio_bytes_per_frame(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc) else {
        return 0
    }
    return asbd.pointee.mBytesPerFrame
}

@_cdecl("cm_format_description_get_audio_format_flags")
public func cm_format_description_get_audio_format_flags(_ formatDescription: UnsafeMutableRawPointer) -> UInt32 {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    guard let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc) else {
        return 0
    }
    return asbd.pointee.mFormatFlags
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

@_cdecl("cm_sample_buffer_hash")
public func cm_sample_buffer_hash(_ sampleBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return buffer.hashValue
}

@_cdecl("cm_block_buffer_hash")
public func cm_block_buffer_hash(_ blockBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMBlockBuffer>.fromOpaque(blockBuffer).takeUnretainedValue()
    return buffer.hashValue
}

@_cdecl("cm_format_description_hash")
public func cm_format_description_hash(_ formatDescription: UnsafeMutableRawPointer) -> Int {
    let desc = Unmanaged<CMFormatDescription>.fromOpaque(formatDescription).takeUnretainedValue()
    return desc.hashValue
}
