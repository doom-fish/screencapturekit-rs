//! Batched data snapshot of `SCShareableContent`.
//!
//! Returned by [`SCShareableContent::snapshot`]. Every field on every
//! display / window / running application is fetched in **one** Swift FFI
//! call per category (instead of `1 + N + 6N` for the per-element accessor
//! pattern).
//!
//! [`SCShareableContent::snapshot`]: super::SCShareableContent::snapshot

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use crate::cg::CGRect;
use crate::ffi::{FFIApplicationData, FFIDisplayData, FFIWindowData};
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::MaybeUninit;

// Caps for the bridge's batch FFI scratch buffers. The bridge silently
// truncates above the cap (`count = min(actual, cap)`), so a saturated count
// is surfaced to callers through [`SnapshotTruncation`] rather than swallowed.
const MAX_DISPLAYS: usize = 64;
const MAX_WINDOWS: usize = 4096;
const MAX_APPS: usize = 1024;
const STRING_POOL_BYTES: usize = 256 * 1024;

/// Plain data describing one display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplaySnapshot {
    pub display_id: u32,
    pub width: i32,
    pub height: i32,
    pub frame: CGRect,
}

/// Plain data describing one running application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSnapshot {
    pub process_id: i32,
    pub bundle_identifier: String,
    pub application_name: String,
}

/// Plain data describing one window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub window_id: u32,
    pub window_layer: i32,
    pub is_on_screen: bool,
    pub is_active: bool,
    pub frame: CGRect,
    /// The window title, or `None` when the window has no title **or** its
    /// title did not fit in the batch string pool. See
    /// [`ContentSnapshot::string_pool_used`] for how to detect pool pressure.
    pub title: Option<String>,
    /// Index into [`ContentSnapshot::applications`], or `None` if the
    /// window has no owning application or the owner wasn't returned in
    /// the same snapshot batch. Always in range for `applications`.
    pub owning_app_index: Option<usize>,
}

/// Which categories of a [`ContentSnapshot`] may have been cut short by the
/// batch FFI scratch-buffer caps.
///
/// A flag is set when the bridge returned exactly as many entries as the
/// buffer could hold, which means the real list is *at least* that long and
/// may be longer. It is deliberately conservative: an exactly-full system
/// reports `true` even though nothing was actually dropped.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotTruncation {
    /// The display list saturated the 64-entry cap.
    pub displays: bool,
    /// The window list saturated the 4096-entry cap.
    pub windows: bool,
    /// The application list saturated the 1024-entry cap.
    pub applications: bool,
}

impl SnapshotTruncation {
    /// Whether any category may have been truncated.
    #[must_use]
    pub const fn any(self) -> bool {
        self.displays || self.windows || self.applications
    }
}

/// All shareable content collected in one batched FFI round-trip.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ContentSnapshot {
    pub displays: Vec<DisplaySnapshot>,
    pub applications: Vec<ApplicationSnapshot>,
    pub windows: Vec<WindowSnapshot>,
    /// Which lists may have been cut short by the scratch-buffer caps.
    pub truncation: SnapshotTruncation,
    /// Bytes of the batch string pool consumed by window titles and
    /// application names.
    ///
    /// The bridge appends each string greedily and **skips** any string that
    /// does not fit in the remaining space (surfacing as `title: None` / an
    /// empty name) rather than partially writing it. Compare against
    /// [`string_pool_capacity`](Self::string_pool_capacity) to judge whether
    /// names may have been dropped.
    pub string_pool_used: usize,
}

impl ContentSnapshot {
    /// Total capacity of the batch string pool, in bytes.
    #[must_use]
    pub const fn string_pool_capacity() -> usize {
        STRING_POOL_BYTES
    }

    /// Drive the three `_batch` Swift FFI functions and unpack their packed
    /// `repr(C)` payloads into Rust-side data structures.
    ///
    /// Returns `None` only when `content` is null. Buffer saturation is
    /// reported through [`ContentSnapshot::truncation`], not as `None`.
    pub(crate) fn collect(content: *const c_void) -> Option<Self> {
        if content.is_null() {
            return None;
        }

        // One scratch pool, reused across the two string-bearing batch calls.
        // Each call copies everything it needs into owned `String`s before
        // returning, so the buffer is free to be overwritten afterwards.
        let mut pool = StringPool::new();

        // SAFETY: each batch FFI function writes at most `max_*` packed
        // entries into the supplied buffer and reports how many it wrote.
        // We only ever read back that many.
        let (displays, displays_truncated) = unsafe { collect_displays(content) };
        let (applications, apps_truncated, apps_pool_used) =
            unsafe { collect_applications(content, &mut pool) };
        let (windows, windows_truncated, windows_pool_used) =
            unsafe { collect_windows(content, &applications, &mut pool) };

        Some(Self {
            displays,
            applications,
            windows,
            truncation: SnapshotTruncation {
                displays: displays_truncated,
                windows: windows_truncated,
                applications: apps_truncated,
            },
            string_pool_used: apps_pool_used.max(windows_pool_used),
        })
    }
}

