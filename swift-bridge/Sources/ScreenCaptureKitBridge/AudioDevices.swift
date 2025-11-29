// Audio device enumeration using AVFoundation

import AVFoundation
import Foundation

/// Represents an audio input device (microphone)
public struct AudioInputDevice {
    public let id: String
    public let name: String
    public let isDefault: Bool
}

// MARK: - FFI Functions

/// Get the count of available audio input devices
@_cdecl("sc_audio_get_input_device_count")
public func getInputDeviceCount() -> Int {
    let discoverySession = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.builtInMicrophone, .externalUnknown],
        mediaType: .audio,
        position: .unspecified
    )
    return discoverySession.devices.count
}

/// Get audio input device ID at index into a buffer
@_cdecl("sc_audio_get_input_device_id")
public func getInputDeviceId(index: Int, buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let buffer = buffer, bufferSize > 0 else { return false }
    
    let discoverySession = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.builtInMicrophone, .externalUnknown],
        mediaType: .audio,
        position: .unspecified
    )
    let devices = discoverySession.devices
    guard index >= 0 && index < devices.count else { return false }
    
    let deviceId = devices[index].uniqueID
    return deviceId.withCString { src in
        let length = strlen(src)
        guard length < bufferSize else { return false }
        strcpy(buffer, src)
        return true
    }
}

/// Get audio input device name at index into a buffer
@_cdecl("sc_audio_get_input_device_name")
public func getInputDeviceName(index: Int, buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let buffer = buffer, bufferSize > 0 else { return false }
    
    let discoverySession = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.builtInMicrophone, .externalUnknown],
        mediaType: .audio,
        position: .unspecified
    )
    let devices = discoverySession.devices
    guard index >= 0 && index < devices.count else { return false }
    
    let deviceName = devices[index].localizedName
    return deviceName.withCString { src in
        let length = strlen(src)
        guard length < bufferSize else { return false }
        strcpy(buffer, src)
        return true
    }
}

/// Check if the device at index is the default audio input device
@_cdecl("sc_audio_is_default_input_device")
public func isDefaultInputDevice(index: Int) -> Bool {
    let discoverySession = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.builtInMicrophone, .externalUnknown],
        mediaType: .audio,
        position: .unspecified
    )
    let devices = discoverySession.devices
    guard index >= 0 && index < devices.count else { return false }
    
    // The default device is typically the one returned by default()
    if let defaultDevice = AVCaptureDevice.default(for: .audio) {
        return devices[index].uniqueID == defaultDevice.uniqueID
    }
    return false
}

/// Get the default audio input device ID into a buffer
@_cdecl("sc_audio_get_default_input_device_id")
public func getDefaultInputDeviceId(buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let buffer = buffer, bufferSize > 0 else { return false }
    guard let device = AVCaptureDevice.default(for: .audio) else { return false }
    
    return device.uniqueID.withCString { src in
        let length = strlen(src)
        guard length < bufferSize else { return false }
        strcpy(buffer, src)
        return true
    }
}

/// Get the default audio input device name into a buffer
@_cdecl("sc_audio_get_default_input_device_name")
public func getDefaultInputDeviceName(buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let buffer = buffer, bufferSize > 0 else { return false }
    guard let device = AVCaptureDevice.default(for: .audio) else { return false }
    
    return device.localizedName.withCString { src in
        let length = strlen(src)
        guard length < bufferSize else { return false }
        strcpy(buffer, src)
        return true
    }
}
