// Audio device enumeration using AVFoundation

import AVFoundation
import Foundation

/// Represents an audio input device (microphone)
public struct AudioInputDevice {
    public let id: String
    public let name: String
    public let isDefault: Bool
}

/// An immutable list of input devices captured in one pass.
///
/// Enumerating device-by-device against a live `AVCaptureDevice.DiscoverySession`
/// re-runs the query for every accessor, so a microphone plugged in or removed
/// mid-enumeration produces a torn list: a count from one moment, a name from
/// another, and an index that may no longer exist. Freezing one pass into a
/// snapshot makes the whole read consistent.
final class AudioInputDeviceSnapshot {
    let devices: [AudioInputDevice]
    /// Index of the system default input in `devices`, or -1 when there is
    /// none (or it is not part of the discovery session's results).
    let defaultIndex: Int

    init() {
        let discovered = AudioInputDeviceSnapshot.discover()
        let defaultUniqueID = AVCaptureDevice.default(for: .audio)?.uniqueID
        devices = discovered.map {
            AudioInputDevice(
                id: $0.uniqueID,
                name: $0.localizedName,
                isDefault: $0.uniqueID == defaultUniqueID
            )
        }
        defaultIndex = devices.firstIndex { $0.isDefault } ?? -1
    }

    private static func discover() -> [AVCaptureDevice] {
        AVCaptureDevice.DiscoverySession(
            deviceTypes: [.builtInMicrophone, .externalUnknown],
            mediaType: .audio,
            position: .unspecified
        ).devices
    }
}

// MARK: - Snapshot FFI

/// Capture the current input-device list. Never returns nil; release with
/// `sc_audio_input_devices_snapshot_release`.
@_cdecl("sc_audio_input_devices_snapshot_create")
public func createAudioInputDevicesSnapshot() -> OpaquePointer {
    retain(AudioInputDeviceSnapshot())
}

@_cdecl("sc_audio_input_devices_snapshot_release")
public func releaseAudioInputDevicesSnapshot(_ snapshot: OpaquePointer) {
    release(snapshot)
}

@_cdecl("sc_audio_input_devices_snapshot_count")
public func audioInputDevicesSnapshotCount(_ snapshot: OpaquePointer) -> Int {
    let s: AudioInputDeviceSnapshot = unretained(snapshot)
    return s.devices.count
}

/// Index of the default input device, or -1 when there is none.
@_cdecl("sc_audio_input_devices_snapshot_default_index")
public func audioInputDevicesSnapshotDefaultIndex(_ snapshot: OpaquePointer) -> Int {
    let s: AudioInputDeviceSnapshot = unretained(snapshot)
    return s.defaultIndex
}

/// Device unique ID as an owned string (caller frees with `sc_free_string`).
@_cdecl("sc_audio_input_devices_snapshot_id_owned")
public func audioInputDevicesSnapshotIDOwned(
    _ snapshot: OpaquePointer,
    _ index: Int
) -> UnsafeMutablePointer<CChar>? {
    let s: AudioInputDeviceSnapshot = unretained(snapshot)
    guard index >= 0, index < s.devices.count else { return nil }
    return strdup(s.devices[index].id)
}

/// Device display name as an owned string (caller frees with `sc_free_string`).
@_cdecl("sc_audio_input_devices_snapshot_name_owned")
public func audioInputDevicesSnapshotNameOwned(
    _ snapshot: OpaquePointer,
    _ index: Int
) -> UnsafeMutablePointer<CChar>? {
    let s: AudioInputDeviceSnapshot = unretained(snapshot)
    guard index >= 0, index < s.devices.count else { return nil }
    return strdup(s.devices[index].name)
}

@_cdecl("sc_audio_input_devices_snapshot_is_default")
public func audioInputDevicesSnapshotIsDefault(_ snapshot: OpaquePointer, _ index: Int) -> Bool {
    let s: AudioInputDeviceSnapshot = unretained(snapshot)
    guard index >= 0, index < s.devices.count else { return false }
    return s.devices[index].isDefault
}

// MARK: - Legacy per-index FFI
//
// Each of these re-runs the discovery query, so a list assembled from them can
// be torn across a device hot-plug. They remain for ABI compatibility; the
// snapshot API above is what the Rust side uses.

/// Get the count of available audio input devices
@_cdecl("sc_audio_get_input_device_count")
public func getInputDeviceCount() -> Int {
    AudioInputDeviceSnapshot().devices.count
}

/// Get audio input device ID at index into a buffer
@_cdecl("sc_audio_get_input_device_id")
public func getInputDeviceId(index: Int, buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    let devices = AudioInputDeviceSnapshot().devices
    guard index >= 0, index < devices.count else { return false }
    return writeCString(devices[index].id, into: buffer, bufferSize: bufferSize)
}

/// Get audio input device name at index into a buffer
@_cdecl("sc_audio_get_input_device_name")
public func getInputDeviceName(index: Int, buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    let devices = AudioInputDeviceSnapshot().devices
    guard index >= 0, index < devices.count else { return false }
    return writeCString(devices[index].name, into: buffer, bufferSize: bufferSize)
}

/// Check if the device at index is the default audio input device
@_cdecl("sc_audio_is_default_input_device")
public func isDefaultInputDevice(index: Int) -> Bool {
    let devices = AudioInputDeviceSnapshot().devices
    guard index >= 0, index < devices.count else { return false }
    return devices[index].isDefault
}

/// Get the default audio input device ID into a buffer
@_cdecl("sc_audio_get_default_input_device_id")
public func getDefaultInputDeviceId(buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let device = AVCaptureDevice.default(for: .audio) else { return false }
    return writeCString(device.uniqueID, into: buffer, bufferSize: bufferSize)
}

/// Get the default audio input device name into a buffer
@_cdecl("sc_audio_get_default_input_device_name")
public func getDefaultInputDeviceName(buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let device = AVCaptureDevice.default(for: .audio) else { return false }
    return writeCString(device.localizedName, into: buffer, bufferSize: bufferSize)
}
