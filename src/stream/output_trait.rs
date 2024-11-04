use core_media_rs::cm_sample_buffer::CMSampleBuffer;

use super::output_type::SCStreamOutputType;

#[allow(clippy::module_name_repetitions)]
pub trait SCStreamOutputTrait: Send {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}
