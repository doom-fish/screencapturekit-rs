use std::{
    collections::HashMap,
    sync::{Once, RwLock},
};

use objc::{
    class,
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
    Message, *,
};
use objc_foundation::INSObject;
use objc_id::Id;
use once_cell::sync::Lazy;

use crate::os_types::{base::CMTime, four_char_code::FourCharCode};

static OUTPUT_HANDLERS: Lazy<RwLock<HashMap<usize, Box<dyn UnsafeSCStreamOutput + Send + Sync>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[repr(C)]
pub(crate) struct UnsafeSCStreamOutputHandler;

pub trait UnsafeSCStreamOutput: Send + Sync + 'static {
    fn got_sample(&self, sample_buffer: CMSampleBuffer);
}
#[repr(C)]
#[derive(Debug)]
pub struct CVPixelBufferRef {
    _priv: [u8; 9],
}
unsafe impl Message for CVPixelBufferRef {}

unsafe impl Send for CVPixelBufferRef {}

#[derive(Debug)]
pub struct CMSampleBuffer {
    pub presentation_timestamp: CMTime,
    pub pixel_buffer: Option<*mut CVPixelBufferRef>,
}
unsafe impl Message for CMSampleBuffer {}
unsafe impl Send for CMSampleBuffer {}

#[derive(Debug)]
pub struct CMFormatDescription {
    pub media_type: FourCharCode,
    pub media_sub_type: FourCharCode,
}

unsafe impl Message for UnsafeSCStreamOutputHandler {}

extern "C" {
    pub fn CFRetain(sample: *mut Object) -> *mut Object;
    pub fn CFAutorelease(sample: *mut Object) -> *mut Object;
    pub fn CMSampleBufferDataIsReady(sample: *mut Object) -> bool;
    pub fn CMSampleBufferGetDuration(sample: *mut Object) -> CMTime;
    pub fn CMSampleBufferGetOutputDuration(sample: *mut Object) -> CMTime;
    pub fn CMSampleBufferGetNumSamples(sample: *mut Object) -> u32;
    pub fn CMSampleBufferGetDataBuffer(sample: *mut Object) -> *mut Object;
    pub fn CMSampleBufferGetImageBuffer(sample: *mut Object) -> *mut CVPixelBufferRef;
    pub fn CMSampleBufferGetFormatDescription(sample: *mut Object) -> *mut Object;
    pub fn CMSampleBufferGetPresentationTimeStamp(sample: *mut Object) -> CMTime;
    pub fn CMFormatDescriptionGetMediaType(fd: *mut Object) -> u32;
    pub fn CMFormatDescriptionGetMediaSubType(fd: *mut Object) -> u32;
}

impl INSObject for UnsafeSCStreamOutputHandler {
    fn class() -> &'static Class {
        static REGISTER_UNSAFE_SC_OUTPUT_HANDLER: Once = Once::new();
        REGISTER_UNSAFE_SC_OUTPUT_HANDLER.call_once(|| {
            let mut decl = ClassDecl::new("SCStreamOutputHandler", class!(NSObject)).unwrap();
            decl.add_ivar::<usize>("_trait");

            extern "C" fn stream_output(
                this: &mut Object,
                _cmd: Sel,
                _stream: *mut Object,
                sample: *mut Object,
                /* TODO: Expose this into resulting sample buffer */
                _of_type: u8,
            ) {
                unsafe {
                    if sample.is_null() {
                        return;
                    }
                    // TODO: handle sample attachments and FrameInfo instead of null check
                    let pixel_buffer = CMSampleBufferGetImageBuffer(sample);
                    if !pixel_buffer.is_null() {
                        let handler_trait_ptr_address = this.get_ivar::<usize>("_trait");
                        let lookup = OUTPUT_HANDLERS.read().unwrap();
                        let output_handler_trait = lookup.get(handler_trait_ptr_address).unwrap();
                        let sb = CMSampleBuffer {
                            presentation_timestamp: CMSampleBufferGetPresentationTimeStamp(sample),
                            pixel_buffer: Some(pixel_buffer),
                        };

                        output_handler_trait.got_sample(sb)
                    }
                };
            }
            unsafe {
                let stream_output_method: extern "C" fn(
                    &mut Object,
                    Sel,
                    *mut Object,
                    *mut Object,
                    u8,
                ) = stream_output;

                decl.add_method(
                    sel!(stream:didOutputSampleBuffer:ofType:),
                    stream_output_method,
                );
            }

            decl.register();
        });
        class!(SCStreamOutputHandler)
    }
}

impl UnsafeSCStreamOutputHandler {
    fn store_output_handler(&mut self, output_handler: impl UnsafeSCStreamOutput) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut Object);
            let hash = self.hash_code();
            OUTPUT_HANDLERS
                .write()
                .unwrap()
                .insert(hash, Box::new(output_handler));
            obj.set_ivar("_trait", hash);
        }
    }
    pub fn init(output_handler: impl UnsafeSCStreamOutput) -> Id<Self> {
        let mut handle = Self::new();
        handle.store_output_handler(output_handler);
        handle
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[repr(C)]
    struct TestHandler {}
    impl UnsafeSCStreamOutput for TestHandler {
        fn got_sample(&self, sample: CMSampleBuffer) {
            println!("GOT SAMPLE! {:?}", sample);
        }
    }

    #[test]
    fn test_sc_stream_output_handler() {
        let handle = TestHandler {};
        let handle = UnsafeSCStreamOutputHandler::init(handle);
        let _: () = unsafe {
            msg_send![handle, stream: ptr::null_mut() as *mut Object didOutputSampleBuffer: ptr::null_mut() as *mut Object ofType: 0]
        };
    }
}
