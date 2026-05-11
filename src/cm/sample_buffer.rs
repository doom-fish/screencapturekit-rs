//! `CMSampleBuffer` - Container for media samples

use super::ffi;
use super::{
    AudioBuffer, AudioBufferList, AudioBufferListRaw, CMBlockBuffer, CMFormatDescription,
    CMSampleTimingInfo, CMTime, SCFrameStatus,
};
use crate::cv::CVPixelBuffer;
use std::fmt;

/// Bit flags marking which fields the batched [`CMSampleBuffer::frame_info`]
/// fetch managed to populate. Mirrors `FrameInfoFieldBits` in the Swift
/// bridge — keep them in sync.
struct FrameInfoFields;

impl FrameInfoFields {
    const STATUS: u32 = 1 << 0;
    const DISPLAY_TIME: u32 = 1 << 1;
    const SCALE_FACTOR: u32 = 1 << 2;
    const CONTENT_SCALE: u32 = 1 << 3;
    const CONTENT_RECT: u32 = 1 << 4;
    const BOUNDING_RECT: u32 = 1 << 5;
    const SCREEN_RECT: u32 = 1 << 6;
    const PRESENTER_OVERLAY_RECT: u32 = 1 << 7;
}

/// Snapshot of every `SCStreamFrameInfo` attachment on a sample buffer.
///
/// Returned by [`CMSampleBuffer::frame_info`]. Each field is `Some` when the
/// underlying attachment was present (depends on macOS version, output type,
/// and stream configuration); `None` indicates the attachment was missing.
//
// Eq cannot be derived because of the f64-bearing fields (CGRect, f64). The
// allow keeps clippy happy under -D warnings without forcing us to ship a
// hand-rolled Eq that would lie about NaN handling.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FrameInfo {
    /// `SCStreamFrameInfo.status` — frame completeness / idle state.
    pub frame_status: Option<SCFrameStatus>,
    /// `SCStreamFrameInfo.displayTime` — mach absolute time the frame was
    /// composited.
    pub display_time: Option<u64>,
    /// `SCStreamFrameInfo.scaleFactor` — display scale (e.g. 2.0 for Retina).
    pub scale_factor: Option<f64>,
    /// `SCStreamFrameInfo.contentScale` — capture scale relative to the
    /// source content.
    pub content_scale: Option<f64>,
    /// `SCStreamFrameInfo.contentRect` — captured content within the frame.
    pub content_rect: Option<crate::cg::CGRect>,
    /// `SCStreamFrameInfo.boundingRect` — bounding rect of all captured
    /// windows (macOS 14.0+).
    pub bounding_rect: Option<crate::cg::CGRect>,
    /// `SCStreamFrameInfo.screenRect` — full screen rect (macOS 13.1+).
    pub screen_rect: Option<crate::cg::CGRect>,
    /// `SCStreamFrameInfo.presenterOverlayContentRect` — Presenter Overlay
    /// bounding rect (macOS 14.2+).
    pub presenter_overlay_content_rect: Option<crate::cg::CGRect>,
}

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

    /// Get the Presenter Overlay content rectangle from frame info (macOS 14.2+).
    ///
    /// When a stream is configured with Presenter Overlay (see
    /// [`SCStreamConfiguration::set_presenter_overlay_privacy_alert_setting`](
    /// crate::stream::configuration::SCStreamConfiguration::set_presenter_overlay_privacy_alert_setting)),
    /// `ScreenCaptureKit` attaches the overlay's bounding rectangle (within the
    /// captured frame) to each delivered sample. Returns `None` when the
    /// attachment is missing — typically because the stream isn't using
    /// Presenter Overlay or no overlay is currently visible.
    ///
    /// This complements the existing [`content_rect`](Self::content_rect),
    /// [`bounding_rect`](Self::bounding_rect), [`screen_rect`](Self::screen_rect),
    /// and [`dirty_rects`](Self::dirty_rects) accessors and rounds out the
    /// `SCStreamFrameInfo` attachment surface (9 of 9 keys exposed).
    #[cfg(feature = "macos_14_2")]
    pub fn presenter_overlay_content_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x: f64 = 0.0;
            let mut y: f64 = 0.0;
            let mut width: f64 = 0.0;
            let mut height: f64 = 0.0;
            if ffi::cm_sample_buffer_get_presenter_overlay_content_rect(
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

    /// Read every `SCStreamFrameInfo` attachment in a single FFI round-trip.
    ///
    /// Each per-attribute accessor (`frame_status`, `display_time`,
    /// `scale_factor`, `content_scale`, `content_rect`, `bounding_rect`,
    /// `screen_rect`, `presenter_overlay_content_rect`) re-fetches the
    /// `CMSampleBuffer` attachment array and re-bridges the dictionary from
    /// `CoreFoundation` to Swift. Calling five of them per frame measured at
    /// ~11 µs on a captured 1080p frame; this batched method does it once for
    /// ~3 µs (~4× faster). Use this in any per-frame consumer that reads more
    /// than one attribute.
    ///
    /// Returns `None` when the sample buffer carries no attachments at all.
    /// Otherwise returns a [`FrameInfo`] whose individual fields may still be
    /// `None` if a particular attachment was missing (e.g. older macOS
    /// versions or non-screen samples).
    pub fn frame_info(&self) -> Option<FrameInfo> {
        let mut fields: u32 = 0;
        let mut status: i32 = 0;
        let mut display_time: u64 = 0;
        let mut scale_factor: f64 = 0.0;
        let mut content_scale: f64 = 0.0;
        let mut content_rect: [f64; 4] = [0.0; 4];
        let mut bounding_rect: [f64; 4] = [0.0; 4];
        let mut screen_rect: [f64; 4] = [0.0; 4];
        let mut presenter_overlay_rect: [f64; 4] = [0.0; 4];

        let any = unsafe {
            ffi::cm_sample_buffer_get_frame_info(
                self.0,
                &mut fields,
                &mut status,
                &mut display_time,
                &mut scale_factor,
                &mut content_scale,
                content_rect.as_mut_ptr(),
                bounding_rect.as_mut_ptr(),
                screen_rect.as_mut_ptr(),
                presenter_overlay_rect.as_mut_ptr(),
            )
        };

        if !any {
            return None;
        }

        let to_rect = |arr: [f64; 4]| crate::cg::CGRect::new(arr[0], arr[1], arr[2], arr[3]);

        Some(FrameInfo {
            frame_status: ((fields & FrameInfoFields::STATUS) != 0)
                .then(|| SCFrameStatus::from_raw(status))
                .flatten(),
            display_time: ((fields & FrameInfoFields::DISPLAY_TIME) != 0).then_some(display_time),
            scale_factor: ((fields & FrameInfoFields::SCALE_FACTOR) != 0).then_some(scale_factor),
            content_scale: ((fields & FrameInfoFields::CONTENT_SCALE) != 0)
                .then_some(content_scale),
            content_rect: ((fields & FrameInfoFields::CONTENT_RECT) != 0)
                .then(|| to_rect(content_rect)),
            bounding_rect: ((fields & FrameInfoFields::BOUNDING_RECT) != 0)
                .then(|| to_rect(bounding_rect)),
            screen_rect: ((fields & FrameInfoFields::SCREEN_RECT) != 0)
                .then(|| to_rect(screen_rect)),
            presenter_overlay_content_rect: ((fields & FrameInfoFields::PRESENTER_OVERLAY_RECT)
                != 0)
                .then(|| to_rect(presenter_overlay_rect)),
        })
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
