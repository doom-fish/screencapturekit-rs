use screencapturekit_sys::stream_output_handler::{CMSampleBuffer, UnsafeSCStreamOutput};

pub trait StreamOutput: Sync + Send + 'static {
    fn stream_output(&self, sample: CMSampleBuffer);
}

pub(crate) struct StreamOutputWrapper<T: StreamOutput>(T);

impl<T: StreamOutput> StreamOutputWrapper<T> {
    pub fn new(output: T) -> Self {
        Self(output)
    }
}

impl<TOutput: StreamOutput> UnsafeSCStreamOutput for StreamOutputWrapper<TOutput> {
    fn got_sample(&self, sample: CMSampleBuffer) {
        self.0.stream_output(sample);
    }
}
