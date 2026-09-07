//! Demonstrates why foreign allocations must not be reconstructed as a Rust
//! `Vec`: `Vec` routes deallocation through the selected global allocator.
//!
//! `AudioBufferList` now returns its descriptor array to a Swift deallocator;
//! these tests retain the allocator regression rationale.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Mutex;

static TRACKED_PTR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
static TRACKED_DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
// Serialize tests that use the tracking state
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Allocator wrapper that detects when a specific tracked pointer is
/// deallocated through the global allocator path.
struct DetectingAllocator;

unsafe impl GlobalAlloc for DetectingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let tracked = TRACKED_PTR.load(Ordering::SeqCst);
        if !tracked.is_null() && ptr == tracked {
            TRACKED_DEALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        // Always forward to system allocator (memory was allocated there)
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: DetectingAllocator = DetectingAllocator;

/// Demonstrates the bug: `Vec::from_raw_parts` routes deallocation through the
/// global allocator. When the global allocator differs from the system allocator
/// (e.g. mimalloc), this causes a crash (`EXC_BAD_ACCESS`).
#[test]
fn vec_from_raw_parts_routes_through_global_allocator() {
    let _guard = TEST_LOCK.lock().unwrap();

    // Allocate with system allocator (simulating Swift's UnsafeMutablePointer.allocate)
    let layout = Layout::array::<u64>(4).unwrap();
    let ptr = unsafe { System.alloc(layout) };
    assert!(!ptr.is_null());

    TRACKED_PTR.store(ptr, Ordering::SeqCst);
    TRACKED_DEALLOC_COUNT.store(0, Ordering::SeqCst);

    // Vec::from_raw_parts → Vec::drop → global allocator dealloc
    // This is the BUGGY path from AudioBufferList::drop. The lints below
    // are exactly the patterns we're testing - silence them deliberately.
    #[allow(
        clippy::cast_ptr_alignment,
        clippy::ptr_as_ptr,
        clippy::same_item_push,
        clippy::same_length_and_capacity,
        clippy::manual_slice_size_calculation
    )]
    unsafe {
        drop(Vec::from_raw_parts(ptr.cast::<u64>(), 4, 4));
    }

    TRACKED_PTR.store(std::ptr::null_mut(), Ordering::SeqCst);

    let count = TRACKED_DEALLOC_COUNT.load(Ordering::SeqCst);
    assert!(
        count > 0,
        "Vec::from_raw_parts SHOULD route through global allocator (count={count}). \
         This proves the bug: with a real custom allocator like mimalloc, this would crash."
    );
}

/// Demonstrates that an explicitly selected allocator bypasses Rust's global
/// allocator. The production bridge uses the stronger form of this rule by
/// deallocating in Swift itself.
#[test]
fn system_dealloc_bypasses_global_allocator() {
    let _guard = TEST_LOCK.lock().unwrap();

    let layout = Layout::array::<u64>(4).unwrap();
    let ptr = unsafe { System.alloc(layout) };
    assert!(!ptr.is_null());

    TRACKED_PTR.store(ptr, Ordering::SeqCst);
    TRACKED_DEALLOC_COUNT.store(0, Ordering::SeqCst);

    // System.dealloc → system free() directly, bypassing global allocator
    // The production path calls the Swift deallocator instead.
    unsafe {
        System.dealloc(ptr, layout);
    }

    TRACKED_PTR.store(std::ptr::null_mut(), Ordering::SeqCst);

    let count = TRACKED_DEALLOC_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        count, 0,
        "System.dealloc should NOT route through global allocator (count={count})"
    );
}