/// Reusable uninitialised scratch buffer for the bridge's string pool.
///
/// Deliberately **not** zero-filled: the bridge writes `[0, used)` and reports
/// `used`, and nothing ever reads past that, so zeroing 256 KiB per snapshot
/// would be pure waste.
struct StringPool {
    buf: Vec<MaybeUninit<u8>>,
}

impl StringPool {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(STRING_POOL_BYTES),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut i8 {
        self.buf.as_mut_ptr().cast::<i8>()
    }

    /// Borrow the prefix the bridge reported as written.
    ///
    /// # Safety
    ///
    /// `used` bytes starting at the buffer base must have been initialised by
    /// the bridge call that produced `used`.
    unsafe fn initialised(&self, used: usize) -> &[u8] {
        let used = used.min(STRING_POOL_BYTES);
        // SAFETY: the caller guarantees `[0, used)` was written by the bridge,
        // and `used` is clamped to the allocation size.
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr().cast::<u8>(), used) }
    }
}

/// Read the `i`th packed entry the bridge wrote into an uninitialised buffer.
///
/// Takes the base pointer rather than a slice reference: the buffer is built
/// with `Vec::with_capacity`, so `len()` is 0. Coercing it to `&[MaybeUninit<T>]`
/// would yield a slice spanning zero bytes, and offsetting that pointer past
/// the first element is out of bounds for its provenance. `Vec::as_ptr`
/// carries the whole allocation.
///
/// # Safety
///
/// `base` must point at the start of a buffer with capacity for at least
/// `i + 1` entries, and the bridge must have initialised entry `i`.
unsafe fn packed_at<T: Copy>(base: *const MaybeUninit<T>, i: usize) -> T {
    // SAFETY: the caller guarantees the bridge initialised entry `i`, and `i`
    // is bounded by the count the bridge reported (capped at capacity).
    unsafe { base.add(i).read().assume_init() }
}

/// Returns `(displays, possibly_truncated)`.
unsafe fn collect_displays(content: *const c_void) -> (Vec<DisplaySnapshot>, bool) {
    unsafe {
        // `MaybeUninit` because the bridge writes exactly `count` fully
        // initialised entries; materialising a `Vec<FFIDisplayData>` over
        // uninitialised memory would be unsound (the element type has
        // validity invariants).
        let mut buffer: Vec<MaybeUninit<FFIDisplayData>> = Vec::with_capacity(MAX_DISPLAYS);
        let written = crate::ffi::sc_shareable_content_get_displays_batch(
            content,
            buffer.as_mut_ptr().cast::<c_void>(),
            MAX_DISPLAYS as isize,
        );
        if written <= 0 {
            return (Vec::new(), false);
        }
        let count = (written as usize).min(MAX_DISPLAYS);

        let displays = (0..count)
            .map(|i| {
                let d = packed_at(buffer.as_ptr(), i);
                DisplaySnapshot {
                    display_id: d.display_id,
                    width: d.width,
                    height: d.height,
                    frame: CGRect::new(d.frame.x, d.frame.y, d.frame.width, d.frame.height),
                }
            })
            .collect();

        (displays, count == MAX_DISPLAYS)
    }
}

/// Returns `(applications, possibly_truncated, string_pool_used)`.
unsafe fn collect_applications(
    content: *const c_void,
    pool: &mut StringPool,
) -> (Vec<ApplicationSnapshot>, bool, usize) {
    unsafe {
        let mut packed: Vec<MaybeUninit<FFIApplicationData>> = Vec::with_capacity(MAX_APPS);
        let mut strings_used: isize = 0;

        let written = crate::ffi::sc_shareable_content_get_applications_batch(
            content,
            packed.as_mut_ptr().cast::<c_void>(),
            MAX_APPS as isize,
            pool.as_mut_ptr(),
            STRING_POOL_BYTES as isize,
            &mut strings_used,
        );
        if written <= 0 {
            return (Vec::new(), false, 0);
        }
        let count = (written as usize).min(MAX_APPS);
        let used = (strings_used.max(0) as usize).min(STRING_POOL_BYTES);
        let bytes = pool.initialised(used);

        let apps = (0..count)
            .map(|i| {
                let app = packed_at(packed.as_ptr(), i);
                ApplicationSnapshot {
                    process_id: app.process_id,
                    bundle_identifier: read_string(
                        bytes,
                        app.bundle_id_offset,
                        app.bundle_id_length,
                    ),
                    application_name: read_string(bytes, app.app_name_offset, app.app_name_length),
                }
            })
            .collect();

        (apps, count == MAX_APPS, used)
    }
}

