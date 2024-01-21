use std::mem;

use objc::*;

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
    use objc::*;

    use std::{ffi::c_void, mem};

    use core_foundation::{
        base::*,
        declare_TCFType,
        error::CFError,
        impl_TCFType,
        number::{CFNumber, CFNumberRef},
        string::CFString,
    };

    use crate::utils::{error::internal::create_cf_error, objc::impl_deref};

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
    impl_deref!(SCStreamFrameInfo);
    pub(crate) fn init() -> SCStreamFrameInfo {
        unsafe {
            let ptr: SCStreamFrameInfoRef = msg_send![class!(SCStreamFrameInfo), alloc];
            let ptr = msg_send![ptr, init];
            SCStreamFrameInfo::wrap_under_create_rule(ptr)
        }
    }
    pub fn status(status_info: &SCStreamFrameInfo) -> Result<SCFrameStatus, CFError> {
        unsafe {
            let key = CFString::from("StreamUpdateFrameStatus");
            let raw_status: CFNumberRef = msg_send!(*status_info, objectForKey: key);
            if (raw_status.is_null()) {
                return Err(create_cf_error(*"Could not get StreamUpdateFrameStatus, the CMSampleBuffer does not contain any frame data", 0));
            }

            let status = CFNumber::wrap_under_get_rule(raw_status);

            Ok(mem::transmute(*status.to_i32()))
        }
    }
}
pub use internal::SCStreamFrameInfo;

impl SCStreamFrameInfo {
    fn new() -> Self {
        internal::init()
    }
}
