use std::{ffi::c_void, ptr};

use crate::{
    stream::{
        sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        sc_stream_delegate_trait::SCStreamDelegateTrait,
        sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
    },
    utils::{
        block::{new_void_completion_handler, CompletionHandler},
        error::create_sc_error,
        objc::get_concrete_from_void,
    },
};
use core_foundation::error::CFError;
use core_foundation::{
    base::{CFTypeID, TCFType},
    declare_TCFType, impl_TCFType,
};
use dispatch::{Queue, QueuePriority};

use objc::{class, msg_send, runtime::Object, sel, sel_impl};

use super::{output_handler, stream_delegate};

#[repr(C)]
pub struct __SCStreamRef(c_void);
extern "C" {
    pub fn SCStreamGetTypeID() -> CFTypeID;
}
pub type SCStreamRef = *mut __SCStreamRef;

declare_TCFType! {SCStream, SCStreamRef}
impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);
impl SCStream {
    pub fn internal_init_with_filter(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Self {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        Self::internal_init_with_filter_and_delegate(filter, configuration, None::<NoopDelegate>)
    }
    pub fn internal_init_with_filter_and_delegate< T: SCStreamDelegateTrait>( 
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: Option<T>,
    ) -> Self {
        unsafe {
            let delegate = delegate.map_or(ptr::null_mut(), stream_delegate::get_handler);
            let inner: *mut Object = msg_send![class!(SCStream), alloc];
            let inner: *mut Object = msg_send![inner, initWithFilter: filter.clone().as_CFTypeRef()  configuration: configuration.clone().as_CFTypeRef() delegate: delegate];

            get_concrete_from_void(inner.cast())
        }
    }

    pub fn internal_remove_output_handler(&mut self, _index: usize, _of_type: SCStreamOutputType) {}

    pub fn internal_add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait,
        of_type: SCStreamOutputType,
    ) -> usize {
        unsafe {
            let error: *mut Object = ptr::null_mut();
            let handler = output_handler::get_handler(handler);
            let stream_queue = Queue::global(QueuePriority::Low);

            match of_type {
                SCStreamOutputType::Screen => {
                    let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Screen sampleHandlerQueue: stream_queue error: error];
                }
                SCStreamOutputType::Audio => {
                    let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Audio sampleHandlerQueue: stream_queue error: error];
                }
            }
        };

        0
    }
    /// Returns the internal start capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn internal_start_capture(&self) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), startCaptureWithCompletionHandler: handler];

            rx.recv()
                .map_err(|_| create_sc_error("Could not receive from completion handler"))?
        }
    }
    /// Returns the internal stop capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn internal_stop_capture(&self) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), stopCaptureWithCompletionHandler: handler];

            rx.recv()
                .map_err(|_| create_sc_error("Could not receive from completion handler"))?
        }
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use core_foundation::error::CFError;

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
            sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
        },
    };

    use super::SCStream;

    struct OutputHandler {
        pub output: String,
    }
    impl SCStreamOutputTrait for OutputHandler {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: core_media_rs::cm_sample_buffer::CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            println!("Output 2: {}", self.output);
            println!("Sample buffer 2: {sample_buffer:?}");
            println!("Sample buffer: {:?}", sample_buffer.get_audio_buffer_list());
            println!("Output type 2: {of_type:?}");
        }
    }
    #[test]
    fn create() -> Result<(), CFError> {
        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        let display = SCShareableContent::get()?.displays().remove(0);
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::internal_init_with_filter(&filter, &config);

        stream.internal_add_output_handler(
            OutputHandler {
                output: "Audio".to_string(),
            },
            SCStreamOutputType::Audio,
        );

        stream.internal_start_capture()?;

        thread::sleep(Duration::from_secs(1));
        stream.internal_stop_capture()
    }
}
