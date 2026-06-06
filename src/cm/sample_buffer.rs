//! `CMSampleBuffer` — re-exported from [`apple_cf::cm::CMSampleBuffer`] plus
//! `ScreenCaptureKit`-specific extension traits for the `SCStreamFrameInfo`
//! attachment readers and the few sample-buffer accessors that aren't
//! framework-agnostic enough to live in `apple-cf` yet.
//!
//! Bring [`CMSampleBufferSCExt`] into scope to call `frame_status()`,
//! `display_time()`, `frame_info()`, etc. on any `CMSampleBuffer` carrying
//! `ScreenCaptureKit` attachments.
//!
//! Bring [`CMSampleBufferExt`] into scope for the
//! `image_buffer()`/`audio_buffer_list()`/`make_data_ready()` accessors
//! that are pending an apple-cf v0.2 API addition.

use super::ffi;
use super::{
    AudioBuffer, AudioBufferList, AudioBufferListRaw, CMBlockBuffer, CMSampleTimingInfo, CMTime,
    SCFrameStatus,
};
use crate::cv::CVPixelBuffer;

/// Re-exported `CMSampleBuffer` — same opaque-pointer wrapper used across
/// the doom-fish suite.
pub use apple_cf::cm::CMSampleBuffer;

// ------------------------------------------------------------------
// FrameInfoFields — bit flags for the batched frame_info reader.
// ------------------------------------------------------------------

/// Bit flags marking which fields the batched [`CMSampleBufferSCExt::frame_info`]
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
/// Returned by [`CMSampleBufferSCExt::frame_info`]. Each field is `Some` when
/// the underlying attachment was present (depends on macOS version, output
/// type, and stream configuration); `None` indicates the attachment was
/// missing.
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

// ------------------------------------------------------------------
// CMSampleBufferSCExt — ScreenCaptureKit-specific attachment readers.
// ------------------------------------------------------------------

/// Extension trait that exposes `SCStreamFrameInfo` attachment accessors on
/// any [`CMSampleBuffer`] produced by `ScreenCaptureKit`.
///
/// These are SC-specific by design: they read attachment keys defined on
/// `SCStreamFrameInfo` and are meaningless on sample buffers from other
/// sources (videotoolbox, `AVFoundation` capture, etc.).
pub trait CMSampleBufferSCExt {
    /// `SCStreamFrameInfo.status` attachment.
    fn frame_status(&self) -> Option<SCFrameStatus>;
    /// `SCStreamFrameInfo.displayTime` attachment.
    fn display_time(&self) -> Option<u64>;
    /// `SCStreamFrameInfo.scaleFactor` attachment.
    fn scale_factor(&self) -> Option<f64>;
    /// `SCStreamFrameInfo.contentScale` attachment.
    fn content_scale(&self) -> Option<f64>;
    /// `SCStreamFrameInfo.contentRect` attachment.
    fn content_rect(&self) -> Option<crate::cg::CGRect>;
    /// `SCStreamFrameInfo.boundingRect` attachment.
    fn bounding_rect(&self) -> Option<crate::cg::CGRect>;
    /// `SCStreamFrameInfo.screenRect` attachment.
    fn screen_rect(&self) -> Option<crate::cg::CGRect>;
    /// `SCStreamFrameInfo.presenterOverlayContentRect` attachment.
    fn presenter_overlay_content_rect(&self) -> Option<crate::cg::CGRect>;
    /// `SCStreamFrameInfo.dirtyRects` attachment.
    fn dirty_rects(&self) -> Option<Vec<crate::cg::CGRect>>;
    /// Read every populated `SCStreamFrameInfo` attachment in a single
    /// FFI round-trip.
    fn frame_info(&self) -> Option<FrameInfo>;
}

impl CMSampleBufferSCExt for CMSampleBuffer {
    fn frame_status(&self) -> Option<SCFrameStatus> {
        unsafe {
            let status = ffi::cm_sample_buffer_get_frame_status(self.as_ptr());
            if status >= 0 {
                SCFrameStatus::from_raw(status)
            } else {
                None
            }
        }
    }

