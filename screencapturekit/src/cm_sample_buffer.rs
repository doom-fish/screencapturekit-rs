use screencapturekit_sys::{
    audio_buffer::CopiedAudioBuffer, cm_sample_buffer_ref::CMSampleBufferRef,
    cv_image_buffer_ref::CVImageBufferRef, os_types::rc::Id, sc_stream_frame_info::SCFrameStatus,
};

use crate::cv_pixel_buffer::CVPixelBuffer;

#[derive(Debug)]
pub struct CMSampleBuffer {
    pub sys_ref: Id<CMSampleBufferRef>,
    pub image_buf_ref: Option<Id<CVImageBufferRef>>,
    pub pixel_buffer: Option<CVPixelBuffer>,
    pub audio_buffer_list: Option<Vec<CopiedAudioBuffer>>,
    pub frame_status: SCFrameStatus,
}

impl CMSampleBuffer {
    pub fn new(sys_ref: Id<CMSampleBufferRef>) -> Self {
        let frame_status = sys_ref.get_frame_info().status();
        let image_buf_ref = sys_ref.get_image_buffer();
        let audio_buffer_list = sys_ref.get_av_audio_buffer_list();
        let pixel_buffer = image_buf_ref
            .as_ref()
            .map(|i| CVPixelBuffer::new(i.as_pixel_buffer()));
        Self {
            sys_ref,
            image_buf_ref,
            pixel_buffer,
            audio_buffer_list,
            frame_status,
        }
    }
}
