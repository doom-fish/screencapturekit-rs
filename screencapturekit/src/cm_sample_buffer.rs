use screencapturekit_sys::{
    cm_sample_buffer_ref::CMSampleBufferRef,
    cv_image_buffer_ref::CVImageBufferRef,
    cv_pixel_buffer_ref::CVPixelBufferRef,
    os_types::rc::{Id, ShareId},
    sc_stream_frame_info::SCFrameStatus,
};

#[derive(Debug)]
pub struct CMSampleBuffer {
    pub sys_ref: Id<CMSampleBufferRef>,
    pub image_buf_ref: Id<CVImageBufferRef>,
    pub pixel_buffer_ref: ShareId<CVPixelBufferRef>,
    pub frame_status: SCFrameStatus,
}

impl CMSampleBuffer {
    pub fn new(sys_ref: Id<CMSampleBufferRef>) -> Self {
        let frame_status = sys_ref.get_frame_info().status();
        let image_buf_ref = sys_ref.get_image_buffer();
        let pixel_buffer_ref = image_buf_ref.as_pixel_buffer();
        Self {
            sys_ref,
            image_buf_ref,
            pixel_buffer_ref,
            frame_status,
        }
    }
}
