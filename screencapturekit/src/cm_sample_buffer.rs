use screencapturekit_sys::{
    cm_sample_buffer_ref::CMSampleBufferRef, cv_image_buffer_ref::CVImageBufferRef,
    os_types::rc::Id, sc_stream_frame_info::SCFrameStatus,
};

use crate::cv_pixel_buffer::CVPixelBuffer;

#[derive(Debug)]
pub struct CMSampleBuffer {
    pub sys_ref: Id<CMSampleBufferRef>,
    pub image_buf_ref: Id<CVImageBufferRef>,
    pub pixel_buffer: CVPixelBuffer,
    pub frame_status: SCFrameStatus,
}

impl CMSampleBuffer {
    pub fn new(sys_ref: Id<CMSampleBufferRef>) -> Self {
        let frame_status = sys_ref.get_frame_info().status();
        let image_buf_ref = sys_ref.get_image_buffer();
        let pixel_buffer = CVPixelBuffer::new(image_buf_ref.as_pixel_buffer());
        Self {
            sys_ref,
            image_buf_ref,
            pixel_buffer,
            frame_status,
        }
    }
}
