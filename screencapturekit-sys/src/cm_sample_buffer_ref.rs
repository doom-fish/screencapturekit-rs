use std::mem;

use objc::{runtime::Object, Encode, Message, *};
use objc_foundation::{INSString, INSValue, NSString, NSValue};
use objc_id::{Id, ShareId};

use crate::{cv_image_buffer::CVImageBufferRef, os_types::base::CMTime};

#[repr(C)]
#[derive(Debug)]
pub struct CMSampleBufferRef {
    _priv: [u8; 0],
}

unsafe impl Message for CMSampleBufferRef {}
unsafe impl Send for CMSampleBufferRef {}

#[derive(Debug)]
#[repr(C)]
pub struct SCStreamFrameInfo {
    _priv: [u8; 0],
}

// TODO: Documnent using comment docs matching apple
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

impl SCStreamFrameInfo {
    pub fn status(&self) -> SCFrameStatus {
        unsafe {
            let key = NSString::from_str("SCStreamUpdateFrameStatus");
            let raw_status: ShareId<NSValue<i32>> = msg_send!(self, objectForKey: key);
            mem::transmute(raw_status.value())
        }
    }
}
unsafe impl Message for SCStreamFrameInfo {}

unsafe impl Encode for &CMSampleBufferRef {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("^v") }
    }
}

impl CMSampleBufferRef {
    pub fn get_attachments(&self) -> Id<SCStreamFrameInfo> {
        unsafe {
            let raw_attachments_array = CMSampleBufferGetSampleAttachmentsArray(self, 0);
            let first = msg_send![raw_attachments_array, firstObject];
            Id::from_ptr(first)
        }
    }
    pub fn get_presentation_timestamp(&self) -> CMTime {
        unsafe { CMSampleBufferGetPresentationTimeStamp(self) }
    }

    pub fn get_image_buffer(&self) -> Id<CVImageBufferRef> {
        unsafe { Id::from_ptr(CMSampleBufferGetImageBuffer(self)) }
    }
}

extern "C" {
    pub fn CMSampleBufferGetSampleAttachmentsArray(
        sample: &CMSampleBufferRef,
        create: u8,
    ) -> *mut Object;
    pub fn CMSampleBufferGetImageBuffer(sample: &CMSampleBufferRef) -> *mut CVImageBufferRef;
    pub fn CMSampleBufferGetPresentationTimeStamp(sample: &CMSampleBufferRef) -> CMTime;
}
