use crate::error::SCError;

use crate::cm::CMTime;
use super::internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    pub fn set_queue_depth(self, queue_depth: u32) -> Result<Self, SCError> {
        // FFI expects isize; u32 may wrap on 32-bit platforms (acceptable)
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_queue_depth(self.as_ptr(), queue_depth as isize);
        }
        Ok(self)
    }
    
    pub fn get_queue_depth(&self) -> u32 {
        // FFI returns isize but queue depth is always positive and fits in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_queue_depth(self.as_ptr()) as u32
        }
    }

    pub fn set_minimum_frame_interval(self, cm_time: &CMTime) -> Result<Self, SCError> {
        unsafe {
            crate::ffi::sc_stream_configuration_set_minimum_frame_interval(
                self.as_ptr(),
                cm_time.value,
                cm_time.timescale,
                cm_time.flags,
                cm_time.epoch,
            );
        }
        Ok(self)
    }
    
    pub fn get_minimum_frame_interval(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            
            crate::ffi::sc_stream_configuration_get_minimum_frame_interval(
                self.as_ptr(),
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );
            
            CMTime {
                value,
                timescale,
                flags,
                epoch,
            }
        }
    }

    /// Set the capture resolution for the stream
    ///
    /// Available on macOS 14.0+. Controls the resolution at which content is captured.
    /// 
    /// # Arguments
    /// * `width` - The width in pixels
    /// * `height` - The height in pixels
    pub fn set_capture_resolution(self, width: usize, height: usize) -> Result<Self, SCError> {
        // FFI expects isize for dimensions (Objective-C NSInteger)
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_capture_resolution(
                self.as_ptr(),
                width as isize,
                height as isize,
            );
        }
        Ok(self)
    }

    /// Get the configured capture resolution
    ///
    /// Returns (width, height) tuple
    pub fn get_capture_resolution(&self) -> (usize, usize) {
        // FFI returns isize but dimensions are always positive
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            let mut width: isize = 0;
            let mut height: isize = 0;
            crate::ffi::sc_stream_configuration_get_capture_resolution(
                self.as_ptr(),
                &mut width,
                &mut height,
            );
            (width as usize, height as usize)
        }
    }
}

