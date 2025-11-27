// Dispatch Queue Management

import Foundation
import ScreenCaptureKit

// MARK: - Dispatch Queue Management

@_cdecl("dispatch_queue_create")
public func createDispatchQueue(_ label: UnsafePointer<CChar>, _ qos: Int32) -> OpaquePointer {
    let labelStr = String(cString: label)
    let qosClass: DispatchQoS

    switch qos {
    case 0: qosClass = .background
    case 1: qosClass = .utility
    case 2: qosClass = .default
    case 3: qosClass = .userInitiated
    case 4: qosClass = .userInteractive
    default: qosClass = .default
    }

    let queue = DispatchQueue(label: labelStr, qos: qosClass)
    return retain(queue)
}

@_cdecl("dispatch_queue_release")
public func releaseDispatchQueue(_ queue: OpaquePointer) {
    release(queue)
}

@_cdecl("sc_stream_start_capture")
public func startStreamCapture(
    _ stream: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.startCapture()
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_stop_capture")
public func stopStreamCapture(
    _ stream: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    Task {
        do {
            try await scStream.stopCapture()
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_update_content_filter")
public func updateStreamContentFilter(
    _ stream: OpaquePointer,
    _ filter: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    let scFilter: SCContentFilter = unretained(filter)
    Task {
        do {
            try await scStream.updateContentFilter(scFilter)
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_update_configuration")
public func updateStreamConfiguration(
    _ stream: OpaquePointer,
    _ config: OpaquePointer,
    _ callback: @escaping @convention(c) (Bool, UnsafePointer<CChar>?) -> Void
) {
    let scStream: SCStream = unretained(stream)
    let scConfig: SCStreamConfiguration = unretained(config)
    Task {
        do {
            try await scStream.updateConfiguration(scConfig)
            callback(true, nil)
        } catch {
            let errorMsg = error.localizedDescription
            errorMsg.withCString { callback(false, $0) }
        }
    }
}

@_cdecl("sc_stream_retain")
public func retainStream(_ stream: OpaquePointer) -> OpaquePointer {
    let s: SCStream = unretained(stream)
    return retain(s)
}

@_cdecl("sc_stream_release")
public func releaseStream(_ stream: OpaquePointer) {
    release(stream)
}
