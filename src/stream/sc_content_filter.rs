mod internal {
    #![allow(non_snake_case)]

    use std::ffi::c_void;

    use crate::shareable_content::{
        sc_display::SCDisplay, sc_running_application::SCRunningApplication, sc_window::SCWindow,
    };
    use crate::utils::objc::SendableObjc;
    use core_foundation::{array::CFArray, base::*, *};
    use objc::*;
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

    pub(crate) fn init_with_desktop_independent_window(filter: &SCContentFilter, window: SCWindow) {
        unsafe {
            msg_send![filter.to_sendable(), initWithDesktopIndependentWindow: window.as_CFTypeRef()]
        }
    }
    pub(crate) fn init_with_display_including_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        including_windows: &[&SCWindow],
    ) {
        unsafe {
            let cfarr = CFArray::from_CFTypes(clone_elements(including_windows).as_slice());
            msg_send![filter.to_sendable(), initWithDisplay: display.as_CFTypeRef() includingWindows: cfarr.as_CFTypeRef() ]
        }
    }
    pub(crate) fn init_with_display_excluding_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        excluding_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excluding_windows).as_slice());
            msg_send![filter.to_sendable(), initWithDisplay: display.as_CFTypeRef() includingWindows: windows.as_CFTypeRef()]
        }
    }
    pub(crate) fn init_with_display_including_applications_excepting_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        including_applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excepting_windows).as_slice());
            let applications =
                CFArray::from_CFTypes(clone_elements(including_applications).as_slice());
            msg_send![filter.to_sendable(), initWithDisplay: display.as_CFTypeRef() includingApplications: applications.as_CFTypeRef() exceptingWindows: windows.as_CFTypeRef()]
        }
    }
    pub(crate) fn init_with_display_excluding_applications_excepting_windows(
        filter: &SCContentFilter,
        display: &SCDisplay,
        excluding_applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        unsafe {
            let windows = CFArray::from_CFTypes(clone_elements(excepting_windows).as_slice());
            let applications =
                CFArray::from_CFTypes(clone_elements(excluding_applications).as_slice());
            msg_send![filter.to_sendable(), initWithDisplay: display.as_CFTypeRef() excludingApplications: applications.as_CFTypeRef() exceptingWindows: windows.as_CFTypeRef()]
        }
    }
    pub(crate) fn create() -> SCContentFilter {
        unsafe {
            let ptr: SCContentFilterRef = msg_send![class!(SCContentFilter), alloc];
            // let ptr: SCContentFilterRef = msg_send![ptr.to_sendable(), init];
            SCContentFilter::wrap_under_create_rule(ptr)
        }
    }
}
pub use internal::{SCContentFilter, SCContentFilterRef};

use crate::shareable_content::{
    sc_display::SCDisplay, sc_running_application::SCRunningApplication, sc_window::SCWindow,
};

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

    pub fn with_desktop_independent_window(self, window: SCWindow) -> Self {
        init_with_desktop_independent_window(&self, window);
        self
    }
    pub fn init_with_display_excluding_windows(
        &self,
        display: &SCDisplay,
        excluding_windows: &[&SCWindow],
    ) {
        init_with_display_excluding_windows(self, display, excluding_windows)
    }
    pub fn with_display_including_windows(
        &self,
        display: &SCDisplay,
        including_windows: &[&SCWindow],
    ) {
        init_with_display_including_windows(self, display, including_windows)
    }
    pub fn with_display_including_application_excepting_windows(
        &self,
        display: &SCDisplay,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        init_with_display_including_applications_excepting_windows(
            self,
            display,
            applications,
            excepting_windows,
        )
    }
    pub fn with_display_excluding_applications_excepting_windows(
        &self,
        display: &SCDisplay,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) {
        init_with_display_excluding_applications_excepting_windows(
            self,
            display,
            applications,
            excepting_windows,
        )
    }
}

impl Default for SCContentFilter {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod test_content_filter {
    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::sc_content_filter::SCContentFilter,
    };

    #[test]
    fn test_init_with_display() {
        let displays = SCShareableContent::get().expect("Should work").displays();
        let display = displays.first().unwrap();
        SCContentFilter::new().init_with_display_excluding_windows(display, &[])
    }
}
