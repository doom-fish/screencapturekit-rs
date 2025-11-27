// Recording Output APIs (macOS 15.0+)
// Stub implementation for macOS < 15.0

import Foundation
import ScreenCaptureKit

// MARK: - Recording Output (macOS 15.0+)

#if compiler(>=6.0)
// Full implementation for Xcode 16+ / Swift 6+ (macOS 15 SDK)

@available(macOS 15.0, *)
private class RecordingDelegate: NSObject, SCRecordingOutputDelegate {
    func recordingOutput(_ recordingOutput: SCRecordingOutput, didFailWithError error: Error) {
    }

    func recordingOutputDidFinishRecording(_ recordingOutput: SCRecordingOutput) {
    }
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
    return retain(output)
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

@_cdecl("sc_recording_output_configuration_set_average_bitrate")
public func setRecordingOutputAverageBitrate(_ config: OpaquePointer?, _ bitrate: Int64) {
}

@_cdecl("sc_recording_output_configuration_retain")
public func retainRecordingOutputConfiguration(_ config: OpaquePointer?) -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_configuration_release")
public func releaseRecordingOutputConfiguration(_ config: OpaquePointer?) {
}

@_cdecl("sc_recording_output_create")
public func createRecordingOutput(_ config: OpaquePointer?) -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_retain")
public func retainRecordingOutput(_ output: OpaquePointer?) -> OpaquePointer? {
    return nil
}

@_cdecl("sc_recording_output_release")
public func releaseRecordingOutput(_ output: OpaquePointer?) {
}

#endif
