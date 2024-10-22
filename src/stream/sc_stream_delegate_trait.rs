use core_foundation::error::CFError;

pub use self::internal::SCStreamDelegate;

use super::internal::sc_stream::SCStream;

mod internal {

    pub struct SCStreamDelegate;
}
pub trait SCStreamDelegateTrait: Send {
    fn output_video_effect_did_start_for_stream(&self, _stream: SCStream) {}
    fn output_video_effect_did_stop_for_stream(&self, _stream: SCStream) {}
    fn did_stop_with_error(&self, _stream: SCStream, _error: CFError) {}
}
