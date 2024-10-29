use core_foundation::error::CFError;

use super::internal::output_handler::SCStreamOutput;
use super::sc_stream_delegate_trait::SCStreamDelegateTrait;
use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
    sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
};

pub use super::internal::sc_stream::SCStream;
pub use super::internal::sc_stream::SCStreamRef;

impl SCStream {
    pub fn new_with_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: impl SCStreamDelegateTrait,
    ) -> Self {
        Self::internal_init_with_filter_and_delegate(filter, configuration, Some(delegate))
    }

    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        Self::internal_init_with_filter(filter, configuration)
    }

    pub fn add_output_handler(
        &mut self,
        output_trait: impl SCStreamOutputTrait,
        of_type: SCStreamOutputType,
    ) -> Option<SCStreamOutput> {
        self.internal_add_output_handler(output_trait, of_type)
    }

    pub fn remove_output_handler(
        &mut self,
        index: SCStreamOutput,
        of_type: SCStreamOutputType,
    ) -> bool {
        self.internal_remove_output_handler(index, of_type)
    }

    /// Returns the start capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn start_capture(&self) -> Result<(), CFError> {
        self.internal_start_capture()
    }
    /// Returns the stop capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn stop_capture(&self) -> Result<(), CFError> {
        self.internal_stop_capture()
    }
}

#[cfg(test)]
mod stream_test {

    use std::sync::mpsc::{channel, Sender};

    use core_foundation::error::CFError;
    use core_media_rs::cm_sample_buffer::CMSampleBuffer;

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::{SCStream, SCStreamOutputTrait, SCStreamOutputType};

    #[derive(Debug)]
    struct TestStreamOutput {
        sender: Sender<(CMSampleBuffer, SCStreamOutputType)>,
    }

    impl SCStreamOutputTrait for TestStreamOutput {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            self.sender
                .send((sample_buffer, of_type))
                .expect("could not send from output buffer");
        }
    }

    #[test]
    fn test_remove_output_handler() -> Result<(), CFError> {
        let c = channel();
        let output_handler = TestStreamOutput { sender: c.0 };
        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        let display = SCShareableContent::get().unwrap().displays().remove(0);
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        let id = stream.add_output_handler(output_handler, SCStreamOutputType::Screen);
        assert!(id.is_some());
        let removed = stream.remove_output_handler(id.unwrap(), SCStreamOutputType::Screen);
        assert!(removed);
        drop(stream);
        Ok(())
    }
    #[test]
    fn test_sc_stream_audio_list() -> Result<(), CFError> {
        let (tx, rx) = channel();

        let stream = {
            let config = SCStreamConfiguration::new()
                .set_captures_audio(true)?
                .set_width(100)?
                .set_height(100)?;

            let display = SCShareableContent::get().unwrap().displays().remove(0);
            let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
            let mut stream = SCStream::new(&filter, &config);
            stream.add_output_handler(TestStreamOutput { sender: tx }, SCStreamOutputType::Audio);
            stream
        };
        stream.start_capture()?;
        let (buf, _) = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("could not receive from output_buffer");
        let b = buf.get_audio_buffer_list().expect("should work");
        println!("{b:?}");

        stream.stop_capture()?;
        Ok(())
    }
}
