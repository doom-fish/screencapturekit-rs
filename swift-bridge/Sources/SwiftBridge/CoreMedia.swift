import CoreMedia
import CoreVideo
import Foundation
import IOSurface
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
    
    // Get the SCFrameStatus attachment
    guard let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer, createIfNecessary: false) as? [[CFString: Any]],
          let firstAttachment = attachments.first,
          let status = firstAttachment[SCStreamFrameInfo.status.rawValue as CFString] as? SCFrameStatus else {
        return -1  // No status available
    }
    
    return Int32(status.rawValue)
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
    Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).retain()
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

    guard status == noErr && index < audioBufferList.mNumberBuffers else {
        return 0
    }

    return withUnsafePointer(to: &audioBufferList.mBuffers) { buffersPtr in
        let buffersArray = UnsafeBufferPointer(start: buffersPtr, count: Int(index + 1))
        return buffersArray[Int(index)].mNumberChannels
    }
}

@_cdecl("cm_sample_buffer_get_audio_buffer_list")
public func cm_sample_buffer_get_audio_buffer_list(_ sampleBuffer: UnsafeMutableRawPointer, _ outNumBuffers: UnsafeMutablePointer<UInt32>, _ outBuffersPtr: UnsafeMutablePointer<UnsafeMutableRawPointer?>, _ outBuffersLen: UnsafeMutablePointer<UInt>) {
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

    guard status == noErr else {
        outNumBuffers.pointee = 0
        outBuffersPtr.pointee = nil
        outBuffersLen.pointee = 0
        return
    }

    let numBuffers = Int(audioBufferList.mNumberBuffers)
    let buffers = UnsafeMutablePointer<AudioBufferBridge>.allocate(capacity: numBuffers)

    withUnsafePointer(to: &audioBufferList.mBuffers) { buffersPtr in
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

    guard status == noErr && index < audioBufferList.mNumberBuffers else {
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

    guard status == noErr && index < audioBufferList.mNumberBuffers else {
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

// MARK: - New CMSampleBuffer APIs

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

// MARK: - CMFormatDescription APIs

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


// MARK: - CVPixelBuffer Bridge

@_cdecl("cv_pixel_buffer_get_width")
public func cv_pixel_buffer_get_width(_ pixelBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferGetWidth(buffer)
}

@_cdecl("cv_pixel_buffer_get_height")
public func cv_pixel_buffer_get_height(_ pixelBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferGetHeight(buffer)
}

@_cdecl("cv_pixel_buffer_get_pixel_format_type")
public func cv_pixel_buffer_get_pixel_format_type(_ pixelBuffer: UnsafeMutableRawPointer) -> UInt32 {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferGetPixelFormatType(buffer)
}

@_cdecl("cv_pixel_buffer_get_bytes_per_row")
public func cv_pixel_buffer_get_bytes_per_row(_ pixelBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferGetBytesPerRow(buffer)
}

@_cdecl("cv_pixel_buffer_lock_base_address")
public func cv_pixel_buffer_lock_base_address(_ pixelBuffer: UnsafeMutableRawPointer, flags: UInt32) -> Int32 {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: CVOptionFlags(flags)))
}

@_cdecl("cv_pixel_buffer_unlock_base_address")
public func cv_pixel_buffer_unlock_base_address(_ pixelBuffer: UnsafeMutableRawPointer, flags: UInt32) -> Int32 {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: CVOptionFlags(flags)))
}

@_cdecl("cv_pixel_buffer_get_base_address")
public func cv_pixel_buffer_get_base_address(_ pixelBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return CVPixelBufferGetBaseAddress(buffer)
}

@_cdecl("cv_pixel_buffer_get_io_surface")
public func cv_pixel_buffer_get_io_surface(_ pixelBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    guard let ioSurface = CVPixelBufferGetIOSurface(buffer) else {
        return nil
    }
    return Unmanaged.passRetained(ioSurface.takeUnretainedValue()).toOpaque()
}

@_cdecl("cv_pixel_buffer_release")
public func cv_pixel_buffer_release(_ pixelBuffer: UnsafeMutableRawPointer) {
    Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).release()
}

