use std::ptr;

use objc::{
    runtime::{Class, Object},
    Message, *,
};
use objc_foundation::{
    INSDictionary, INSObject, INSString, NSData, NSDictionary, NSObject, NSString,
};
use objc_id::{Id, ShareId};

use crate::{
    as_ptr::{AsMutPtr, AsPtr},
    cv_image_buffer::CVImageBufferRef, os_types::geometry::CGSize,
};

pub(crate) struct CIContext;

pub(crate) struct CIImage;

unsafe impl Message for CIContext {}

impl INSObject for CIContext {
    fn class() -> &'static Class {
        Class::get("CIContext").expect("Should exist")
    }
}

unsafe impl Message for CIImage {}

#[repr(C)]
pub(crate) enum CIFormat {
    ARGB8,
    BGRA8,
    RGBA8,
    ABGR8,
    RGBAh,
    RGBA16,
    RGBAf,
    RGBX16,
    RGBXh,
    RGBXf,
    RGB10,
    A8,
    A16,
    Af,
    Ah,
    R8,
    R16,
    Rh,
    Rf,
    RG8,
    RG16,
    RGh,
    RGf,
    L8,
    L16,
    Lh,
    Lf,
    LA8,
    LA16,
    LAh,
    LAf,
}

impl CIImage {
    pub fn init(image_buffer: *mut CVImageBufferRef) -> Id<CIImage> {
        let cls = class!(CIImage);
        unsafe {
            let obj: *mut Self = msg_send![cls, alloc];
            let obj: *mut Self = msg_send![obj, initWithCVImageBuffer: image_buffer];
            println!(":dsize : {:?}", CVImageBufferGetDisplaySize(image_buffer));
            println!(":esize : {:?}", CVImageBufferGetEncodedSize(image_buffer));
            Id::from_retained_ptr(obj)
        }
    }
    pub fn png_representation_of_image(&self, format: CIFormat) -> ShareId<NSData> {
        unsafe {
            let context = CIContext::new();
            let pb: *mut Object = msg_send![self.as_ptr(), pixelBuffer];
            let color_space = CVImageBufferGetColorSpace(pb);
            println!("C: {:p}", pb);
            let string = NSString::from_str("noop");
            let obj = NSObject::new();
            let d = NSDictionary::from_keys_and_objects(&[&*string], vec![obj]);
            // let a: NSDictionary<NSString, *mut Object> = msg_send![self.as_ptr(), ]
            let res: *mut Object = msg_send![context, JPEGRepresentationOfImage: self.as_ptr() colorSpace: color_space options: d];

            println!("AWE: {:p}", res);
            ShareId::from_ptr(res as *mut _)
        }
    }
}

extern "C" {
    fn CVImageBufferGetColorSpace(image: *mut Object) -> *mut Object;
    fn CVImageBufferGetDisplaySize(image:*mut CVImageBufferRef) -> CGSize;
    fn CVImageBufferGetEncodedSize(image:*mut CVImageBufferRef) -> CGSize;
}
