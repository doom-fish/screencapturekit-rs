// Core memory management utilities for the Swift bridge

import CoreGraphics
import Foundation

// MARK: - CoreGraphics Initialization

/// Force CoreGraphics initialization by calling CGMainDisplayID
/// This prevents CGS_REQUIRE_INIT crashes on headless systems
/// Made public so it can be called from Rust FFI
@_cdecl("sc_initialize_core_graphics")
public func initializeCoreGraphics() {
    _ = CGMainDisplayID()
}

// MARK: - Error Types

/// Strongly typed errors for the ScreenCaptureKit bridge
public enum SCBridgeError: Error, CustomStringConvertible {
    /// Failed to get shareable content
    case contentUnavailable(String)
    /// Stream operation failed
    case streamError(String)
    /// Configuration error
    case configurationError(String)
    /// Screenshot capture failed
    case screenshotError(String)
    /// Recording operation failed
    case recordingError(String)
    /// Content picker error
    case pickerError(String)
    /// Invalid parameter provided
    case invalidParameter(String)
    /// Permission denied
    case permissionDenied
    /// Unknown error
    case unknown(String)
    
    public var description: String {
        switch self {
        case .contentUnavailable(let msg): return "Content unavailable: \(msg)"
        case .streamError(let msg): return "Stream error: \(msg)"
        case .configurationError(let msg): return "Configuration error: \(msg)"
        case .screenshotError(let msg): return "Screenshot error: \(msg)"
        case .recordingError(let msg): return "Recording error: \(msg)"
        case .pickerError(let msg): return "Picker error: \(msg)"
        case .invalidParameter(let msg): return "Invalid parameter: \(msg)"
        case .permissionDenied: return "Permission denied"
        case .unknown(let msg): return "Unknown error: \(msg)"
        }
    }
    
    /// Convert any Error to SCBridgeError
    static func from(_ error: Error) -> SCBridgeError {
        if let bridgeError = error as? SCBridgeError {
            return bridgeError
        }
        return .unknown(error.localizedDescription)
    }
}

/// Helper to convert error to C string for FFI callback
func errorToCString(_ error: Error) -> UnsafeMutablePointer<CChar>? {
    let bridgeError = SCBridgeError.from(error)
    return strdup(bridgeError.description)
}

// MARK: - Memory Management

/// Helper class to box value types for retain/release
class Box<T> {
    var value: T
    init(_ value: T) {
        self.value = value
    }
}

/// Retains and returns an opaque pointer to a Swift object
/// - Parameter obj: The Swift object to retain
/// - Returns: An opaque pointer that can be passed to Rust
func retain<T: AnyObject>(_ obj: T) -> OpaquePointer {
    OpaquePointer(Unmanaged.passRetained(obj).toOpaque())
}

/// Gets an unretained reference to a Swift object from an opaque pointer
/// - Parameter ptr: The opaque pointer from Rust
/// - Returns: The Swift object without changing retain count
func unretained<T: AnyObject>(_ ptr: OpaquePointer) -> T {
    Unmanaged<T>.fromOpaque(UnsafeRawPointer(ptr)).takeUnretainedValue()
}

/// Releases a retained Swift object
/// - Parameter ptr: The opaque pointer to release
func release(_ ptr: OpaquePointer) {
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(ptr)).release()
}