/// Returns `(windows, possibly_truncated, string_pool_used)`.
unsafe fn collect_windows(
    content: *const c_void,
    applications: &[ApplicationSnapshot],
    pool: &mut StringPool,
) -> (Vec<WindowSnapshot>, bool, usize) {
    unsafe {
        let mut packed: Vec<MaybeUninit<FFIWindowData>> = Vec::with_capacity(MAX_WINDOWS);
        let mut strings_used: isize = 0;

        let written = crate::ffi::sc_shareable_content_get_windows_batch(
            content,
            packed.as_mut_ptr().cast::<c_void>(),
            MAX_WINDOWS as isize,
            pool.as_mut_ptr(),
            STRING_POOL_BYTES as isize,
            &mut strings_used,
        );

        if written <= 0 {
            return (Vec::new(), false, 0);
        }
        let count = (written as usize).min(MAX_WINDOWS);
        let used = (strings_used.max(0) as usize).min(STRING_POOL_BYTES);
        let bytes = pool.initialised(used);
        let app_indices: HashMap<i32, usize> = applications
            .iter()
            .enumerate()
            .map(|(index, app)| (app.process_id, index))
            .collect();

        let windows = (0..count)
            .map(|i| {
                let w = packed_at(packed.as_ptr(), i);
                let title = if w.title_length == 0 {
                    None
                } else {
                    let s = read_string(bytes, w.title_offset, w.title_length);
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                };
                let owning_app_index = app_indices.get(&w.owning_app_process_id).copied();
                WindowSnapshot {
                    window_id: w.window_id,
                    window_layer: w.window_layer,
                    is_on_screen: w.is_on_screen,
                    is_active: w.is_active,
                    frame: CGRect::new(w.frame.x, w.frame.y, w.frame.width, w.frame.height),
                    title,
                    owning_app_index,
                }
            })
            .collect();

        (windows, count == MAX_WINDOWS, used)
    }
}

fn read_string(pool: &[u8], offset: u32, length: u32) -> String {
    read_str(pool, offset, length).map_or_else(String::new, str::to_owned)
}

/// Zero-copy view of a string slice in the shared pool.
///
/// Returns `None` if the `[offset, offset+length)` range is out of bounds or
/// the bytes are not valid UTF-8. The borrow is tied to the pool, so callers
/// only allocate when they need an owned value.
fn read_str(pool: &[u8], offset: u32, length: u32) -> Option<&str> {
    let start = offset as usize;
    let end = start.checked_add(length as usize)?;
    let bytes = pool.get(start..end)?;
    std::str::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::FFIRect;

    /// Regression test for the `Vec::with_capacity` + `buffer[i]` panic:
    /// `with_capacity` leaves `len() == 0`, so indexing panicked for every
    /// entry the bridge wrote. `packed_at` reads through the pointer instead.
    #[test]
    fn packed_at_reads_bridge_written_entries_from_a_len_zero_vec() {
        let mut buffer: Vec<MaybeUninit<FFIDisplayData>> = Vec::with_capacity(MAX_DISPLAYS);
        assert_eq!(buffer.len(), 0, "with_capacity must not set len");

        let written = [
            FFIDisplayData {
                display_id: 7,
                width: 1920,
                height: 1080,
                frame: FFIRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1920.0,
                    height: 1080.0,
                },
            },
            FFIDisplayData {
                display_id: 9,
                width: 800,
                height: 600,
                frame: FFIRect {
                    x: 1.0,
                    y: 2.0,
                    width: 800.0,
                    height: 600.0,
                },
            },
        ];
        unsafe {
            std::ptr::copy_nonoverlapping(
                written.as_ptr(),
                buffer.as_mut_ptr().cast::<FFIDisplayData>(),
                written.len(),
            );
        }

        let read: Vec<u32> = (0..written.len())
            .map(|i| unsafe { packed_at(buffer.as_ptr(), i) }.display_id)
            .collect();
        assert_eq!(read, vec![7, 9]);
    }

    #[test]
    fn string_pool_reads_only_the_written_prefix() {
        let mut pool = StringPool::new();
        let src = b"hello world";
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), pool.as_mut_ptr().cast::<u8>(), src.len());
        }
        let bytes = unsafe { pool.initialised(src.len()) };
        assert_eq!(bytes, src);
        assert_eq!(read_str(bytes, 0, 5), Some("hello"));
        assert_eq!(read_str(bytes, 6, 5), Some("world"));
        // Past the written prefix -> None, never a read of uninitialised memory.
        assert_eq!(read_str(bytes, 6, 99), None);
        assert_eq!(read_str(bytes, u32::MAX, 1), None);
    }

    #[test]
    fn truncation_any_reflects_individual_flags() {
        assert!(!SnapshotTruncation::default().any());
        assert!(SnapshotTruncation {
            windows: true,
            ..Default::default()
        }
        .any());
        assert!(SnapshotTruncation {
            displays: true,
            ..Default::default()
        }
        .any());
        assert!(SnapshotTruncation {
            applications: true,
            ..Default::default()
        }
        .any());
    }

    #[test]
    fn collect_rejects_null_content() {
        assert!(ContentSnapshot::collect(std::ptr::null()).is_none());
    }
}
