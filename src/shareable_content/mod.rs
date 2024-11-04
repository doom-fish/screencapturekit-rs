pub mod display;
pub mod running_application;
pub mod window;
pub use display::SCDisplay;
pub use running_application::SCRunningApplication;
pub use window::SCWindow;

use core::fmt;

use core_foundation::{base::TCFType, error::CFError};
use objc::{class, msg_send, sel, sel_impl};

mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };

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

#[allow(clippy::module_name_repetitions)]
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
    #[must_use]
    pub const fn exclude_desktop(mut self) -> Self {
        self.exclude_desktop = true;
        self
    }
    #[must_use]
    pub fn on_screen_windows_only(mut self) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreen;
        self
    }
    #[must_use]
    pub fn on_screen_windows_only_above(mut self, window: SCWindow) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreenAbove(window);
        self
    }
    #[must_use]
    pub fn on_screen_windows_only_below(mut self, window: SCWindow) -> Self {
        self.capture_option = CaptureOption::OnlyOnScreenBelow(window);
        self
    }
    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn get(self) -> Result<SCShareableContent, CFError> {
        let CompletionHandler(handler, rx) = new_completion_handler();

        unsafe {
            let _: () = match self.capture_option {
                CaptureOption::Default => msg_send![
                    class!(SCShareableContent),
                    getShareableContentWithCompletionHandler: handler
                ],
                CaptureOption::OnlyOnScreen => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: u8::from(self.exclude_desktop)
                    onScreenWindowsOnly: 1
                    completionHandler: handler
                ],
                CaptureOption::OnlyOnScreenAbove(w) => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: u8::from(self.exclude_desktop)
                    onScreenWindowsOnlyAboveWindow: w.as_CFTypeRef()
                    completionHandler: handler
                ],
                CaptureOption::OnlyOnScreenBelow(w) => msg_send![
                    class!(SCShareableContent),
                    getShareableContentExcludingDesktopWindows: u8::from(self.exclude_desktop)
                    onScreenWindowsOnlyBelowWindow: w.as_CFTypeRef()
                    completionHandler: handler
                ],
            };
        };

        rx.recv()
            .expect("Should receive a return from completion handler")
    }
}

use crate::utils::{
    block::{new_completion_handler, CompletionHandler},
    objc::get_vec_property,
};

impl SCShareableContent {
    pub fn with_options() -> SCShareableContentOptions {
        SCShareableContentOptions::default()
    }
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn get() -> Result<Self, CFError> {
        Self::with_options().get()
    }

    pub fn displays(&self) -> Vec<SCDisplay> {
        get_vec_property(self, sel!(displays))
    }

    pub fn applications(&self) -> Vec<SCRunningApplication> {
        get_vec_property(self, sel!(applications))
    }
    pub fn windows(&self) -> Vec<SCWindow> {
        get_vec_property(self, sel!(windows))
    }
}
impl fmt::Debug for SCShareableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCShareableContent")
            .field("displays", &self.displays().len())
            .field("applications", &self.applications().len())
            .field("windows", &self.windows().len())
            .finish()
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
    fn debug_format() {
        println!("{:?}", SCShareableContent::get());
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
