//! Declarative macro for retain/release wrapper boilerplate.
//!
//! Many wrapper types hold a single `*const c_void` pointer to a retained
//! Objective-C / Swift object and hand-roll identical `Clone` (retain) and
//! `Drop` (release) implementations. `sc_retained!` consolidates that
//! boilerplate into a single audited place.
//!
//! The generated impls preserve the exact behavior of the previous
//! hand-written versions:
//! - `Clone` bumps the retain count by calling the supplied `retain` FFI fn.
//! - `Drop` null-checks the pointer before calling the supplied `release`
//!   FFI fn (matching the original `if !ptr.is_null()` guards).
//!
//! Types whose `Clone`/`Drop` carry extra logic beyond retain/release +
//! null-check (e.g. `SCStream`'s teardown ordering or `SCRecordingOutput`'s
//! delegate refcount bookkeeping) are intentionally left hand-written.

/// Generate `Clone` and/or `Drop` impls for a retain/release pointer wrapper.
///
/// Variants:
/// - Tuple newtype, full `Clone` + `Drop`:
///   `sc_retained!(Ty, retain = path::retain, release = path::release);`
/// - Named-field struct (`{ ptr }`), full `Clone` + `Drop`:
///   `sc_retained!(Ty, field = ptr, retain = path::retain, release = path::release);`
/// - Tuple newtype, drop-only:
///   `sc_retained!(Ty, release = path::release);`
/// - Named-field struct, drop-only:
///   `sc_retained!(Ty, field = ptr, release = path::release);`
macro_rules! sc_retained {
    // Named-field struct: Clone + Drop
    ($ty:ty, field = $field:ident, retain = $retain:path, release = $release:path $(,)?) => {
        impl Clone for $ty {
            fn clone(&self) -> Self {
                Self {
                    $field: unsafe { $retain(self.$field) },
                }
            }
        }

        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.$field.is_null() {
                    unsafe {
                        $release(self.$field);
                    }
                }
            }
        }
    };

    // Named-field struct: Drop only
    ($ty:ty, field = $field:ident, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.$field.is_null() {
                    unsafe {
                        $release(self.$field);
                    }
                }
            }
        }
    };

    // Tuple newtype: Clone + Drop
    ($ty:ty, retain = $retain:path, release = $release:path $(,)?) => {
        impl Clone for $ty {
            fn clone(&self) -> Self {
                Self(unsafe { $retain(self.0) })
            }
        }

        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    unsafe {
                        $release(self.0);
                    }
                }
            }
        }
    };

    // Tuple newtype: Drop only
    ($ty:ty, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    unsafe {
                        $release(self.0);
                    }
                }
            }
        }
    };
}

pub(crate) use sc_retained;