@_cdecl("cv_pixel_buffer_retain")
public func cv_pixel_buffer_retain(_ pixelBuffer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return Unmanaged.passRetained(buffer).toOpaque()
}

// MARK: - New CVPixelBuffer APIs

@_cdecl("cv_pixel_buffer_create_with_planar_bytes")
public func cv_pixel_buffer_create_with_planar_bytes(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ numPlanes: Int,
    _ planeBaseAddresses: UnsafePointer<UnsafeMutableRawPointer?>,
    _ planeWidths: UnsafePointer<Int>,
    _ planeHeights: UnsafePointer<Int>,
    _ planeBytesPerRow: UnsafePointer<Int>,
    _ pixelBufferOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    var pixelBuffer: CVPixelBuffer?
    
    // Create temporary mutable copies for the API
    var planeBaseAddressesCopy = Array(UnsafeBufferPointer(start: planeBaseAddresses, count: numPlanes))
    var planeWidthsCopy = Array(UnsafeBufferPointer(start: planeWidths, count: numPlanes))
    var planeHeightsCopy = Array(UnsafeBufferPointer(start: planeHeights, count: numPlanes))
    var planeBytesPerRowCopy = Array(UnsafeBufferPointer(start: planeBytesPerRow, count: numPlanes))
    
    let status = CVPixelBufferCreateWithPlanarBytes(
        kCFAllocatorDefault,
        width,
        height,
        OSType(pixelFormatType),
        nil,
        0,
        numPlanes,
        &planeBaseAddressesCopy,
        &planeWidthsCopy,
        &planeHeightsCopy,
        &planeBytesPerRowCopy,
        nil,
        nil,
        nil,
        &pixelBuffer
    )
    
    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = Unmanaged.passRetained(buffer).toOpaque()
    } else {
        pixelBufferOut.pointee = nil
    }
    
    return status
}

@_cdecl("cv_pixel_buffer_create_with_io_surface")
public func cv_pixel_buffer_create_with_io_surface(
    _ ioSurface: UnsafeMutableRawPointer,
    _ pixelBufferOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let surface = Unmanaged<IOSurface>.fromOpaque(ioSurface).takeUnretainedValue()
    var pixelBuffer: Unmanaged<CVPixelBuffer>?
    
    let status = CVPixelBufferCreateWithIOSurface(
        kCFAllocatorDefault,
        surface,
        nil,
        &pixelBuffer
    )
    
    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = buffer.toOpaque()
    } else {
        pixelBufferOut.pointee = nil
    }
    
    return status
}

@_cdecl("cv_pixel_buffer_get_type_id")
public func cv_pixel_buffer_get_type_id() -> Int {
    return Int(CVPixelBufferGetTypeID())
}

// MARK: - CVPixelBufferPool APIs

@_cdecl("cv_pixel_buffer_pool_create")
public func cv_pixel_buffer_pool_create(
    _ width: Int,
    _ height: Int,
    _ pixelFormatType: UInt32,
    _ maxBuffers: Int,
    _ poolOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    var poolAttributes: [String: Any] = [:]
    if maxBuffers > 0 {
        poolAttributes[kCVPixelBufferPoolMinimumBufferCountKey as String] = maxBuffers
    }
    
    let pixelBufferAttributes: [String: Any] = [
        kCVPixelBufferWidthKey as String: width,
        kCVPixelBufferHeightKey as String: height,
        kCVPixelBufferPixelFormatTypeKey as String: pixelFormatType,
        kCVPixelBufferIOSurfacePropertiesKey as String: [:]
    ]
    
    var pool: CVPixelBufferPool?
    let status = CVPixelBufferPoolCreate(
        kCFAllocatorDefault,
        poolAttributes as CFDictionary,
        pixelBufferAttributes as CFDictionary,
        &pool
    )
    
    if status == kCVReturnSuccess, let bufferPool = pool {
        poolOut.pointee = Unmanaged.passRetained(bufferPool).toOpaque()
    } else {
        poolOut.pointee = nil
    }
    
    return status
}

