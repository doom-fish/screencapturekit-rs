use screencapturekit_sys::{cv_pixel_buffer_ref::CVPixelBufferRef, os_types::rc::ShareId};

struct CVPixelBuffer {
    pub is_planar: bool,
    pub num_planes: usize,
    unsafe_ref: ShareId<CVPixelBufferRef>,
}

impl CVPixelBuffer {
    pub fn new(unsafe_ref: ShareId<CVPixelBufferRef>) -> Self {
        let is_planar = unsafe_ref.is_planar();
        Self {
            unsafe_ref,
            num_planes: 0, //unsafe_ref.num_planes(),
            is_planar,
        }
    }
    pub fn lock() -> bool {
        false
    }
    pub fn unlock() -> bool {
        false
    }
    pub fn get_base_adress() {}
}
