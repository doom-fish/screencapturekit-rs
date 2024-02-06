mod internal {

    #![allow(non_snake_case)]

    use std::{ffi::c_void, ptr};

    use core_foundation::{
        base::{kCFAllocatorDefault, Boolean, CFTypeID, OSStatus, TCFType},
        declare_TCFType, impl_TCFType,
        mach_port::CFAllocatorRef,
    };

    #[repr(C)]
    pub struct __CMSampleBufferRef(c_void);
    extern "C" {
        pub fn CMSampleBufferGetTypeID() -> CFTypeID;
        pub fn CMSampleBufferCreate(
            allocator: CFAllocatorRef,
            dataBuffer: *const c_void,
            dataReady: Boolean,
            makeDataReadyCallback: *const c_void,
            makeDataReadyRefcon: *const c_void,
            formatDescription: *const c_void,
            numSamples: u64,
            numSampleTimingEntries: u64,
            sampleTimingArray: *const c_void,
            numSampleSizeEntries: u64,
            sampleSizeArray: usize,
            sampleBufferOut: *const CMSampleBufferRef,
        ) -> OSStatus;

    }
    pub type CMSampleBufferRef = *mut __CMSampleBufferRef;

    declare_TCFType! {CMSampleBuffer, CMSampleBufferRef}
    impl_TCFType!(CMSampleBuffer, CMSampleBufferRef, CMSampleBufferGetTypeID);
    pub fn empty() -> Option<CMSampleBuffer> {
        let sampleBufferOut: CMSampleBufferRef = ptr::null_mut();
        unsafe {
            let result = CMSampleBufferCreate(
                kCFAllocatorDefault,
                ptr::null(),
                1,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                0,
                0,
                ptr::null(),
                0,
                0,
                &sampleBufferOut,
            );
            if result == 0 {
                Some(CMSampleBuffer::wrap_under_create_rule(sampleBufferOut))
            } else {
                None
            }
        }
    }
}

pub use internal::{CMSampleBuffer, CMSampleBufferRef};

impl CMSampleBuffer {
    /// Creates a new [`CMSampleBuffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn new_empty() -> Self {
        internal::empty().expect("could not create empty sample buffer")
    }
}

impl Default for CMSampleBuffer {
    fn default() -> Self {
        Self::new_empty()
    }
}

#[cfg(test)]
mod test_cm_sample_buffer {
    use super::CMSampleBuffer;

    #[test]
    pub fn test_create_empty() {
        CMSampleBuffer::new_empty();
    }
}