    fn display_time(&self) -> Option<u64> {
        unsafe {
            let mut value: u64 = 0;
            if ffi::cm_sample_buffer_get_display_time(self.as_ptr(), &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    fn scale_factor(&self) -> Option<f64> {
        unsafe {
            let mut value: f64 = 0.0;
            if ffi::cm_sample_buffer_get_scale_factor(self.as_ptr(), &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    fn content_scale(&self) -> Option<f64> {
        unsafe {
            let mut value: f64 = 0.0;
            if ffi::cm_sample_buffer_get_content_scale(self.as_ptr(), &mut value) {
                Some(value)
            } else {
                None
            }
        }
    }

    fn content_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut w = 0.0;
            let mut h = 0.0;
            if ffi::cm_sample_buffer_get_content_rect(self.as_ptr(), &mut x, &mut y, &mut w, &mut h)
            {
                Some(crate::cg::CGRect::new(x, y, w, h))
            } else {
                None
            }
        }
    }

    fn bounding_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut w = 0.0;
            let mut h = 0.0;
            if ffi::cm_sample_buffer_get_bounding_rect(
                self.as_ptr(),
                &mut x,
                &mut y,
                &mut w,
                &mut h,
            ) {
                Some(crate::cg::CGRect::new(x, y, w, h))
            } else {
                None
            }
        }
    }

    fn screen_rect(&self) -> Option<crate::cg::CGRect> {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut w = 0.0;
            let mut h = 0.0;
            if ffi::cm_sample_buffer_get_screen_rect(self.as_ptr(), &mut x, &mut y, &mut w, &mut h)
            {
                Some(crate::cg::CGRect::new(x, y, w, h))
            } else {
                None
            }
        }
    }

    fn presenter_overlay_content_rect(&self) -> Option<crate::cg::CGRect> {
        #[cfg(feature = "macos_14_2")]
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut w = 0.0;
            let mut h = 0.0;
            if ffi::cm_sample_buffer_get_presenter_overlay_content_rect(
                self.as_ptr(),
                &mut x,
                &mut y,
                &mut w,
                &mut h,
            ) {
                Some(crate::cg::CGRect::new(x, y, w, h))
            } else {
                None
            }
        }
        #[cfg(not(feature = "macos_14_2"))]
        None
    }

    fn dirty_rects(&self) -> Option<Vec<crate::cg::CGRect>> {
        unsafe {
            let mut rects_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut count: usize = 0;
            if !ffi::cm_sample_buffer_get_dirty_rects(self.as_ptr(), &mut rects_ptr, &mut count) {
                return None;
            }
            if rects_ptr.is_null() || count == 0 {
                return None;
            }
            let rects_typed = rects_ptr.cast::<f64>();
            let mut rects = Vec::with_capacity(count);
            for i in 0..count {
                let base = rects_typed.add(i * 4);
                rects.push(crate::cg::CGRect::new(
                    *base,
                    *base.add(1),
                    *base.add(2),
                    *base.add(3),
                ));
            }
            ffi::cm_sample_buffer_free_dirty_rects(rects_ptr);
            Some(rects)
        }
    }

    fn frame_info(&self) -> Option<FrameInfo> {
        unsafe {
            let mut fields: u32 = 0;
            let mut status: i32 = 0;
            let mut display_time: u64 = 0;
            let mut scale_factor: f64 = 0.0;
            let mut content_scale: f64 = 0.0;
            let mut content_rect = [0.0_f64; 4];
            let mut bounding_rect = [0.0_f64; 4];
            let mut screen_rect = [0.0_f64; 4];
            let mut presenter_overlay_rect = [0.0_f64; 4];
            if !ffi::cm_sample_buffer_get_frame_info(
                self.as_ptr(),
                &mut fields,
                &mut status,
                &mut display_time,
                &mut scale_factor,
                &mut content_scale,
                content_rect.as_mut_ptr(),
                bounding_rect.as_mut_ptr(),
                screen_rect.as_mut_ptr(),
                presenter_overlay_rect.as_mut_ptr(),
            ) {
                return None;
            }
            let to_rect = |a: [f64; 4]| crate::cg::CGRect::new(a[0], a[1], a[2], a[3]);
            Some(FrameInfo {
                frame_status: ((fields & FrameInfoFields::STATUS) != 0)
                    .then(|| SCFrameStatus::from_raw(status))
                    .flatten(),
                display_time: ((fields & FrameInfoFields::DISPLAY_TIME) != 0)
                    .then_some(display_time),
                scale_factor: ((fields & FrameInfoFields::SCALE_FACTOR) != 0)
                    .then_some(scale_factor),
                content_scale: ((fields & FrameInfoFields::CONTENT_SCALE) != 0)
                    .then_some(content_scale),
                content_rect: ((fields & FrameInfoFields::CONTENT_RECT) != 0)
                    .then(|| to_rect(content_rect)),
                bounding_rect: ((fields & FrameInfoFields::BOUNDING_RECT) != 0)
                    .then(|| to_rect(bounding_rect)),
                screen_rect: ((fields & FrameInfoFields::SCREEN_RECT) != 0)
                    .then(|| to_rect(screen_rect)),
                presenter_overlay_content_rect: ((fields
                    & FrameInfoFields::PRESENTER_OVERLAY_RECT)
                    != 0)
                    .then(|| to_rect(presenter_overlay_rect)),
            })
        }
    }
}

