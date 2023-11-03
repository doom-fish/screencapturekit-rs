use objc::{runtime::Object, *};
use objc_id::Id;

use crate::{
    cv_image_buffer_ref::CVImageBufferRef,
    macros::declare_ref_type, os_types::base::CMTime, sc_stream_frame_info::SCStreamFrameInfo,
};

declare_ref_type!(CMSampleBufferRef);

impl CMSampleBufferRef {
    pub fn get_frame_info(&self) -> Id<SCStreamFrameInfo> {
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
        sample: *const CMSampleBufferRef,
        create: u8,
    ) -> *mut Object;
    pub fn CMSampleBufferGetImageBuffer(sample: *const CMSampleBufferRef) -> *mut CVImageBufferRef;
    pub fn CMSampleBufferGetPresentationTimeStamp(sample: *const CMSampleBufferRef) -> CMTime;
}
