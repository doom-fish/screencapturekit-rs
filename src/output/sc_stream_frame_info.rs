mod internal {

    #![allow(non_snake_case)]

    use core::fmt;
    use std::mem;

    use core_foundation::{
        array::{CFArray, CFArrayRef},
        base::{CFType, TCFType, TCFTypeRef},
        error::CFError,
        number::CFNumber,
        string::{CFString, CFStringRef},
    };
    use core_graphics::display::{CFDictionary, CFDictionaryRef, CGPoint, CGRect, CGSize};

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
        data: CFDictionary<CFString, CFType>,
    }
    impl fmt::Debug for SCStreamFrameInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SCStreamFrameInfo")
                .field("status", &self.internal_status())
                .field("display_time", &self.internal_display_time())
                .field("scale_factor", &self.internal_scale_factor())
                .field("content_scale", &self.internal_content_scale())
                .field("bounding_rect", &self.internal_bounding_rect())
                .field("content_rect", &self.internal_content_rect())
                .field("screen_rect", &self.internal_screen_rect())
                .field("dirty_rect", &self.internal_dirty_rects())
                .finish()
        }
    }
    impl SCStreamFrameInfo {
        pub(crate) fn internal_from_buffer(
            sample_buffer: &CMSampleBuffer,
        ) -> Result<Self, CFError> {
            let array: CFArray = unsafe {
                CFArray::wrap_under_get_rule(CMSampleBufferGetSampleAttachmentsArray(
                    sample_buffer.as_concrete_TypeRef(),
                    1,
                ))
            };

            if array.is_empty() {
                return Err(create_cf_error(
                    "could not get CMSampleBufferSampleAttachmentsArray",
                    0,
                ));
            }

            let data = unsafe {
                let raw = CFDictionaryRef::from_void_ptr(array.get_unchecked(0).to_owned());
                CFDictionary::<CFString, CFType>::wrap_under_get_rule(raw)
            };

            Ok(Self { data })
        }
        pub(crate) fn internal_status(&self) -> SCFrameStatus {
            unsafe {
                self.data
                    .get(SCStreamFrameInfoStatus)
                    .downcast()
                    .and_then(|n: CFNumber| n.to_i64())
                    .map(|n| mem::transmute::<i64, SCFrameStatus>(n))
                    .expect("could not get status")
            }
        }
        pub(crate) fn internal_display_time(&self) -> u64 {
            unsafe {
                self.data
                    .get(SCStreamFrameInfoDisplayTime)
                    .downcast()
                    .and_then(|n: CFNumber| n.to_i64())
                    .and_then(|n| u64::try_from(n).ok())
                    .expect("could not get display time")
            }
        }
        pub(crate) fn internal_scale_factor(&self) -> f64 {
            unsafe {
                self.data
                    .get(SCStreamFrameInfoScaleFactor)
                    .downcast()
                    .and_then(|n: CFNumber| n.to_f64())
                    .expect("could not get scale factor")
            }
        }
        pub(crate) fn internal_content_scale(&self) -> f64 {
            self.data
                .get(unsafe { SCStreamFrameInfoContentScale })
                .downcast()
                .and_then(|n: CFNumber| n.to_f64())
                .expect("could not get content scale")
        }
        pub(crate) fn internal_bounding_rect(&self) -> CGRect {
            dict_to_cg_rect(
                self.data
                    .get(unsafe { SCStreamFrameInfoBoundingRect })
                    .downcast()
                    .expect("should have bounding rect"),
            )
        }
        pub(crate) fn internal_content_rect(&self) -> CGRect {
            dict_to_cg_rect(
                self.data
                    .get(unsafe { SCStreamFrameInfoContentRect })
                    .downcast()
                    .expect("should have content rect"),
            )
        }
        pub(crate) fn internal_screen_rect(&self) -> CGRect {
            dict_to_cg_rect(
                self.data
                    .get(unsafe { SCStreamFrameInfoScreenRect })
                    .downcast()
                    .expect("should have screen rect"),
            )
        }
        pub(crate) fn internal_dirty_rects(&self) -> Vec<CGRect> {
            unsafe {
                self.data
                    .find(SCStreamFrameInfoDirtyRects)
                    .and_then(|a| a.downcast::<CFArray>())
                    .map(|a| {
                        a.into_iter()
                            .map(|x| get_concrete_from_void(x.to_owned()))
                            .map(dict_to_cg_rect)
                            .collect()
                    })
                    .expect("could not get dirty rects")
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

    #[derive(Debug, Clone)]
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
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
pub use internal::SCFrameStatus;
pub use internal::SCStreamFrameInfo;

impl SCStreamFrameInfo {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn from_buffer(sample_buffer: &CMSampleBuffer) -> Result<Self, CFError> {
        Self::internal_from_buffer(sample_buffer)
    }
    /// Returns the status of this [`SCStreamFrameInfo`].
    pub fn status(&self) -> SCFrameStatus {
        self.internal_status()
    }
}