// ------------------------------------------------------------------
// CMSampleBufferExt — generic accessors not yet in apple-cf.
// ------------------------------------------------------------------

/// Extension trait carrying generic `CMSampleBuffer` accessors that aren't
/// available on [`apple_cf::cm::CMSampleBuffer`] yet (planned for an
/// `apple-cf` v0.2 release).
pub trait CMSampleBufferExt {
    /// Construct a sample buffer wrapping a `CVPixelBuffer`.
    ///
    /// # Errors
    ///
    /// Returns the underlying `OSStatus` if `CoreMedia` fails to create the
    /// sample buffer.
    fn create_for_image_buffer(
        image_buffer: &CVPixelBuffer,
        presentation_time: CMTime,
        duration: CMTime,
    ) -> Result<Self, i32>
    where
        Self: Sized;

    /// Borrow the attached `CVPixelBuffer`, if any.
    fn image_buffer(&self) -> Option<CVPixelBuffer>;

    /// Read the audio sample buffer's underlying `AudioBufferList`, if any.
    fn audio_buffer_list(&self) -> Option<AudioBufferList>;

    /// Output presentation timestamp (after timing adjustments).
    fn output_presentation_timestamp(&self) -> CMTime;

    /// Override the output presentation timestamp.
    ///
    /// # Errors
    ///
    /// Returns the underlying `OSStatus` if `CoreMedia` rejects the new value.
    fn set_output_presentation_timestamp(&self, time: CMTime) -> Result<(), i32>;

    /// Size of one sample at `index` in bytes.
    fn sample_size(&self, index: usize) -> usize;

    /// Sum of all sample sizes in this buffer.
    fn total_sample_size(&self) -> usize;

    /// Whether the underlying data is ready for reading.
    fn is_data_ready(&self) -> bool;

    /// Mark the underlying data as ready (flushes any pending make-data-ready
    /// callbacks).
    ///
    /// # Errors
    ///
    /// Returns the underlying `OSStatus` if `CoreMedia` reports failure.
    fn make_data_ready(&self) -> Result<(), i32>;

    /// Read the timing info for the sample at `index`.
    ///
    /// # Errors
    ///
    /// Returns the underlying `OSStatus` if `index` is out of range.
    fn sample_timing_info(&self, index: usize) -> Result<CMSampleTimingInfo, i32>;

    /// Build an [`apple_cf::cg::CGImage`] from the buffer's attached
    /// `CVImageBuffer`.
    ///
    /// Backed by `VTCreateCGImageFromCVPixelBuffer`, which understands every
    /// pixel format `ScreenCaptureKit` (or any other `CoreMedia` producer) can
    /// emit — BGRA, 420v YCbCr 8-bit bi-planar video range, l10r 10-bit ARGB,
    /// etc. — and uses Apple's hardware path when one exists. The resulting
    /// `CGImage` is `IOSurface`-backed when the source was, so passing it
    /// straight into `ImageIO` (`CGImageDestinationAddImage` / `imageio-rs`
    /// `ImageDestination::add_cg_image`) or into Metal sampling avoids any
    /// host-side pixel copy.
    ///
    /// Returns the canonical `apple_cf::cg::CGImage` (the same type used by
    /// `imageio-rs` and every other doom-fish suite crate that consumes
    /// `CGImage`s), so the result flows straight into safe APIs with no
    /// pointer juggling at the callsite.
    ///
    /// # Errors
    ///
    /// Returns the underlying `OSStatus` from `VTCreateCGImageFromCVPixelBuffer`
    /// (or `-12731` `kCMSampleBufferError_NoSampleBufferContent` when the
    /// buffer has no image buffer attached — typical for audio-only or
    /// timing-metadata-only samples).
    fn cg_image(&self) -> Result<apple_cf::cg::CGImage, i32>;
}

