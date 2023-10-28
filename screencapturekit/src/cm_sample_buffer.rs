use screencapturekit_sys::{cm_sample_buffer_ref::{CMSampleBufferRef, self}, os_types::rc::Id};


#[derive(Debug)]
pub struct CMSampleBuffer {
    pub ptr: Id<CMSampleBufferRef>,
    pub frame_status: cm_sample_buffer_ref::SCFrameStatus,
}

impl CMSampleBuffer {}

impl CMSampleBuffer {
    pub fn new(unsafe_ref: Id<CMSampleBufferRef>) -> Self {
        let attachments = unsafe_ref.get_attachments();
        Self {
            ptr: unsafe_ref,
            frame_status: attachments.status(),
        }
    }
}

