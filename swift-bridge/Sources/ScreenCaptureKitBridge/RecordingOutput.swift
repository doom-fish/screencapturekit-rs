// Recording Output APIs (macOS 15.0+)
// Stub implementation for macOS < 15.0

import Foundation
import ScreenCaptureKit

// MARK: - Recording Output (macOS 15.0+)

// Callback type definitions for recording delegate
public typealias RecordingStartedCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias RecordingFailedCallback = @convention(c) (UnsafeMutableRawPointer?, Int32, UnsafePointer<CChar>) -> Void
public typealias RecordingFinishedCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

#if compiler(>=6.0)
// Full implementation for Xcode 16+ / Swift 6+ (macOS 15 SDK)

@available(macOS 15.0, *)
private class RecordingDelegate: NSObject, SCRecordingOutputDelegate {
    var startedCallback: RecordingStartedCallback?
    var failedCallback: RecordingFailedCallback?
    var finishedCallback: RecordingFinishedCallback?
    var context: UnsafeMutableRawPointer?
    weak var outputRef: AnyObject?
    
    func recordingOutputDidStartRecording(_ recordingOutput: SCRecordingOutput) {
        if let cb = startedCallback {
            cb(context)
        }
    }
    
    func recordingOutput(_ recordingOutput: SCRecordingOutput, didFailWithError error: Error) {
        if let cb = failedCallback {
            let errorCode = extractStreamErrorCode(error)
            error.localizedDescription.withCString { cb(context, errorCode, $0) }
        }
    }

    func recordingOutputDidFinishRecording(_ recordingOutput: SCRecordingOutput) {
        if let cb = finishedCallback {
            cb(context)
        }
    }
}

// Storage for delegate to prevent deallocation
@available(macOS 15.0, *)
private var delegateStorage: [ObjectIdentifier: RecordingDelegate] = [:]
@available(macOS 15.0, *)
private let delegateStorageLock = NSLock()

@available(macOS 15.0, *)
private func storeDelegateRef(_ delegate: RecordingDelegate, for output: SCRecordingOutput) {
    delegateStorageLock.lock()
    delegateStorage[ObjectIdentifier(output)] = delegate
    delegateStorageLock.unlock()
}

