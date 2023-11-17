use std::ffi::c_void;

use screencapturekit_sys::{cv_pixel_buffer_ref::CVPixelBufferRef, os_types::rc::ShareId};

#[derive(Debug, Clone)]
pub struct CVPixelBuffer {
    pub is_planar: bool,
    pub plane_count: u64,
    pub sys_ref: ShareId<CVPixelBufferRef>,
}

impl CVPixelBuffer {
    pub fn new(sys_ref: ShareId<CVPixelBufferRef>) -> Self {
        let is_planar = sys_ref.is_planar();
        let plane_count = sys_ref.plane_count();
        Self {
            sys_ref,
            plane_count,
            is_planar,
        }
    }
    pub fn lock(&self) -> bool {
        self.sys_ref.lock_base_address(0) == 0
    }
    pub fn unlock(&self) -> bool {
        self.sys_ref.unlock_base_address(0) == 0
    }
    pub fn get_base_adress(&self) -> *mut c_void {
        self.sys_ref.get_base_address()
    }
    pub fn get_base_adress_of_plane(&self, plane_index: u64) -> *mut c_void {
        self.sys_ref.get_base_address_of_plane(plane_index)
    }
}
