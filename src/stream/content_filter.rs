mod internal {
    #![allow(non_snake_case)]

    use std::ffi::c_void;

    use crate::{
        shareable_content::{SCDisplay, SCRunningApplication, SCWindow},
        utils::objc::MessageForTFType,
    };
    use core_foundation::{
        array::CFArray,
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };
    use objc::{class, msg_send, sel, sel_impl};
    #[repr(C)]
    pub struct __SCContentFilterRef(c_void);
    extern "C" {
        pub fn SCContentFilterGetTypeID() -> CFTypeID;
    }
    pub type SCContentFilterRef = *mut __SCContentFilterRef;

    declare_TCFType! {SCContentFilter, SCContentFilterRef}
    impl_TCFType!(
        SCContentFilter,
        SCContentFilterRef,
        SCContentFilterGetTypeID
    );
    fn clone_elements<T: Clone>(elements: &[&T]) -> Vec<T> {
        elements.iter().map(|e| e.to_owned().clone()).collect()
    }

    pub fn init_with_desktop_independent_window(filter: &SCContentFilter, window: &SCWindow) {
        unsafe {
            let _: () = msg_send![filter.as_sendable(), initWithDesktopIndependentWindow: window.as_CFTypeRef()];
        }
    }
    pub fn init_with_display_including_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        including_windows: &[&SCWindow],
    ) {
        unsafe {
            let cfarr = CFArray::from_CFTypes(clone_elements(including_windows).as_slice());
            let _: () = msg_send![filter.as_sendable(), initWithDisplay: display.as_CFTypeRef() includingWindows: cfarr.as_CFTypeRef() ];
        }
    }
    pub fn init_with_display_excluding_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        excluding_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excluding_windows).as_slice());
            let _: () = msg_send![filter.as_sendable(), initWithDisplay: display.clone().as_CFTypeRef() excludingWindows: windows.as_CFTypeRef()];
        }
    }
    pub fn init_with_display_including_applications_excepting_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        including_applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excepting_windows).as_slice());
            let applications =
                CFArray::from_CFTypes(clone_elements(including_applications).as_slice());
            let _: () = msg_send![filter.as_sendable(), initWithDisplay: display.as_CFTypeRef() includingApplications: applications.as_CFTypeRef() exceptingWindows: windows.as_CFTypeRef()];
        }
    }
    pub fn init_with_display_excluding_applications_excepting_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        excluding_applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excepting_windows).as_slice());
            let applications =
                CFArray::from_CFTypes(clone_elements(excluding_applications).as_slice());
            let _: () = msg_send![filter.as_sendable(), initWithDisplay: display.as_CFTypeRef() excludingApplications: applications.as_CFTypeRef() exceptingWindows: windows.as_CFTypeRef()];
        }
    }
    pub fn create() -> SCContentFilter {
        unsafe {
            let ptr: SCContentFilterRef = msg_send![class!(SCContentFilter), alloc];
            SCContentFilter::wrap_under_create_rule(ptr)
        }
    }
}
pub use internal::SCContentFilterRef;
#[allow(clippy::module_name_repetitions)]
pub use internal::SCContentFilter;

use crate::shareable_content::{SCDisplay, SCRunningApplication, SCWindow};

use self::internal::{
    create, init_with_desktop_independent_window,
    init_with_display_excluding_applications_excepting_windows,
    init_with_display_excluding_windows,
    init_with_display_including_applications_excepting_windows,
    init_with_display_including_windows,
};

impl SCContentFilter {
    pub fn new() -> Self {
        create()
    }

    #[must_use]
    pub fn with_desktop_independent_window(self, window: &SCWindow) -> Self {
        init_with_desktop_independent_window(&self, window);
        self
    }
    #[must_use]
    pub fn with_display_excluding_windows(
        self,
        display: &SCDisplay,
        excluding_windows: &[&SCWindow],
    ) -> Self {
        init_with_display_excluding_windows(&self, display, excluding_windows);
        self
    }
    #[must_use]
    pub fn with_display_including_windows(
        self,
        display: &SCDisplay,
        including_windows: &[&SCWindow],
    ) -> Self {
        init_with_display_including_windows(&self, display, including_windows);
        self
    }
    #[must_use]
    pub fn with_display_including_application_excepting_windows(
        self,
        display: &SCDisplay,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        init_with_display_including_applications_excepting_windows(
            &self,
            display,
            applications,
            excepting_windows,
        );
        self
    }
    #[must_use]
    pub fn with_display_excluding_applications_excepting_windows(
        self,
        display: &SCDisplay,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        init_with_display_excluding_applications_excepting_windows(
            &self,
            display,
            applications,
            excepting_windows,
        );
        self
    }
}

impl Default for SCContentFilter {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod test_content_filter {
    use crate::{shareable_content::SCShareableContent, stream::content_filter::SCContentFilter};

    #[test]
    fn test_init_with_display() {
        let displays = SCShareableContent::get().expect("Should work").displays();
        let display = displays.first().unwrap();
        let _ = SCContentFilter::new().with_display_excluding_windows(display, &[]);
    }
}