@available(macOS 15.0, *)
private func removeDelegateRef(for output: SCRecordingOutput) {
    delegateStorageLock.lock()
    delegateStorage.removeValue(forKey: ObjectIdentifier(output))
    delegateStorageLock.unlock()
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_create")
public func createRecordingOutputConfiguration() -> OpaquePointer {
    let config = SCRecordingOutputConfiguration()
    let box = Box(config)
    return retain(box)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_output_url")
public func setRecordingOutputURL(_ config: OpaquePointer, _ path: UnsafePointer<CChar>) {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    let pathString = String(cString: path)
    box.value.outputURL = URL(fileURLWithPath: pathString)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_video_codec")
public func setRecordingOutputVideoCodec(_ config: OpaquePointer, _ codec: Int32) {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    switch codec {
    case 0: box.value.videoCodecType = .h264
    case 1: box.value.videoCodecType = .hevc
    default: break
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_video_codec")
public func getRecordingOutputVideoCodec(_ config: OpaquePointer) -> Int32 {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    switch box.value.videoCodecType {
    case .h264: return 0
    case .hevc: return 1
    default: return 0
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_output_file_type")
public func setRecordingOutputFileType(_ config: OpaquePointer, _ fileType: Int32) {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    switch fileType {
    case 0: box.value.outputFileType = .mp4
    case 1: box.value.outputFileType = .mov
    default: break
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_output_file_type")
public func getRecordingOutputFileType(_ config: OpaquePointer) -> Int32 {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    switch box.value.outputFileType {
    case .mp4: return 0
    case .mov: return 1
    default: return 0
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_available_video_codecs_count")
public func getRecordingOutputAvailableVideoCodecsCount(_ config: OpaquePointer) -> Int {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    return box.value.availableVideoCodecTypes.count
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_available_video_codec_at")
public func getRecordingOutputAvailableVideoCodecAt(_ config: OpaquePointer, _ index: Int) -> Int32 {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    guard index >= 0 && index < box.value.availableVideoCodecTypes.count else { return -1 }
    let codec = box.value.availableVideoCodecTypes[index]
    switch codec {
    case .h264: return 0
    case .hevc: return 1
    default: return -1
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_available_output_file_types_count")
public func getRecordingOutputAvailableFileTypesCount(_ config: OpaquePointer) -> Int {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    return box.value.availableOutputFileTypes.count
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_get_available_output_file_type_at")
public func getRecordingOutputAvailableFileTypeAt(_ config: OpaquePointer, _ index: Int) -> Int32 {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    guard index >= 0 && index < box.value.availableOutputFileTypes.count else { return -1 }
    let fileType = box.value.availableOutputFileTypes[index]
    switch fileType {
    case .mp4: return 0
    case .mov: return 1
    default: return -1
    }
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_set_average_bitrate")
public func setRecordingOutputAverageBitrate(_ config: OpaquePointer, _ bitrate: Int64) {
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_retain")
public func retainRecordingOutputConfiguration(_ config: OpaquePointer) -> OpaquePointer {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    return retain(box)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_configuration_release")
public func releaseRecordingOutputConfiguration(_ config: OpaquePointer) {
    release(config)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_create")
public func createRecordingOutput(_ config: OpaquePointer) -> OpaquePointer? {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    let delegate = RecordingDelegate()
    let output = SCRecordingOutput(configuration: box.value, delegate: delegate)
    
    // Store delegate to prevent deallocation
    storeDelegateRef(delegate, for: output)
    
    delegate.outputRef = output
    return retain(output)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_create_with_delegate")
public func createRecordingOutputWithDelegate(
    _ config: OpaquePointer,
    _ startedCallback: RecordingStartedCallback?,
    _ failedCallback: RecordingFailedCallback?,
    _ finishedCallback: RecordingFinishedCallback?,
    _ context: UnsafeMutableRawPointer?
) -> OpaquePointer? {
    let box: Box<SCRecordingOutputConfiguration> = unretained(config)
    let delegate = RecordingDelegate()
    delegate.startedCallback = startedCallback
    delegate.failedCallback = failedCallback
    delegate.finishedCallback = finishedCallback
    delegate.context = context
    
    let output = SCRecordingOutput(configuration: box.value, delegate: delegate)
    
    // Store delegate to prevent deallocation
    storeDelegateRef(delegate, for: output)
    
    delegate.outputRef = output
    return retain(output)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_get_recorded_duration")
public func getRecordingOutputRecordedDuration(_ output: OpaquePointer, _ value: UnsafeMutablePointer<Int64>, _ timescale: UnsafeMutablePointer<Int32>) {
    let o: SCRecordingOutput = unretained(output)
    let duration = o.recordedDuration
    value.pointee = duration.value
    timescale.pointee = duration.timescale
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_get_recorded_file_size")
public func getRecordingOutputRecordedFileSize(_ output: OpaquePointer) -> Int64 {
    let o: SCRecordingOutput = unretained(output)
    return Int64(o.recordedFileSize)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_retain")
public func retainRecordingOutput(_ output: OpaquePointer) -> OpaquePointer {
    let o: SCRecordingOutput = unretained(output)
    return retain(o)
}

@available(macOS 15.0, *)
@_cdecl("sc_recording_output_release")
public func releaseRecordingOutput(_ output: OpaquePointer) {
    let o: SCRecordingOutput = unretained(output)
    
    // Clean up delegate storage
    removeDelegateRef(for: o)
    
    release(output)
}

#else
// Stub implementation for older compilers (macOS < 15 SDK)

@_cdecl("sc_recording_output_configuration_create")
public func createRecordingOutputConfiguration() -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_configuration_set_output_url")
public func setRecordingOutputURL(_ config: OpaquePointer?, _ path: UnsafePointer<CChar>) {
}

@_cdecl("sc_recording_output_configuration_set_video_codec")
public func setRecordingOutputVideoCodec(_ config: OpaquePointer?, _ codec: Int32) {
}

@_cdecl("sc_recording_output_configuration_get_video_codec")
public func getRecordingOutputVideoCodec(_ config: OpaquePointer?) -> Int32 { 0 }

@_cdecl("sc_recording_output_configuration_set_output_file_type")
public func setRecordingOutputFileType(_ config: OpaquePointer?, _ fileType: Int32) {
}

@_cdecl("sc_recording_output_configuration_get_output_file_type")
public func getRecordingOutputFileType(_ config: OpaquePointer?) -> Int32 { 0 }

@_cdecl("sc_recording_output_configuration_get_available_video_codecs_count")
public func getRecordingOutputAvailableVideoCodecsCount(_ config: OpaquePointer?) -> Int { 0 }

@_cdecl("sc_recording_output_configuration_get_available_video_codec_at")
public func getRecordingOutputAvailableVideoCodecAt(_ config: OpaquePointer?, _ index: Int) -> Int32 { -1 }

@_cdecl("sc_recording_output_configuration_get_available_output_file_types_count")
public func getRecordingOutputAvailableFileTypesCount(_ config: OpaquePointer?) -> Int { 0 }

@_cdecl("sc_recording_output_configuration_get_available_output_file_type_at")
public func getRecordingOutputAvailableFileTypeAt(_ config: OpaquePointer?, _ index: Int) -> Int32 { -1 }

@_cdecl("sc_recording_output_configuration_set_average_bitrate")
public func setRecordingOutputAverageBitrate(_ config: OpaquePointer?, _ bitrate: Int64) {
}

@_cdecl("sc_recording_output_configuration_retain")
public func retainRecordingOutputConfiguration(_ config: OpaquePointer?) -> OpaquePointer? {
    nil
}

@_cdecl("sc_recording_output_configuration_release")
public func releaseRecordingOutputConfiguration(_ config: OpaquePointer?) {
}

@_cdecl("sc_recording_output_create")
public func createRecordingOutput(_ config: OpaquePointer?) -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_create_with_delegate")
public func createRecordingOutputWithDelegate(
    _ config: OpaquePointer?,
    _ startedCallback: RecordingStartedCallback?,
    _ failedCallback: RecordingFailedCallback?,
    _ finishedCallback: RecordingFinishedCallback?,
    _ context: UnsafeMutableRawPointer?
) -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_get_recorded_duration")
public func getRecordingOutputRecordedDuration(_ output: OpaquePointer?, _ value: UnsafeMutablePointer<Int64>, _ timescale: UnsafeMutablePointer<Int32>) {
    value.pointee = 0
    timescale.pointee = 0
}

@_cdecl("sc_recording_output_get_recorded_file_size")
public func getRecordingOutputRecordedFileSize(_ output: OpaquePointer?) -> Int64 { 0 }

@_cdecl("sc_recording_output_retain")
public func retainRecordingOutput(_ output: OpaquePointer?) -> OpaquePointer? {
    nil
}

@_cdecl("sc_recording_output_release")
public func releaseRecordingOutput(_ output: OpaquePointer?) {
}

#endif
