// Shared helpers for writing Swift strings into caller-supplied C buffers.

import Foundation

/// Copy `value` into `buffer` as a NUL-terminated C string.
///
/// Returns `false` — leaving an empty string in the buffer when there is room
/// for one — if the value does not fit. Silent truncation (what `strlcpy`
/// does on its own) is worse than failure here: a truncated bundle ID, UTType
/// identifier or file path is indistinguishable from a real one on the Rust
/// side, so callers would act on a corrupted value.
///
/// `bufferSize` is the caller's declared capacity; a non-positive size is
/// rejected without touching the buffer.
func writeCString(_ value: String, into buffer: UnsafeMutablePointer<CChar>?, bufferSize: Int) -> Bool {
    guard let buffer, bufferSize > 0 else { return false }
    return value.withCString { src in
        let length = strlen(src)
        guard length < bufferSize else {
            buffer[0] = 0
            return false
        }
        memcpy(buffer, src, length + 1)
        return true
    }
}
