// Core memory management utilities for the Swift bridge

import CoreGraphics
import Foundation

// MARK: - FFI Data Structures

/// Packed CGRect for efficient FFI transfer (32 bytes)
@frozen
public struct FFIRect {
    public var x: Double
    public var y: Double
    public var width: Double
    public var height: Double
    
    public init(_ rect: CGRect) {
        self.x = rect.origin.x
        self.y = rect.origin.y
        self.width = rect.size.width
        self.height = rect.size.height
    }
    
    public static let zero = FFIRect(x: 0, y: 0, width: 0, height: 0)
    
    public init(x: Double, y: Double, width: Double, height: Double) {
        self.x = x
        self.y = y
        self.width = width
        self.height = height
    }
}

/// Packed display data for batch retrieval (48 bytes)
@frozen
public struct FFIDisplayData {
    public var displayId: UInt32
    public var width: Int32
    public var height: Int32
    public var frame: FFIRect
}

/// Packed window data for batch retrieval
@frozen
public struct FFIWindowData {
    public var windowId: UInt32
    public var windowLayer: Int32
    public var isOnScreen: Bool
    public var isActive: Bool
    public var frame: FFIRect
    // Title handled separately via titleOffset/titleLength into string buffer
    public var titleOffset: UInt32
    public var titleLength: UInt32
    // Owning app index (-1 if none)
    public var owningAppIndex: Int32
    public var _padding: Int32
}

/// Packed application data for batch retrieval
@frozen
public struct FFIApplicationData {
    public var processId: Int32
    public var _padding: Int32
    // Bundle ID and app name via offsets into string buffer
    public var bundleIdOffset: UInt32
    public var bundleIdLength: UInt32
    public var appNameOffset: UInt32
    public var appNameLength: UInt32
}

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

/// Extract SCStreamError code from an error, if applicable
/// Returns the raw error code, or 0 if not an SCStreamError
func extractStreamErrorCode(_ error: Error) -> Int32 {
    let nsError = error as NSError
    if nsError.domain == "com.apple.ScreenCaptureKit.SCStreamErrorDomain" {
        return Int32(nsError.code)
    }
    return 0
}

/// Format error with code for FFI transfer
/// Format: "CODE:message" where CODE is the SCStreamError code or 0
func errorWithCodeToCString(_ error: Error) -> UnsafeMutablePointer<CChar>? {
    let code = extractStreamErrorCode(error)
    let message = error.localizedDescription
    let formatted = "\(code):\(message)"
    return strdup(formatted)
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
