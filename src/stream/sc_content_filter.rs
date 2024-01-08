use objc::{runtime::Class, *};
use objc_foundation::{INSArray, INSObject, NSArray};

use super::shareable_content::{UnsafeSCDisplay, UnsafeSCRunningApplication, UnsafeSCWindow};

#[derive(Debug)]
pub struct UnsafeContentFilter {
    __priv: u8,
}

pub enum UnsafeInitParams {
    DesktopIndependentWindow(UnsafeSCWindow),
    Display(UnsafeSCDisplay),
    DisplayIncludingWindows(UnsafeSCDisplay, Vec<UnsafeSCWindow>),
    DisplayExcludingWindows(UnsafeSCDisplay, Vec<UnsafeSCWindow>),
    DisplayIncludingApplicationsExceptingWindows(
        UnsafeSCDisplay,
        Vec<UnsafeSCRunningApplication>,
        Vec<UnsafeSCWindow>,
    ),
    DisplayExcludingApplicationsExceptingWindows(
        UnsafeSCDisplay,
        Vec<UnsafeSCRunningApplication>,
        Vec<UnsafeSCWindow>,
    ),
}

impl UnsafeContentFilter {
    pub fn init(params: UnsafeInitParams) -> Self {
        let content_filter = UnsafeContentFilter::new();
        unsafe {
            match params {
                UnsafeInitParams::Display(display) => {
                    let _: () = msg_send![content_filter, initWithDisplay: display excludingWindows: NSArray::from_slice(&[] as &[UnsafeSCWindow, Shared>])];
                }
                UnsafeInitParams::DesktopIndependentWindow(window) => {
                    let _: () = msg_send![content_filter, initWithDesktopIndependentWindow: window];
                }
                UnsafeInitParams::DisplayIncludingWindows(display, windows) => {
                    let _: () = msg_send![content_filter, initWithDisplay : display includingWindows: NSArray::from_vec(windows)];
                }
                UnsafeInitParams::DisplayExcludingWindows(display, windows) => {
                    let _: () = msg_send![content_filter, initWithDisplay : display excludingWindows: NSArray::from_vec(windows)];
                }
                UnsafeInitParams::DisplayIncludingApplicationsExceptingWindows(
                    display,
                    applications,
                    windows,
                ) => {
                    let _: () = msg_send![content_filter, initWithDisplay : display excludingApplications : NSArray::from_vec(applications) exceptingWindows:  NSArray::from_vec(windows)];
                }
                UnsafeInitParams::DisplayExcludingApplicationsExceptingWindows(
                    display,
                    applications,
                    windows,
                ) => {
                    let _: () = msg_send![content_filter, initWithDisplay : display includingApplications : NSArray::from_vec(applications) exceptingWindows: NSArray::from_vec(windows)];
                }
            }
        };
        content_filter
    }
}
#[cfg(test)]
mod test_content_filter {

    use crate::shareable_content::UnsafeSCShareableContent;

    use super::*;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_init() {
        let sc = UnsafeSCShareableContent::get().expect("should get shareable content");
        let applications = sc.applications();
        let windows = sc.windows();
        let display = sc.displays().pop().unwrap();

        UnsafeContentFilter::init(UnsafeInitParams::DisplayIncludingWindows(
            display.clone(),
            windows.clone(),
        ));
        UnsafeContentFilter::init(UnsafeInitParams::DisplayExcludingWindows(
            display.clone(),
            windows.clone(),
        ));
        UnsafeContentFilter::init(UnsafeInitParams::DesktopIndependentWindow(
            windows[0].clone(),
        ));
        UnsafeContentFilter::init(
            UnsafeInitParams::DisplayIncludingApplicationsExceptingWindows(
                display.clone(),
                applications.clone(),
                windows.clone(),
            ),
        );
        UnsafeContentFilter::init(
            UnsafeInitParams::DisplayIncludingApplicationsExceptingWindows(
                display.clone(),
                applications.clone(),
                windows.clone(),
            ),
        );

        drop(sc);
        drop(applications);
        drop(windows);
        drop(display);
    }
}
