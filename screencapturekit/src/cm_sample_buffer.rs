use screencapturekit_sys::{
    cm_sample_buffer_ref::CMSampleBufferRef,
    cv_image_buffer_ref::CVImageBufferRef,
    os_types::{base::CMTime, rc::{ShareId, Id}},
    sc_stream_frame_info::SCFrameStatus,
};

use crate::cv_pixel_buffer::CVPixelBuffer;

#[derive(Debug)]
pub struct CMSampleBuffer {
    pub sys_ref: ShareId<CMSampleBufferRef>,
    pub frame_status: SCFrameStatus,
    pub presentation_timestamp: CMTime,
}

impl Clone for CMSampleBuffer {
    fn clone(&self) -> Self {
        Self::new(self.sys_ref.clone())
    }
}

impl CMSampleBuffer {
    pub fn new(sys_ref: ShareId<CMSampleBufferRef>) -> Self {
        let frame_status = sys_ref.get_frame_info().status();
        let presentation_timestamp = sys_ref.get_presentation_timestamp();
        Self {
            sys_ref,
            frame_status,
            presentation_timestamp,
        }
    }
    pub fn get_image_buffer(&self) -> Option<Id<CVImageBufferRef>> {
        self.sys_ref.get_image_buffer()
    }
    pub fn get_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        self.sys_ref
            .get_image_buffer()
            .map(|i| CVPixelBuffer::new(i.as_pixel_buffer()))
    }
}