@_cdecl("cv_pixel_buffer_pool_create_pixel_buffer")
public func cv_pixel_buffer_pool_create_pixel_buffer(
    _ pool: UnsafeMutableRawPointer,
    _ pixelBufferOut: UnsafeMutablePointer<UnsafeMutableRawPointer?>
) -> Int32 {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    var pixelBuffer: CVPixelBuffer?
    
    let status = CVPixelBufferPoolCreatePixelBuffer(
        kCFAllocatorDefault,
        bufferPool,
        &pixelBuffer
    )
    
    if status == kCVReturnSuccess, let buffer = pixelBuffer {
        pixelBufferOut.pointee = Unmanaged.passRetained(buffer).toOpaque()
    } else {
        pixelBufferOut.pointee = nil
    }
    
    return status
}

@_cdecl("cv_pixel_buffer_pool_flush")
public func cv_pixel_buffer_pool_flush(_ pool: UnsafeMutableRawPointer) {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    CVPixelBufferPoolFlush(bufferPool, [])
}

@_cdecl("cv_pixel_buffer_pool_get_type_id")
public func cv_pixel_buffer_pool_get_type_id() -> Int {
    return Int(CVPixelBufferPoolGetTypeID())
}

@_cdecl("cv_pixel_buffer_pool_retain")
public func cv_pixel_buffer_pool_retain(_ pool: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    return Unmanaged.passRetained(bufferPool).toOpaque()
}

@_cdecl("cv_pixel_buffer_pool_release")
public func cv_pixel_buffer_pool_release(_ pool: UnsafeMutableRawPointer) {
    Unmanaged<CVPixelBufferPool>.fromOpaque(pool).release()
}

@_cdecl("cv_pixel_buffer_pool_get_attributes")
public func cv_pixel_buffer_pool_get_attributes(_ pool: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    guard let attributes = CVPixelBufferPoolGetAttributes(bufferPool) else {
        return nil
    }
    return UnsafeRawPointer(Unmanaged.passUnretained(attributes).toOpaque())
}

@_cdecl("cv_pixel_buffer_pool_get_pixel_buffer_attributes")
public func cv_pixel_buffer_pool_get_pixel_buffer_attributes(_ pool: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    guard let attributes = CVPixelBufferPoolGetPixelBufferAttributes(bufferPool) else {
        return nil
    }
    return UnsafeRawPointer(Unmanaged.passUnretained(attributes).toOpaque())
}


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

@_cdecl("io_surface_release")
public func io_surface_release(_ surface: UnsafeMutableRawPointer) {
    Unmanaged<IOSurface>.fromOpaque(surface).release()
}

@_cdecl("io_surface_retain")
public func io_surface_retain(_ surface: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return Unmanaged.passRetained(ioSurface).toOpaque()
}

// MARK: - Hash Functions

@_cdecl("cm_sample_buffer_hash")
public func cm_sample_buffer_hash(_ sampleBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CMSampleBuffer>.fromOpaque(sampleBuffer).takeUnretainedValue()
    return buffer.hashValue
}

@_cdecl("cv_pixel_buffer_hash")
public func cv_pixel_buffer_hash(_ pixelBuffer: UnsafeMutableRawPointer) -> Int {
    let buffer = Unmanaged<CVPixelBuffer>.fromOpaque(pixelBuffer).takeUnretainedValue()
    return buffer.hashValue
}

@_cdecl("cv_pixel_buffer_pool_hash")
public func cv_pixel_buffer_pool_hash(_ pool: UnsafeMutableRawPointer) -> Int {
    let bufferPool = Unmanaged<CVPixelBufferPool>.fromOpaque(pool).takeUnretainedValue()
    return bufferPool.hashValue
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

@_cdecl("io_surface_hash")
public func io_surface_hash(_ surface: UnsafeMutableRawPointer) -> Int {
    let ioSurface = Unmanaged<IOSurface>.fromOpaque(surface).takeUnretainedValue()
    return ioSurface.hashValue
}

