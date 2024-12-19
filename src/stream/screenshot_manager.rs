use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::utils::block::{new_completion_handler, CompletionHandler};
use crate::utils::error::create_sc_error;
use core_foundation::base::TCFType;
use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use objc::{class, msg_send, sel, sel_impl};

pub fn capture(
    filter: &SCContentFilter,
    configuration: &SCStreamConfiguration,
) -> Result<CMSampleBuffer, CFError> {
    unsafe {
        let CompletionHandler(handler, rx) = new_completion_handler();
        let _: () = msg_send![class!(SCScreenshotManager), captureSampleBufferWithFilter: filter.clone().as_CFTypeRef() configuration: configuration.clone().as_CFTypeRef() completionHandler: handler];
        rx.recv()
            .map_err(|_| create_sc_error("Could not receive from completion handler"))?
    }
}

#[cfg(test)]
mod sc_screenshot_manager_test {
    use super::*;
    use crate::shareable_content::SCShareableContent;
    use crate::stream::configuration::SCStreamConfiguration;
    use crate::stream::content_filter::SCContentFilter;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn capture_sample_buffer() {
        let shareable_content = SCShareableContent::get().expect("Failed to get shareable content");
        let filter = SCContentFilter::new()
            .with_display_excluding_windows(&shareable_content.displays()[0], &[]);
        let configuration = SCStreamConfiguration::default();
        let result = capture(&filter, &configuration);
        assert!(result.is_ok(), "Failed to capture display.");
    }
}
