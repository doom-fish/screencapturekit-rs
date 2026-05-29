//! ABI layout assertions for the `#[repr(C)]` structs shared with the Swift bridge.
//!
//! These structs are passed by value (or via packed buffers) across the
//! Rust <-> Swift `@_cdecl` FFI boundary. If their size or alignment ever drifts
//! from what the Swift side expects, the data marshalling silently corrupts.
//! These tests pin the layout so accidental field reordering / type changes are
//! caught at `cargo test` time rather than as runtime garbage.

use std::mem::{align_of, size_of};

use screencapturekit::ffi::{
    sc_verify_ffi_layout, FFIApplicationData, FFIDisplayData, FFIRect, FFIWindowData,
};

#[test]
fn ffi_rect_layout() {
    // 4 x f64
    assert_eq!(size_of::<FFIRect>(), 32, "FFIRect size drifted");
    assert_eq!(align_of::<FFIRect>(), 8, "FFIRect alignment drifted");
}

#[test]
fn ffi_display_data_layout() {
    // u32 + i32 + i32, then 8-byte-aligned FFIRect (32) => 16 + 32
    assert_eq!(
        size_of::<FFIDisplayData>(),
        48,
        "FFIDisplayData size drifted"
    );
    assert_eq!(
        align_of::<FFIDisplayData>(),
        8,
        "FFIDisplayData alignment drifted"
    );
}

#[test]
fn ffi_window_data_layout() {
    // u32, i32, bool, bool, (pad to 8), FFIRect(32), u32, u32, i32, i32(pad)
    assert_eq!(size_of::<FFIWindowData>(), 64, "FFIWindowData size drifted");
    assert_eq!(
        align_of::<FFIWindowData>(),
        8,
        "FFIWindowData alignment drifted"
    );
}

#[test]
fn ffi_application_data_layout() {
    // i32, i32(pad), u32, u32, u32, u32
    assert_eq!(
        size_of::<FFIApplicationData>(),
        24,
        "FFIApplicationData size drifted"
    );
    assert_eq!(
        align_of::<FFIApplicationData>(),
        4,
        "FFIApplicationData alignment drifted"
    );
}

/// Cross-language ABI check: asks the Swift bridge to verify that *its*
/// `MemoryLayout` (size/stride/alignment) for all four FFI structs matches the
/// values pinned on the Rust side. A `false` return means the Rust and Swift
/// layouts genuinely disagree, which is a real ABI bug.
#[test]
fn ffi_layout_matches_swift() {
    // SAFETY: `sc_verify_ffi_layout` takes no arguments and only reads
    // compile-time `MemoryLayout` constants in the Swift bridge.
    let matches = unsafe { sc_verify_ffi_layout() };
    assert!(
        matches,
        "Swift FFI struct layout disagrees with Rust layout (ABI mismatch)"
    );
}