impl CMSampleBufferExt for CMSampleBuffer {
    fn create_for_image_buffer(
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
                Self::from_raw(sample_buffer_ptr).ok_or(status)
            } else {
                Err(status)
            }
        }
    }

    fn image_buffer(&self) -> Option<CVPixelBuffer> {
        unsafe {
            // SAFETY: cm_sample_buffer_get_image_buffer returns a +1
            // (passRetained) CVImageBuffer; CVPixelBuffer::from_raw adopts that
            // +1 reference, so ownership is balanced (released on drop).
            let ptr = ffi::cm_sample_buffer_get_image_buffer(self.as_ptr());
            CVPixelBuffer::from_raw(ptr)
        }
    }

    fn audio_buffer_list(&self) -> Option<AudioBufferList> {
        unsafe {
            let mut num_buffers: u32 = 0;
            let mut buffers_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut buffers_len: usize = 0;
            let mut block_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

            ffi::cm_sample_buffer_get_audio_buffer_list(
                self.as_ptr(),
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

    fn output_presentation_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_output_presentation_timestamp(
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

    fn set_output_presentation_timestamp(&self, time: CMTime) -> Result<(), i32> {
        let status = unsafe {
            ffi::cm_sample_buffer_set_output_presentation_timestamp(
                self.as_ptr(),
                time.value,
                time.timescale,
                time.flags,
                time.epoch,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    fn sample_size(&self, index: usize) -> usize {
        unsafe { ffi::cm_sample_buffer_get_sample_size(self.as_ptr(), index) }
    }

    fn total_sample_size(&self) -> usize {
        unsafe { ffi::cm_sample_buffer_get_total_sample_size(self.as_ptr()) }
    }

    fn is_data_ready(&self) -> bool {
        unsafe { ffi::cm_sample_buffer_is_ready_for_data_access(self.as_ptr()) }
    }

    fn make_data_ready(&self) -> Result<(), i32> {
        let status = unsafe { ffi::cm_sample_buffer_make_data_ready(self.as_ptr()) };
        if status == 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    fn sample_timing_info(&self, index: usize) -> Result<CMSampleTimingInfo, i32> {
        unsafe {
            let mut dur_v: i64 = 0;
            let mut dur_s: i32 = 0;
            let mut dur_f: u32 = 0;
            let mut dur_e: i64 = 0;
            let mut pts_v: i64 = 0;
            let mut pts_s: i32 = 0;
            let mut pts_f: u32 = 0;
            let mut pts_e: i64 = 0;
            let mut dts_v: i64 = 0;
            let mut dts_s: i32 = 0;
            let mut dts_f: u32 = 0;
            let mut dts_e: i64 = 0;
            let status = ffi::cm_sample_buffer_get_sample_timing_info(
                self.as_ptr(),
                index,
                &mut dur_v,
                &mut dur_s,
                &mut dur_f,
                &mut dur_e,
                &mut pts_v,
                &mut pts_s,
                &mut pts_f,
                &mut pts_e,
                &mut dts_v,
                &mut dts_s,
                &mut dts_f,
                &mut dts_e,
            );
            if status == 0 {
                Ok(CMSampleTimingInfo {
                    duration: CMTime {
                        value: dur_v,
                        timescale: dur_s,
                        flags: dur_f,
                        epoch: dur_e,
                    },
                    presentation_time_stamp: CMTime {
                        value: pts_v,
                        timescale: pts_s,
                        flags: pts_f,
                        epoch: pts_e,
                    },
                    decode_time_stamp: CMTime {
                        value: dts_v,
                        timescale: dts_s,
                        flags: dts_f,
                        epoch: dts_e,
                    },
                })
            } else {
                Err(status)
            }
        }
    }

    fn cg_image(&self) -> Result<apple_cf::cg::CGImage, i32> {
        unsafe {
            let mut status: i32 = 0;
            let ptr = ffi::cm_sample_buffer_create_cg_image(self.as_ptr(), &mut status);
            if !ptr.is_null() && status == 0 {
                // Safety: the Swift bridge returns a retained CGImage on
                // success; passing it straight to CGImage::from_raw takes
                // ownership of that refcount.
                Ok(apple_cf::cg::CGImage::from_raw(ptr.cast_mut()))
            } else {
                Err(status)
            }
        }
    }
}

// ------------------------------------------------------------------
// data_buffer wrapper that returns the *local* CMBlockBuffer type
// (for backward compat). apple_cf::cm::CMSampleBuffer also has its own
// data_buffer() returning apple_cf::cm::CMBlockBuffer; the local one
// here returns crate::cm::CMBlockBuffer which currently is its own
// type. (Merging the two is Phase 4.)
// ------------------------------------------------------------------

/// Convenience: like [`apple_cf::cm::CMSampleBuffer::data_buffer`] but
/// returns the local `crate::cm::CMBlockBuffer` (which is currently a
/// different wrapper around the same underlying type).
pub trait CMSampleBufferDataBufferExt {
    fn data_buffer_local(&self) -> Option<CMBlockBuffer>;
}

impl CMSampleBufferDataBufferExt for CMSampleBuffer {
    fn data_buffer_local(&self) -> Option<CMBlockBuffer> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_data_buffer(self.as_ptr());
            if ptr.is_null() {
                return None;
            }
            // `CMSampleBufferGetDataBuffer` returns a +0 (unretained) reference.
            // `CMBlockBuffer::from_raw` adopts a +1 reference and releases on
            // drop, so we must retain first to keep the refcount balanced.
            // (Mirrors apple-cf's own `CMSampleBuffer::data_buffer`.)
            let retained = ffi::cm_block_buffer_retain(ptr);
            CMBlockBuffer::from_raw(retained)
        }
    }
}
