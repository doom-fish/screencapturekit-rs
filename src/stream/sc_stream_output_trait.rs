use core_media_rs::cm_sample_buffer::CMSampleBuffer;

use super::sc_stream_output_type::SCStreamOutputType;

pub trait SCStreamOutputTrait: Send {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}
