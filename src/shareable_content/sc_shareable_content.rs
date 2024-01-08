use core_foundation::{array::CFArray, base::TCFType, base::*, error::CFError};
use objc::{msg_send, *};

mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};
    #[repr(C)]
    pub struct __SCShareableContentRef(c_void);
    extern "C" {
        pub fn SCShareableContentGetTypeID() -> CFTypeID;
    }
    pub type SCShareableContentRef = *mut __SCShareableContentRef;

    declare_TCFType! {SCShareableContent, SCShareableContentRef}
    impl_TCFType!(
        SCShareableContent,
        SCShareableContentRef,
        SCShareableContentGetTypeID
    );
}
pub use internal::SCShareableContent;

#[derive(Default)]
enum CaptureOption {
    #[default]
    Default,
    OnlyOnScreen,
    OnlyOnScreenAbove(SCWindow),
    OnlyOnScreenBelow(SCWindow),
}

#[derive(Default)]
pub struct SCShareableContentOptions {
    capture_option: CaptureOption,
    exclude_desktop: bool,
}

impl SCShareableContentOptions {
    pub fn exclude_desktop(mut self) -> Self {
        self.exclude_desktop = true;
        self
    }
    pub fn on_screen_windows_only(mut self) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreen;
        self
    }
    pub fn on_screen_windows_only_above(mut self, window: SCWindow) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreenAbove(window);
        self
    }
    pub fn on_screen_windows_only_below(mut self, window: SCWindow) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreenBelow(window);
        self
    }
    pub fn get(self) -> Result<SCShareableContent, CFError> {
        let CompletionHandler(completion_handler, rx) = new_completion_handler();

        unsafe {
            let _: () = match self.capture_option {
                CaptureOption::Default => msg_send![
                    class!(SCShareableContent),
                    getShareableContentWithCompletionHandler: completion_handler
                ],
                CaptureOption::OnlyOnScreen => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: self.exclude_desktop as u8
                    onScreenWindowsOnly: 1
                    completionHandler: completion_handler
                ],
                CaptureOption::OnlyOnScreenAbove(w) => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: self.exclude_desktop as u8
                    onScreenWindowsOnlyAboveWindow: w.as_CFTypeRef()
                    completionHandler: completion_handler
                ],
                CaptureOption::OnlyOnScreenBelow(w) => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: self.exclude_desktop as u8
                    onScreenWindowsOnlyBelowWindow: w.as_CFTypeRef()
                    completionHandler: completion_handler
                ],
            };
        };

        rx.recv().expect("should work")
    }
}

use crate::utils::{
    block::{new_completion_handler, CompletionHandler},
    objc::SendableObjc,
};

use super::{
    sc_display::{SCDisplay, SCDisplayRef},
    sc_running_application::{SCRunningApplication, SCRunningApplicationRef},
    sc_window::{SCWindow, SCWindowRef},
};

impl SCShareableContent {
    pub fn with_options() -> SCShareableContentOptions {
        SCShareableContentOptions::default()
    }
    pub fn get() -> Result<Self, CFError> {
        Self::with_options().get()
    }

    pub fn displays(&self) -> Vec<SCDisplay> {
        unsafe {
            CFArray::<SCDisplayRef>::wrap_under_get_rule(msg_send![self.to_sendable(), displays])
                .into_untyped()
                .iter()
                .map(|ptr| SCDisplay::wrap_under_get_rule(SCDisplayRef::from_void_ptr(*ptr)))
                .collect()
        }
    }
    pub fn applications(&self) -> Vec<SCRunningApplication> {
        unsafe {
            CFArray::<SCRunningApplicationRef>::wrap_under_get_rule(msg_send![
                self.to_sendable(),
                applications
            ])
            .into_untyped()
            .iter()
            .map(|ptr| {
                SCRunningApplication::wrap_under_get_rule(SCRunningApplicationRef::from_void_ptr(
                    *ptr,
                ))
            })
            .collect()
        }
    }
    pub fn windows(&self) -> Vec<SCWindow> {
        unsafe {
            CFArray::<SCWindowRef>::wrap_under_get_rule(msg_send![self.to_sendable(), windows])
                .into_untyped()
                .iter()
                .map(|ptr| SCWindow::wrap_under_get_rule(SCWindowRef::from_void_ptr(*ptr)))
                .collect()
        }
    }
}

#[cfg(test)]
mod sc_shareable_content_test {
    use super::SCShareableContent;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn get_default() {
        let content = SCShareableContent::get().expect("Should work");
        assert!(!content.displays().is_empty());
        assert!(!content.windows().is_empty());
        assert!(!content.applications().is_empty());
    }
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn get_on_screen() {
        SCShareableContent::with_options()
            .exclude_desktop()
            .on_screen_windows_only()
            .get()
            .expect("should work");
        SCShareableContent::with_options()
            .on_screen_windows_only()
            .get()
            .expect("should work");
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn get_on_screen_above() {
        let windows = SCShareableContent::get().expect("should work").windows();
        SCShareableContent::with_options()
            .exclude_desktop()
            .on_screen_windows_only_above(windows.first().unwrap().clone())
            .get()
            .expect("should work");
        SCShareableContent::with_options()
            .on_screen_windows_only_above(windows.first().unwrap().clone())
            .get()
            .expect("should work");
    }
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn get_on_screen_below() {
        let windows = SCShareableContent::get().expect("should work").windows();
        SCShareableContent::with_options()
            .exclude_desktop()
            .on_screen_windows_only_below(windows.first().unwrap().clone())
            .get()
            .expect("should work");
        SCShareableContent::with_options()
            .on_screen_windows_only_below(windows.first().unwrap().clone())
            .get()
            .expect("should work");
    }
}
