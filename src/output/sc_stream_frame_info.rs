mod internal {

    #![allow(non_snake_case)]

    use core::fmt;
    use std::mem;

    use core_foundation::{
        array::{CFArray, CFArrayRef},
        base::{CFType, FromVoid, ItemRef, TCFType},
        error::CFError,
        number::CFNumber,
        string::{CFString, CFStringRef},
    };
    use core_graphics::display::{CFDictionary, CGPoint, CGRect, CGSize};

    use core_media_rs::cm_sample_buffer::{CMSampleBuffer, CMSampleBufferRef};

    use crate::utils::{error::create_cf_error, objc::get_concrete_from_void};

    extern "C" {
        //A key to retrieve the status of a video frame.
        static SCStreamFrameInfoStatus: CFStringRef;
        //A key to retrieve the display time of a video frame.
        static SCStreamFrameInfoDisplayTime: CFStringRef;

        //A key to retrieve the scale factor of a video frame.
        static SCStreamFrameInfoScaleFactor: CFStringRef;
        //A key to retrieve the content scale of a video frame.
        static SCStreamFrameInfoContentScale: CFStringRef;
        //A key to retrieve the content rectangle of a video frame.
        static SCStreamFrameInfoContentRect: CFStringRef;
        //A key to retrieve the bounding rectangle for a video frame.
        static SCStreamFrameInfoBoundingRect: CFStringRef;
        //A key to retrieve the onscreen location of captured content.
        static SCStreamFrameInfoScreenRect: CFStringRef;
        //A key to retrieve the areas of a video frame that contain changes.
        static SCStreamFrameInfoDirtyRects: CFStringRef;
        pub fn CMSampleBufferGetSampleAttachmentsArray(
            sample: CMSampleBufferRef,
            create: u8,
        ) -> CFArrayRef;
    }
    pub struct SCStreamFrameInfo {
        data: CFArray<CFDictionary<CFString, CFType>>,
    }

    impl fmt::Debug for SCStreamFrameInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut s = f.debug_struct("SCStreamFrameInfo");

            s.field("status", &self.internal_status());
            s.field("display_time", &self.internal_display_time());
            if let Some(ref scale_factor) = self.internal_scale_factor() {
                s.field("scale_factor", &scale_factor);
            }
            if let Some(ref content_scale) = self.internal_content_scale() {
                s.field("content_scale", &content_scale);
            }
            if let Some(ref bounding_rect) = self.internal_bounding_rect() {
                s.field("bounding_rect", &bounding_rect);
            }
            if let Some(ref content_rect) = self.internal_content_rect() {
                s.field("content_rect", &content_rect);
            }
            if let Some(ref screen_rect) = self.internal_screen_rect() {
                s.field("screen_rect", &screen_rect);
            }
            if let Some(ref dirty_rects) = self.internal_dirty_rects() {
                s.field("dirty_rect", &dirty_rects);
            }
            s.finish()
        }
    }
    impl SCStreamFrameInfo {
        pub(crate) fn internal_sample_from_buffer(
            sample_buffer: &CMSampleBuffer,
        ) -> Result<Self, CFError> {
            let data: CFArray<CFDictionary<CFString, CFType>> = unsafe {
                CFArray::wrap_under_get_rule(CMSampleBufferGetSampleAttachmentsArray(
                    sample_buffer.as_concrete_TypeRef(),
                    1,
                ))
            };

            if data.is_empty() {};
            if let Some(ref dict) = data.get(0) {
                let KEY_FOR_SC_STREAM_STATUS =
                    unsafe { CFString::from_void(SCStreamFrameInfoStatus.cast()) };

                if dict.contains_key(&KEY_FOR_SC_STREAM_STATUS) {
                    return Ok(Self { data });
                }
            }
            Err(create_cf_error(
                "could not get CMSampleBufferSampleAttachmentsArray",
                0,
            ))
        }
        fn data(&self) -> ItemRef<'_, CFDictionary<CFString, CFType>> {
            self.data.get(0).expect("should have data")
        }

        pub(crate) fn internal_status(&self) -> SCFrameStatus {
            unsafe {
                self.data()
                    .get(SCStreamFrameInfoStatus)
                    .downcast()
                    .and_then(|n: CFNumber| n.to_i64())
                    .map(|n| mem::transmute::<i64, SCFrameStatus>(n))
                    .expect("could not get status")
            }
        }
        pub(crate) fn internal_display_time(&self) -> u64 {
            unsafe {
                self.data()
                    .get(SCStreamFrameInfoDisplayTime)
                    .downcast()
                    .and_then(|n: CFNumber| n.to_i64())
                    .and_then(|n| u64::try_from(n).ok())
                    .expect("could not get display time")
            }
        }
        pub(crate) fn internal_scale_factor(&self) -> Option<f64> {
            unsafe {
                self.data()
                    .find(SCStreamFrameInfoScaleFactor)
                    .and_then(|n| n.downcast())
                    .and_then(|n: CFNumber| n.to_f64())
            }
        }
        pub(crate) fn internal_content_scale(&self) -> Option<f64> {
            self.data()
                .find(unsafe { SCStreamFrameInfoContentScale })
                .and_then(|n| n.downcast())
                .and_then(|n: CFNumber| n.to_f64())
        }
        pub(crate) fn internal_bounding_rect(&self) -> Option<CGRect> {
            self.data()
                .find(unsafe { SCStreamFrameInfoBoundingRect })
                .and_then(|n| n.downcast())
                .map(dict_to_cg_rect)
        }
        pub(crate) fn internal_content_rect(&self) -> Option<CGRect> {
            self.data()
                .find(unsafe { SCStreamFrameInfoContentRect })
                .and_then(|n| n.downcast())
                .map(dict_to_cg_rect)
        }
        pub(crate) fn internal_screen_rect(&self) -> Option<CGRect> {
            self.data()
                .find(unsafe { SCStreamFrameInfoScreenRect })
                .and_then(|n| n.downcast())
                .map(dict_to_cg_rect)
        }
        pub(crate) fn internal_dirty_rects(&self) -> Option<Vec<CGRect>> {
            unsafe {
                self.data()
                    .find(SCStreamFrameInfoDirtyRects)
                    .and_then(|a| a.downcast::<CFArray>())
                    .map(|a| {
                        a.into_iter()
                            .map(|x| get_concrete_from_void(x.to_owned()))
                            .map(dict_to_cg_rect)
                            .collect()
                    })
            }
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    #[allow(clippy::needless_pass_by_value)]
    fn dict_to_cg_rect(cf_rect_raw: CFDictionary) -> CGRect {
        let cf_rect = unsafe {
            CFDictionary::<CFString, CFNumber>::wrap_under_get_rule(
                cf_rect_raw.as_concrete_TypeRef(),
            )
        };
        let x = cf_rect
            .get(CFString::from("X"))
            .to_f64()
            .map(f64::round)
            .expect("could not get x");
        let y = cf_rect
            .get(CFString::from("Y"))
            .to_f64()
            .map(f64::round)
            .expect("could not get u");
        let width = cf_rect
            .get(CFString::from("Width"))
            .to_f64()
            .map(f64::round)
            .expect("could not get width");
        let height = cf_rect
            .get(CFString::from("Height"))
            .to_f64()
            .map(f64::round)
            .expect("could not get height");
        CGRect::new(&CGPoint::new(x, y), &CGSize::new(width, height))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[repr(i64)]
    pub enum SCFrameStatus {
        // A status that indicates the system successfully generated a new frame.
        Complete,
        // A status that indicates the system didn’t generate a new frame because the display didn’t change.
        Idle,
        // A status that indicates the system didn’t generate a new frame because the display is blank.
        Blank,
        // A status that indicates the system didn’t generate a new frame because you suspended updates.
        Suspended,
        // A status that indicates the frame is the first one sent after the stream starts.
        Started,
        // A status that indicates the frame is in a stopped state.
        Stopped,
    }
}
use core_foundation::error::CFError;
use core_graphics::display::CGRect;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
pub use internal::SCFrameStatus;
pub use internal::SCStreamFrameInfo;

impl SCStreamFrameInfo {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn from_sample_buffer(sample_buffer: &CMSampleBuffer) -> Result<Self, CFError> {
        Self::internal_sample_from_buffer(sample_buffer)
    }
    /// Returns the status of this [`SCStreamFrameInfo`].
    pub fn status(&self) -> SCFrameStatus {
        self.internal_status()
    }
    pub fn display_time(&self) -> u64 {
        self.internal_display_time()
    }
    pub fn scale_factor(&self) -> Option<f64> {
        self.internal_scale_factor()
    }
    pub fn content_scale(&self) -> Option<f64> {
        self.internal_content_scale()
    }
    pub fn bounding_rect(&self) -> Option<CGRect> {
        self.internal_bounding_rect()
    }
    pub fn content_rect(&self) -> Option<CGRect> {
        self.internal_content_rect()
    }
    pub fn screen_rect(&self) -> Option<CGRect> {
        self.internal_screen_rect()
    }
    pub fn dirty_rects(&self) -> Option<Vec<CGRect>> {
        self.internal_dirty_rects()
    }
}
