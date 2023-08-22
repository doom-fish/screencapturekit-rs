use screencapturekit_sys::{
    cm_sample_buffer_ref::{self, CMSampleBufferRef},
    os_types::rc::Id,
    stream_output_handler::UnsafeSCStreamOutput,
};

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

#[repr(u8)]
pub enum SCStreamOutputType {
    Screen,
    Audio,
}
pub trait StreamOutput: Sync + Send + 'static {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}

pub(crate) struct StreamOutputWrapper<T: StreamOutput>(T);

impl<T: StreamOutput> StreamOutputWrapper<T> {
    pub fn new(output: T) -> Self {
        Self(output)
    }
}

impl<TOutput: StreamOutput> UnsafeSCStreamOutput for StreamOutputWrapper<TOutput> {
    fn did_output_sample_buffer(&self, sample_buffer_ref: Id<CMSampleBufferRef>, of_type: u8) {
        self.0.did_output_sample_buffer(
            CMSampleBuffer::new(sample_buffer_ref),
            match of_type {
                0 => SCStreamOutputType::Screen,
                1 => SCStreamOutputType::Audio,
                _ => unreachable!(),
            },
        );
    }
}
