use crate::{
    sc_content_filter::SCContentFilter,
    sc_error_handler::{StreamErrorHandler, StreamErrorHandlerWrapper},
    sc_output_handler::{StreamOutput, StreamOutputWrapper},
    sc_stream_configuration::SCStreamConfiguration,
};
pub use screencapturekit_sys::stream_output_handler::CMSampleBuffer;
use screencapturekit_sys::{os_types::rc::Id, stream::UnsafeSCStream};

pub struct SCStream {
    pub(crate) _unsafe_ref: Id<UnsafeSCStream>,
}

impl SCStream {
    pub fn new(
        filter: SCContentFilter,
        config: SCStreamConfiguration,
        handler: impl StreamErrorHandler,
    ) -> Self {
        Self {
            _unsafe_ref: UnsafeSCStream::init(
                filter._unsafe_ref,
                config.into(),
                StreamErrorHandlerWrapper::new(handler),
            ),
        }
    }
    pub fn add_output(&mut self, output: impl StreamOutput) {
        self._unsafe_ref
            .add_stream_output(StreamOutputWrapper::new(output));
    }
    pub fn start_capture(&self) {
        self._unsafe_ref.start_capture();
    }
    pub fn stop_capture(&self) {
        self._unsafe_ref.stop_capture();
    }
}

#[cfg(test)]
mod tests {

    use std::sync::mpsc::{sync_channel, SyncSender};

    use screencapturekit_sys::stream_output_handler::CMSampleBuffer;

    use crate::{
        sc_content_filter::InitParams::Display, sc_content_filter::SCContentFilter,
        sc_error_handler::StreamErrorHandler, sc_output_handler::StreamOutput,
        sc_shareable_content::SCShareableContent, sc_stream_configuration::SCStreamConfiguration,
    };

    use super::SCStream;
    struct SomeErrorHandler {}
    struct SomeOutputWrapper {
        pub tx: SyncSender<CMSampleBuffer>,
    }
    impl StreamOutput for SomeOutputWrapper {
        fn stream_output(&self, sample: CMSampleBuffer) {
            self.tx.send(sample).unwrap();
        }
    }
    impl StreamErrorHandler for SomeErrorHandler {
        fn on_error(&self) {}
    }
    #[ignore]
    #[test]
    fn test_output_wrapper() {
        let mut content = SCShareableContent::current();
        let display = content.displays.pop().unwrap();
        let filter = SCContentFilter::new(Display(display));
        let config = SCStreamConfiguration::from_size(100, 100, false);
        let (tx, rx) = sync_channel(1);
        let mut stream = SCStream::new(filter, config, SomeErrorHandler {});
        let w = SomeOutputWrapper { tx };
        stream.add_output(w);
        stream.start_capture();
        println!("{:?}", rx.recv().unwrap());
    }
}
