use crate::{
    sc_content_filter::SCContentFilter,
    sc_error_handler::{StreamErrorHandler, StreamErrorHandlerWrapper},
    sc_output_handler::{StreamOutput, StreamOutputWrapper},
    sc_stream_configuration::SCStreamConfiguration,
};
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

    use crate::{
        cm_sample_buffer::CMSampleBuffer,
        sc_content_filter::InitParams::Display,
        sc_content_filter::SCContentFilter,
        sc_error_handler::StreamErrorHandler,
        sc_output_handler::{SCStreamOutputType, StreamOutput},
        sc_shareable_content::SCShareableContent,
        sc_stream_configuration::SCStreamConfiguration,
    };

    use super::SCStream;
    struct SomeErrorHandler {}
    struct SomeOutputWrapper {
        pub audio_tx: SyncSender<CMSampleBuffer>,
        pub video_tx: SyncSender<CMSampleBuffer>,
    }
    impl StreamErrorHandler for SomeErrorHandler {
        fn on_error(&self) {}
    }
    impl StreamOutput for SomeOutputWrapper {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
            match of_type {
                SCStreamOutputType::Screen => self.video_tx.send(sample),
                SCStreamOutputType::Audio => self.audio_tx.send(sample),
            }
            .unwrap()
        }
    }
    #[ignore]
    #[test]
    fn test_output_wrapper() {
        let mut content = SCShareableContent::current();
        let display = content.displays.pop().unwrap();
        let width = display.width;
        let height = display.height;
        let filter = SCContentFilter::new(Display(display));
        let config = SCStreamConfiguration {
            width,
            height,
            captures_audio: true,
            ..Default::default()
        };
        let (audio_tx, audio_rx) = sync_channel(1);
        let (video_tx, video_rx) = sync_channel(1);
        let mut stream = SCStream::new(filter, config, SomeErrorHandler {});
        let w = SomeOutputWrapper { video_tx, audio_tx };
        stream.add_output(w);
        stream.start_capture();
        println!("{:?}", audio_rx.recv().unwrap());
    }
}
