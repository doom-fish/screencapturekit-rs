//! `CMSampleBuffer` - Container for media samples

use super::ffi;
use super::{
    AudioBuffer, AudioBufferList, AudioBufferListRaw, CMBlockBuffer, CMFormatDescription,
    CMSampleTimingInfo, CMTime, SCFrameStatus,
};
use crate::cv::CVPixelBuffer;
use std::fmt;

/// Opaque handle to `CMSampleBuffer`
#[repr(transparent)]
#[derive(Debug)]
pub struct CMSampleBuffer(*mut std::ffi::c_void);

impl PartialEq for CMSampleBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMSampleBuffer {}

impl std::hash::Hash for CMSampleBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_sample_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CMSampleBuffer {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMSampleBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a sample buffer for an image buffer (video frame)
    ///
    /// # Arguments
    ///
    /// * `image_buffer` - The pixel buffer containing the video frame
    /// * `presentation_time` - When the frame should be presented
    /// * `duration` - How long the frame should be displayed
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the sample buffer creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::{CMSampleBuffer, CMTime};
    /// use screencapturekit::cv::CVPixelBuffer;
    ///
    /// // Create a pixel buffer
    /// let pixel_buffer = CVPixelBuffer::create(1920, 1080, 0x42475241)
    ///     .expect("Failed to create pixel buffer");
    ///
    /// // Create timing information (30fps video)
    /// let presentation_time = CMTime::new(0, 30); // Frame 0 at 30 fps
    /// let duration = CMTime::new(1, 30);          // 1/30th of a second
    ///
    /// // Create sample buffer
    /// let sample = CMSampleBuffer::create_for_image_buffer(
    ///     &pixel_buffer,
    ///     presentation_time,
    ///     duration,
    /// ).expect("Failed to create sample buffer");
    ///
    /// assert!(sample.is_valid());
    /// assert_eq!(sample.presentation_timestamp().value, 0);
    /// assert_eq!(sample.presentation_timestamp().timescale, 30);
    /// ```
    pub fn create_for_image_buffer(
        image_buffer: &CVPixelBuffer,
        presentation_time: CMTime,
        duration: CMTime,
    ) -> Result<Self, i32> {
        unsafe {
            let mut sample_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cm_sample_buffer_create_for_image_buffer(
                image_buffer.as_ptr(),
                presentation_time.value,
                presentation_time.timescale,
                duration.value,
                duration.timescale,
                &mut sample_buffer_ptr,
            );

            if status == 0 && !sample_buffer_ptr.is_null() {
                Ok(Self(sample_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Get the image buffer (pixel buffer) from this sample
    pub fn image_buffer(&self) -> Option<CVPixelBuffer> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_image_buffer(self.0);
            CVPixelBuffer::from_raw(ptr)
        }
    }

    /// Get the frame status from a sample buffer
    ///
    /// Returns the `SCFrameStatus` attachment from the sample buffer,
    /// indicating whether the frame is complete, idle, blank, etc.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::{CMSampleBuffer, SCFrameStatus};
    ///
    /// fn handle_frame(sample: CMSampleBuffer) {
    ///     if let Some(status) = sample.frame_status() {
    ///         match status {
    ///             SCFrameStatus::Complete => {
    ///                 println!("Frame is complete, process it");
    ///             }
    ///             SCFrameStatus::Idle => {
    ///                 println!("Frame is idle, no changes");
    ///             }
    ///             _ => {
    ///                 println!("Frame status: {}", status);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn frame_status(&self) -> Option<SCFrameStatus> {
        unsafe {
            let status = ffi::cm_sample_buffer_get_frame_status(self.0);
            if status >= 0 {
                SCFrameStatus::from_raw(status)
            } else {
                None
            }
        }
    }

    /// Get the display time (mach absolute time) from frame info
    ///
    /// This is the time when the frame was displayed on screen.
    pub fn display_time(&self) -> Option<u64> {
        unsafe {
            let mut value: u64 = 0;
            if ffi::cm_sample_buffer_get_display_time(self.0, &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    /// Get the scale factor (point-to-pixel ratio) from frame info
    ///
    /// This indicates the display's scale factor (e.g., 2.0 for Retina displays).
    pub fn scale_factor(&self) -> Option<f64> {
        unsafe {
            let mut value: f64 = 0.0;
            if ffi::cm_sample_buffer_get_scale_factor(self.0, &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    /// Get the content scale from frame info
    pub fn content_scale(&self) -> Option<f64> {
        unsafe {
            let mut value: f64 = 0.0;
            if ffi::cm_sample_buffer_get_content_scale(self.0, &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    /// Get the content rectangle from frame info
    ///
    /// This is the rectangle of the captured content within the frame.
    pub fn content_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x: f64 = 0.0;
            let mut y: f64 = 0.0;
            let mut width: f64 = 0.0;
            let mut height: f64 = 0.0;
            if ffi::cm_sample_buffer_get_content_rect(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            ) {
                Some(crate::cg::CGRect::new(x, y, width, height))
            } else {
                None
            }
        }
    }

    /// Get the bounding rectangle from frame info
    ///
    /// This is the bounding rectangle of all captured windows.
    pub fn bounding_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x: f64 = 0.0;
            let mut y: f64 = 0.0;
            let mut width: f64 = 0.0;
            let mut height: f64 = 0.0;
            if ffi::cm_sample_buffer_get_bounding_rect(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            ) {
                Some(crate::cg::CGRect::new(x, y, width, height))
            } else {
                None
            }
        }
    }

    /// Get the screen rectangle from frame info
    ///
    /// This is the rectangle of the screen being captured.
    pub fn screen_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x: f64 = 0.0;
            let mut y: f64 = 0.0;
            let mut width: f64 = 0.0;
            let mut height: f64 = 0.0;
            if ffi::cm_sample_buffer_get_screen_rect(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            ) {
                Some(crate::cg::CGRect::new(x, y, width, height))
            } else {
                None
            }
        }
    }

    /// Get the dirty rectangles from frame info
    ///
    /// Dirty rectangles indicate areas of the screen that have changed since the last frame.
    /// This can be used for efficient partial screen updates.
    pub fn dirty_rects(&self) -> Option<Vec<crate::cg::CGRect>> {
        unsafe {
            let mut rects_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut count: usize = 0;
            if ffi::cm_sample_buffer_get_dirty_rects(self.0, &mut rects_ptr, &mut count) {
                if rects_ptr.is_null() || count == 0 {
                    return None;
                }
                let data = rects_ptr as *const f64;
                let mut rects = Vec::with_capacity(count);
                for i in 0..count {
                    let x = *data.add(i * 4);
                    let y = *data.add(i * 4 + 1);
                    let width = *data.add(i * 4 + 2);
                    let height = *data.add(i * 4 + 3);
                    rects.push(crate::cg::CGRect::new(x, y, width, height));
                }
                ffi::cm_sample_buffer_free_dirty_rects(rects_ptr);
                Some(rects)
            } else {
                None
            }
        }
    }

    /// Get the presentation timestamp
    pub fn presentation_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_presentation_timestamp(
                self.0,
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

    /// Get the duration of the sample
    pub fn duration(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_duration(
                self.0,
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

    pub fn is_valid(&self) -> bool {
        unsafe { ffi::cm_sample_buffer_is_valid(self.0) }
    }

    /// Get the number of samples in this buffer
    pub fn num_samples(&self) -> usize {
        unsafe { ffi::cm_sample_buffer_get_num_samples(self.0) }
    }

    /// Get the audio buffer list from this sample
    pub fn audio_buffer_list(&self) -> Option<AudioBufferList> {
        unsafe {
            let mut num_buffers: u32 = 0;
            let mut buffers_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut buffers_len: usize = 0;
            let mut block_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

            ffi::cm_sample_buffer_get_audio_buffer_list(
                self.0,
                &mut num_buffers,
                &mut buffers_ptr,
                &mut buffers_len,
                &mut block_buffer_ptr,
            );

            if num_buffers == 0 {
                None
            } else {
                Some(AudioBufferList {
                    inner: AudioBufferListRaw {
                        num_buffers,
                        buffers_ptr: buffers_ptr.cast::<AudioBuffer>(),
                        buffers_len,
                    },
                    block_buffer_ptr,
                })
            }
        }
    }

    /// Get the data buffer (for compressed data)
    pub fn data_buffer(&self) -> Option<CMBlockBuffer> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_data_buffer(self.0);
            CMBlockBuffer::from_raw(ptr)
        }
    }

    /// Get the decode timestamp of the sample buffer
    pub fn decode_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_decode_timestamp(
                self.0,
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

    /// Get the output presentation timestamp
    pub fn output_presentation_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_output_presentation_timestamp(
                self.0,
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

    /// Set the output presentation timestamp
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the operation fails.
    pub fn set_output_presentation_timestamp(&self, time: CMTime) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_set_output_presentation_timestamp(
                self.0,
                time.value,
                time.timescale,
                time.flags,
                time.epoch,
            );
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Get the size of a specific sample
    pub fn sample_size(&self, index: usize) -> usize {
        unsafe { ffi::cm_sample_buffer_get_sample_size(self.0, index) }
    }

    /// Get the total size of all samples
    pub fn total_sample_size(&self) -> usize {
        unsafe { ffi::cm_sample_buffer_get_total_sample_size(self.0) }
    }

    /// Check if the sample buffer data is ready for access
    pub fn is_data_ready(&self) -> bool {
        unsafe { ffi::cm_sample_buffer_is_ready_for_data_access(self.0) }
    }

    /// Make the sample buffer data ready for access
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the operation fails.
    pub fn make_data_ready(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_make_data_ready(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Get the format description
    pub fn format_description(&self) -> Option<CMFormatDescription> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_format_description(self.0);
            CMFormatDescription::from_raw(ptr)
        }
    }

    /// Get sample timing info for a specific sample
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the timing info cannot be retrieved.
    pub fn sample_timing_info(&self, index: usize) -> Result<CMSampleTimingInfo, i32> {
        unsafe {
            let mut timing_info = CMSampleTimingInfo {
                duration: CMTime::INVALID,
                presentation_time_stamp: CMTime::INVALID,
                decode_time_stamp: CMTime::INVALID,
            };
            let status = ffi::cm_sample_buffer_get_sample_timing_info(
                self.0,
                index,
                &mut timing_info.duration.value,
                &mut timing_info.duration.timescale,
                &mut timing_info.duration.flags,
                &mut timing_info.duration.epoch,
                &mut timing_info.presentation_time_stamp.value,
                &mut timing_info.presentation_time_stamp.timescale,
                &mut timing_info.presentation_time_stamp.flags,
                &mut timing_info.presentation_time_stamp.epoch,
                &mut timing_info.decode_time_stamp.value,
                &mut timing_info.decode_time_stamp.timescale,
                &mut timing_info.decode_time_stamp.flags,
                &mut timing_info.decode_time_stamp.epoch,
            );
            if status == 0 {
                Ok(timing_info)
            } else {
                Err(status)
            }
        }
    }

    /// Get all sample timing info as a vector
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if any timing info cannot be retrieved.
    pub fn sample_timing_info_array(&self) -> Result<Vec<CMSampleTimingInfo>, i32> {
        let num_samples = self.num_samples();
        let mut result = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            result.push(self.sample_timing_info(i)?);
        }
        Ok(result)
    }

    /// Invalidate the sample buffer
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the invalidation fails.
    pub fn invalidate(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_invalidate(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Create a copy with new timing information
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the copy cannot be created.
    pub fn create_copy_with_new_timing(
        &self,
        timing_info: &[CMSampleTimingInfo],
    ) -> Result<Self, i32> {
        unsafe {
            let mut new_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cm_sample_buffer_create_copy_with_new_timing(
                self.0,
                timing_info.len(),
                timing_info.as_ptr().cast::<std::ffi::c_void>(),
                &mut new_buffer_ptr,
            );
            if status == 0 && !new_buffer_ptr.is_null() {
                Ok(Self(new_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Copy PCM audio data into an audio buffer list
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the copy operation fails.
    pub fn copy_pcm_data_into_audio_buffer_list(
        &self,
        frame_offset: i32,
        num_frames: i32,
        buffer_list: &mut AudioBufferList,
    ) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_copy_pcm_data_into_audio_buffer_list(
                self.0,
                frame_offset,
                num_frames,
                (buffer_list as *mut AudioBufferList).cast::<std::ffi::c_void>(),
            );
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }
}

impl Drop for CMSampleBuffer {
    fn drop(&mut self) {
        unsafe {
            ffi::cm_sample_buffer_release(self.0);
        }
    }
}

unsafe impl Send for CMSampleBuffer {}
unsafe impl Sync for CMSampleBuffer {}

impl fmt::Display for CMSampleBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CMSampleBuffer(pts: {}, duration: {}, samples: {})",
            self.presentation_timestamp(),
            self.duration(),
            self.num_samples()
        )
    }
}
