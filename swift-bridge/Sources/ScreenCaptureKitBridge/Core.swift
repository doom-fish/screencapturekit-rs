// Core memory management utilities for the Swift bridge

import Foundation

/// Helper class to box value types for retain/release
class Box<T> {
    var value: T
    init(_ value: T) {
        self.value = value
    }
}

/// Retains and returns an opaque pointer to a Swift object
func retain<T: AnyObject>(_ obj: T) -> OpaquePointer {
    OpaquePointer(Unmanaged.passRetained(obj).toOpaque())
}

/// Gets an unretained reference to a Swift object from an opaque pointer
func unretained<T: AnyObject>(_ ptr: OpaquePointer) -> T {
    Unmanaged<T>.fromOpaque(UnsafeRawPointer(ptr)).takeUnretainedValue()
}

/// Releases a retained Swift object
func release(_ ptr: OpaquePointer) {
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(ptr)).release()
}
