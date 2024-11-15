use std::{ffi::c_void, ptr};

use crate::{
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait, internal::delegate,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType,
    },
    utils::{
        block::{new_void_completion_handler, CompletionHandler},
        error::create_sc_error,
    },
};
use core_foundation::{base, error::CFError};
use core_foundation::{
    base::{CFTypeID, TCFType},
    impl_TCFType,
};
use dispatch::{Queue, QueuePriority};

use objc::{class, declare::ClassDecl, msg_send, runtime::Object, sel, sel_impl};

use super::{
    cleanup::Cleanup,
    output_handler::{self, SCStreamOutput},
};

#[repr(C)]
pub struct __SCStreamRef(c_void);
extern "C" {
    pub fn SCStreamGetTypeID() -> CFTypeID;
}
pub type SCStreamRef = *mut __SCStreamRef;

#[allow(clippy::module_name_repetitions)]
pub struct SCStream(SCStreamRef);

impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);

impl Drop for SCStream {
    fn drop(&mut self) {
        unsafe {
            (*self.as_concrete_TypeRef().cast::<Object>())
                .get_mut_ivar::<Cleanup>("cleanup")
                .drop_handlers();

            base::CFRelease(self.as_CFTypeRef());
        }
    }
}
fn register() {
    let mut decl =
        ClassDecl::new("SCStreamWithHandlers", class!(SCStream)).expect("Could not register class");
    decl.add_ivar::<Cleanup>("cleanup");
    decl.register();
}

impl SCStream {
    pub fn store_cleanup(&self, handler: *mut Object) {
        unsafe {
            let obj = self.as_concrete_TypeRef().cast::<Object>();
            (*obj)
                .get_mut_ivar::<Cleanup>("cleanup")
                .add_handler(handler);
        };
    }
    pub fn internal_init_with_filter(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Self {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        Self::internal_init_with_filter_and_delegate(filter, configuration, None::<NoopDelegate>)
    }
    pub fn internal_init_with_filter_and_delegate<T: SCStreamDelegateTrait>(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: Option<T>,
    ) -> Self {
        static REGISTER_ONCE: std::sync::Once = std::sync::Once::new();
        REGISTER_ONCE.call_once(register);
        unsafe {
            let delegate = delegate.map_or(ptr::null_mut(), delegate::get_handler);
            let inner: *mut Object = msg_send![class!(SCStreamWithHandlers), alloc];
            (*inner).set_ivar("cleanup", Cleanup::new(delegate));
            let inner: SCStreamRef = msg_send![inner, initWithFilter: filter.clone().as_CFTypeRef()  configuration: configuration.clone().as_CFTypeRef() delegate: delegate];
            Self::wrap_under_create_rule(inner)
        }
    }

    pub fn internal_remove_output_handler(
        &mut self,
        handler: SCStreamOutput,
        of_type: SCStreamOutputType,
    ) -> bool {
        let error: *mut Object = ptr::null_mut();
        unsafe {
            msg_send![self.as_CFTypeRef().cast::<Object>(), removeStreamOutput: handler type: of_type error: error]
        }
    }

    pub fn internal_add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait,
        of_type: SCStreamOutputType,
    ) -> Option<SCStreamOutput> {
        unsafe {
            let error: *mut Object = ptr::null_mut();
            let handler = output_handler::get_handler(handler);
            let stream_queue = Queue::global(QueuePriority::Low);

            let success: bool = match of_type {
                SCStreamOutputType::Screen => {
                    msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Screen sampleHandlerQueue: stream_queue error: error]
                }
                SCStreamOutputType::Audio => {
                    msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Audio sampleHandlerQueue: stream_queue error: error]
                }
            };

            self.store_cleanup(handler);

            if success {
                Some(handler)
            } else {
                None
            }
        }
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

    use std::sync::mpsc::channel;

    use core_foundation::error::CFError;
    use core_media_rs::cm_sample_buffer::CMSampleBuffer;

    use crate::{
        output::sc_stream_frame_info::SCStreamFrameInfo,
        shareable_content::SCShareableContent,
        stream::{
            configuration::SCStreamConfiguration, content_filter::SCContentFilter,
            delegate_trait::SCStreamDelegateTrait, output_trait::SCStreamOutputTrait,
            output_type::SCStreamOutputType,
        },
    };

    use super::SCStream;

    struct AudioOutputHandler<'a> {
        pub output: &'a str,
    }
    struct ScreenOutputHandler {
        sender: std::sync::mpsc::Sender<CMSampleBuffer>,
    }
    impl SCStreamOutputTrait for AudioOutputHandler<'_> {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: core_media_rs::cm_sample_buffer::CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            println!("Output 2: {}", self.output);
            println!("Sample buffer 2: {sample_buffer:?}");
            println!("Output type 2: {of_type:?}");
        }
    }
    impl SCStreamDelegateTrait for AudioOutputHandler<'_> {}
    impl SCStreamOutputTrait for ScreenOutputHandler {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: CMSampleBuffer,
            _of_type: SCStreamOutputType,
        ) {
            self.sender.send(sample_buffer).unwrap();
        }
    }

    #[test]
    fn test_audio() -> Result<(), CFError> {
        let output = "Audio";
        let mut stream = create_stream(output)?;
        stream
            .internal_add_output_handler(AudioOutputHandler { output }, SCStreamOutputType::Audio);
        stream.internal_start_capture()?;
        stream.internal_stop_capture()
    }

    #[test]
    fn test_video() -> Result<(), CFError> {
        let output = "Video";
        let mut stream = create_stream(output)?;
        let (tx, rx) = channel();
        stream.internal_add_output_handler(
            ScreenOutputHandler { sender: tx },
            SCStreamOutputType::Screen,
        );
        stream.internal_start_capture()?;
        let sample = rx.recv().unwrap();
        let frame_info = SCStreamFrameInfo::from_buffer(&sample)?;

        println!("{frame_info:?}");

        stream.internal_stop_capture()
    }

    fn create_stream(output: &str) -> Result<SCStream, CFError> {
        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        let display = SCShareableContent::get()?.displays().remove(0);
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
        let stream = SCStream::internal_init_with_filter_and_delegate(
            &filter,
            &config,
            Some(AudioOutputHandler { output }),
        );
        Ok(stream)
    }
}
