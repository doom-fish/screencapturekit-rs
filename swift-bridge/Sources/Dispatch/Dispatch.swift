// Dispatch Bridge - DispatchQueue

import Foundation

// MARK: - Dispatch Queue Management

@_cdecl("dispatch_queue_create")
public func createDispatchQueue(_ label: UnsafePointer<CChar>, _ qos: Int32) -> UnsafeMutableRawPointer {
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
    return Unmanaged.passRetained(queue).toOpaque()
}

@_cdecl("dispatch_queue_release")
public func releaseDispatchQueue(_ queue: UnsafeMutableRawPointer) {
    Unmanaged<DispatchQueue>.fromOpaque(queue).release()
}

@_cdecl("dispatch_queue_retain")
public func retainDispatchQueue(_ queue: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let q = Unmanaged<DispatchQueue>.fromOpaque(queue).takeUnretainedValue()
    return Unmanaged.passRetained(q).toOpaque()
}
