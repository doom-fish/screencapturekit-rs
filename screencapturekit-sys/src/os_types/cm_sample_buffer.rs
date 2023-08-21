use objc::Message;
use objc_foundation::INSObject;

#[repr(C)]
pub(crate) struct CMSampleBuffer {}

unsafe impl Message for CMSampleBuffer {}

impl INSObject for CMSampleBuffer {
    fn class() -> &'static objc::runtime::Class {
        class!("CMSampleBuffer")
    }
}
