//! Batched data snapshot of `SCShareableContent`.
//!
//! Returned by [`SCShareableContent::snapshot`]. Every field on every
//! display / window / running application is fetched in **one** Swift FFI
//! call per category (instead of `1 + N + 6N` for the per-element accessor
//! pattern), saving ~70 µs on a system with ~220 windows.
//!
//! [`SCShareableContent::snapshot`]: super::SCShareableContent::snapshot

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use crate::cg::CGRect;
use crate::ffi::{FFIApplicationData, FFIDisplayData, FFIWindowData};
use std::ffi::c_void;
use std::mem::MaybeUninit;

// Static caps for the bridge's batch FFI scratch buffers. These are
// intentionally generous — the bridge silently truncates above the cap, so
// erring high is the safe default. A 256 KiB string pool covers ~256 windows
// each with a 1 KiB title (real-world titles are <128 bytes typically).
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
    pub title: Option<String>,
    /// Index into [`ContentSnapshot::applications`], or `None` if the
    /// window has no owning application or the owner wasn't returned in
    /// the same snapshot batch.
    pub owning_app_index: Option<usize>,
}

/// All shareable content collected in one batched FFI round-trip.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ContentSnapshot {
    pub displays: Vec<DisplaySnapshot>,
    pub applications: Vec<ApplicationSnapshot>,
    pub windows: Vec<WindowSnapshot>,
}

impl ContentSnapshot {
    /// Drive the three `_batch` Swift FFI functions and unpack their packed
    /// `repr(C)` payloads into Rust-side data structures.
    pub(crate) fn collect(content: *const c_void) -> Option<Self> {
        if content.is_null() {
            return None;
        }

        // SAFETY: each batch FFI function writes at most `max_*` packed
        // entries into the supplied buffer. We size the buffers to the caps
        // declared above and trust the bridge to honour them (the bridge
        // takes the cap as a parameter and uses `min(actual, cap)`).
        let displays = unsafe { collect_displays(content) };
        let applications = unsafe { collect_applications(content) };
        let windows = unsafe { collect_windows(content, applications.len()) };

        Some(Self {
            displays,
            applications,
            windows,
        })
    }
}

unsafe fn collect_displays(content: *const c_void) -> Vec<DisplaySnapshot> {
    unsafe {
        // Use `MaybeUninit` for the scratch buffer: the Swift bridge writes
        // exactly `count` fully-initialised entries and we only ever read those.
        // Building a `Vec<FFIDisplayData>` over uninitialised memory would be
        // unsound because the element type has validity invariants.
        let mut buffer: Vec<MaybeUninit<FFIDisplayData>> = Vec::with_capacity(MAX_DISPLAYS);
        let written = crate::ffi::sc_shareable_content_get_displays_batch(
            content,
            buffer.as_mut_ptr().cast::<c_void>(),
            MAX_DISPLAYS as isize,
        );
        if written <= 0 {
            return Vec::new();
        }
        let count = (written as usize).min(MAX_DISPLAYS);

        (0..count)
            .map(|i| {
                let d = buffer[i].assume_init();
                DisplaySnapshot {
                    display_id: d.display_id,
                    width: d.width,
                    height: d.height,
                    frame: CGRect::new(d.frame.x, d.frame.y, d.frame.width, d.frame.height),
                }
            })
            .collect()
    }
}

unsafe fn collect_applications(content: *const c_void) -> Vec<ApplicationSnapshot> {
    unsafe {
        let mut packed: Vec<MaybeUninit<FFIApplicationData>> = Vec::with_capacity(MAX_APPS);
        let mut strings: Vec<i8> = vec![0; STRING_POOL_BYTES];
        let mut strings_used: isize = 0;

        let written = crate::ffi::sc_shareable_content_get_applications_batch(
            content,
            packed.as_mut_ptr().cast::<c_void>(),
            MAX_APPS as isize,
            strings.as_mut_ptr(),
            STRING_POOL_BYTES as isize,
            &mut strings_used,
        );
        if written <= 0 {
            return Vec::new();
        }
        let count = (written as usize).min(MAX_APPS);

        let pool: &[u8] = std::slice::from_raw_parts(
            strings.as_ptr().cast::<u8>(),
            (strings_used as usize).min(STRING_POOL_BYTES),
        );

        (0..count)
            .map(|i| {
                let app = packed[i].assume_init();
                ApplicationSnapshot {
                    process_id: app.process_id,
                    bundle_identifier: read_string(
                        pool,
                        app.bundle_id_offset,
                        app.bundle_id_length,
                    ),
                    application_name: read_string(pool, app.app_name_offset, app.app_name_length),
                }
            })
            .collect()
    }
}

unsafe fn collect_windows(content: *const c_void, app_count_hint: usize) -> Vec<WindowSnapshot> {
    unsafe {
        let mut packed: Vec<MaybeUninit<FFIWindowData>> = Vec::with_capacity(MAX_WINDOWS);
        let mut strings: Vec<i8> = vec![0; STRING_POOL_BYTES];
        let mut strings_used: isize = 0;
        // The Swift bridge also retains app pointers into this buffer for the
        // owning-app index lookup. We don't need to keep the pointers (we already
        // have `applications` from the previous batch call), but we still have to
        // accept and release them so the bridge balances its retain.
        let app_cap = MAX_APPS.max(app_count_hint);
        let mut app_pointers: Vec<*const c_void> = vec![std::ptr::null(); app_cap];
        let mut app_count: isize = 0;

        let written = crate::ffi::sc_shareable_content_get_windows_batch(
            content,
            packed.as_mut_ptr().cast::<c_void>(),
            MAX_WINDOWS as isize,
            strings.as_mut_ptr(),
            STRING_POOL_BYTES as isize,
            &mut strings_used,
            app_pointers.as_mut_ptr(),
            app_cap as isize,
            &mut app_count,
        );

        // Release the SCRunningApplication pointers the bridge retained for us;
        // we don't need them in the snapshot path (the snapshot is plain data).
        let returned_apps = (app_count as usize).min(app_cap);
        for &ptr in &app_pointers[..returned_apps] {
            if !ptr.is_null() {
                crate::ffi::sc_running_application_release(ptr);
            }
        }

        if written <= 0 {
            return Vec::new();
        }
        let count = (written as usize).min(MAX_WINDOWS);

        let pool: &[u8] = std::slice::from_raw_parts(
            strings.as_ptr().cast::<u8>(),
            (strings_used as usize).min(STRING_POOL_BYTES),
        );

        (0..count)
            .map(|i| {
                let w = packed[i].assume_init();
                let title = if w.title_length == 0 {
                    None
                } else {
                    let s = read_string(pool, w.title_offset, w.title_length);
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                };
                WindowSnapshot {
                    window_id: w.window_id,
                    window_layer: w.window_layer,
                    is_on_screen: w.is_on_screen,
                    is_active: w.is_active,
                    frame: CGRect::new(w.frame.x, w.frame.y, w.frame.width, w.frame.height),
                    title,
                    owning_app_index: if w.owning_app_index < 0 {
                        None
                    } else {
                        Some(w.owning_app_index as usize)
                    },
                }
            })
            .collect()
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
    let end = start.saturating_add(length as usize);
    let bytes = pool.get(start..end)?;
    std::str::from_utf8(bytes).ok()
}
