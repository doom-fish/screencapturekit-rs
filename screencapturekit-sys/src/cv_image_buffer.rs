use objc::{runtime::Object, Encode, Encoding, Message};
use objc_foundation::{NSData, INSObject};
use objc_id::{ShareId, Id};

use crate::{
    as_ptr::{AsMutPtr, AsPtr},
    ci_image::{CIFormat, CIImage},
};

#[repr(C)]
#[derive(Debug)]
pub struct CVImageBufferRef {
    _priv: [u8; 0],
}

unsafe impl Message for CVImageBufferRef {}
unsafe impl Send for CVImageBufferRef {}



impl CVImageBufferRef {
    pub fn get_data(&self) -> ShareId<NSData> {
        unsafe {
            let ci_image = CIImage::init(self.as_mut_ptr());
             println!("AAA: {:p}", ci_image);
            let a = ci_image.png_representation_of_image(CIFormat::BGRA8);
            a
        }
    }
}

