use objc::{class, runtime::Object, Message, *};
use objc_foundation::{INSDictionary, INSValue, NSData, NSDictionary, NSString, NSValue};
use objc_id::ShareId;

use crate::as_ptr::AsMutPtr;

#[repr(C)]
#[derive(Debug)]
pub struct CVImageBufferRef {
    _priv: [u8; 0],
}

unsafe impl Message for CVImageBufferRef {}
unsafe impl Send for CVImageBufferRef {}

impl CVImageBufferRef {
    pub fn get_jpeg_data(&self) -> ShareId<NSData> {
        unsafe {
            let ci_image_class = class!(CIImage);
            let ci_context_class = class!(CIContext);
            let ci_context: *mut Self = msg_send![ci_context_class, alloc];
            let ci_context: *mut Self = msg_send![ci_context, init];
            let ci_image: *mut Self = msg_send![ci_image_class, alloc];
            let ci_image: *mut Self = msg_send![ci_image, initWithCVImageBuffer: self.as_mut_ptr()];
            let pixel_buffer: *mut Object = msg_send![ci_image, pixelBuffer];
            let color_space = CVImageBufferGetColorSpace(pixel_buffer);
            let options = NSDictionary::from_keys_and_objects(
                &[&*kCGImageDestinationLossyCompressionQuality],
                vec![NSValue::from_value(1000.0f32)],
            );
            let jpeg_data: *mut NSData = msg_send![ci_context, JPEGRepresentationOfImage: ci_image colorSpace: color_space options: options];
            ShareId::from_ptr(jpeg_data)
        }
    }
}

extern "C" {
    #[allow(improper_ctypes)]
    static kCGImageDestinationLossyCompressionQuality: *const NSString;

    fn CVImageBufferGetColorSpace(image: *mut Object) -> *mut Object;
}
