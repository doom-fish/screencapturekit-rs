use std::mem;

use objc::{runtime::Object, Message, *};
use objc_foundation::{INSString, INSValue, NSString, NSValue};
use objc_id::{Id, ShareId};

use crate::os_types::base::CMTime;
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

impl CMSampleBufferRef {
    pub fn get_attachments(&self) -> Id<SCStreamFrameInfo> {
        unsafe {
            let raw_ptr = self as *const Self;
            let raw_attachments_array = CMSampleBufferGetSampleAttachmentsArray(raw_ptr, 0);
            let first = msg_send![raw_attachments_array, firstObject];
            Id::from_ptr(first)
        }
    }
    pub fn get_presentation_timestamp(&self) -> CMTime {
        unsafe {
            let raw_ptr = self as *const Self;
            CMSampleBufferGetPresentationTimeStamp(raw_ptr)
        }
    }
}

extern "C" {
    pub fn CMSampleBufferGetSampleAttachmentsArray(
        sample: *const CMSampleBufferRef,
        create: u8,
    ) -> *mut Object;
    pub fn CFShow(d: *const SCStreamFrameInfo);
    pub fn CMSampleBufferDataIsReady(sample: *mut Object) -> bool;
    pub fn CMSampleBufferGetDuration(sample: *mut Object) -> CMTime;
    pub fn CMSampleBufferGetOutputDuration(sample: *mut Object) -> CMTime;
    pub fn CMSampleBufferGetNumSamples(sample: *mut Object) -> u32;
    pub fn CMSampleBufferGetDataBuffer(sample: *mut Object) -> *mut Object;
    pub fn CMSampleBufferGetImageBuffer(sample: CMSampleBufferRef) -> *mut Object;
    pub fn CMSampleBufferGetFormatDescription(sample: *mut Object) -> *mut Object;
    pub fn CMSampleBufferGetPresentationTimeStamp(sample: *const CMSampleBufferRef) -> CMTime;
    pub fn CMFormatDescriptionGetMediaType(fd: *mut Object) -> u32;
    pub fn CMFormatDescriptionGetMediaSubType(fd: *mut Object) -> u32;
}
