#[derive(Debug)]
#[repr(i32)]
pub enum SCFrameStatus {
    // A status that indicates the system successfully generated a new frame.
    Complete,
    // A status that indicates the system didn’t generate a new frame because the display didn’t change.
    Idle,
    // A status that indicates the system didn’t generate a new frame because the display is blank.
    Blank,
    // A status that indicates the system didn’t generate a new frame because you suspended updates.
    Suspended,
    // A status that indicates the frame is the first one sent after the stream starts.
    Started,
    // A status that indicates the frame is in a stopped state.
    Stopped,
}

mod internal {

    #![allow(non_snake_case)]
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};

    use std::{ffi::c_void, mem};

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType,
        error::CFError,
        impl_TCFType,
        number::{CFNumber, CFNumberRef},
        string::CFString,
    };

    use core_media_rs::cm_sample_buffer::CMSampleBufferAttachment;

    use crate::utils::{error::create_cf_error, objc::MessageForTFType};

    use super::SCFrameStatus;

    #[repr(C)]
    pub struct __SCStreamFrameInfoRef(c_void);
    extern "C" {
        pub fn SCStreamFrameInfoGetTypeID() -> CFTypeID;
    }

    pub type SCStreamFrameInfoRef = *mut __SCStreamFrameInfoRef;

    declare_TCFType! {SCStreamFrameInfo, SCStreamFrameInfoRef}
    impl_TCFType!(
        SCStreamFrameInfo,
        SCStreamFrameInfoRef,
        SCStreamFrameInfoGetTypeID
    );

    unsafe impl CMSampleBufferAttachment for SCStreamFrameInfo {}

    impl SCStreamFrameInfo {
        pub fn internal_init() -> Self {
            unsafe {
                let ptr: *mut Object = msg_send![class!(SCStreamFrameInfo), alloc];
                let ptr: SCStreamFrameInfoRef = msg_send![ptr, init];
                Self::wrap_under_create_rule(ptr)
            }
        }
        /// Returns the internal status of this [`SCStreamFrameInfo`].
        ///
        /// # Panics
        ///
        /// Panics if .
        ///
        /// # Errors
        ///
        /// This function will return an error if .
        pub fn internal_status(&self) -> Result<SCFrameStatus, CFError> {
            unsafe {
                let key = CFString::from("StreamUpdateFrameStatus");
                let raw_status: CFNumberRef = msg_send![self.as_sendable(), objectForKey: key];

                if raw_status.is_null() {
                    return Err(create_cf_error("Could not get StreamUpdateFrameStatus, the CMSampleBuffer does not contain any frame data", 0));
                }

                let status = CFNumber::wrap_under_get_rule(raw_status);

                Ok(mem::transmute::<i32, SCFrameStatus>(
                    status.to_i32().unwrap() as i32,
                ))
            }
        }
    }
}
use core_foundation::error::CFError;
pub use internal::SCStreamFrameInfo;


impl SCStreamFrameInfo {
    pub fn new() -> Self {
        Self::internal_init()
    }
    /// Returns the status of this [`SCStreamFrameInfo`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn status(&self) -> Result<SCFrameStatus, CFError> {
        self.internal_status()
    }

}

impl Default for SCStreamFrameInfo {
    fn default() -> Self {
        Self::new()
    }
}
