use core_foundation::error::CFError;

use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
    sc_stream_delegate::SCStreamDelegateTrait,
};

mod internal {

    #![allow(non_snake_case)]
    #![allow(clippy::needless_pass_by_value)]
    use std::{
        error::Error,
        ffi::c_void,
        ops::{Deref, DerefMut},
        ptr,
        sync::Once,
    };

    use crate::{
        stream::{
            sc_content_filter::SCContentFilter,
            sc_stream_configuration::SCStreamConfiguration,
            sc_stream_delegate::{SCStreamDelegate, SCStreamDelegateTrait},
        },
        utils::block::{new_void_completion_handler, VoidCompletionHandler},
    };
    use core_foundation::{
        base::TCFType,
        error::{CFError, CFErrorRef},
    };
    use dispatch::{Queue, QueueAttribute};
    use objc::{
        class,
        declare::ClassDecl,
        msg_send,
        runtime::{self, Class, Object, Sel},
        sel, sel_impl,
    };
    use once_cell::sync::Lazy;
    pub struct SCStream {
        obj: *mut Object,
        // output_handler: Vec<Box<dyn SCStreamOutputTrait>>,
    }
    impl SCStream {
        fn test(&self) {
            println!("TEST!");
        }
    }

    static QUEUE: Lazy<Queue> =
        Lazy::new(|| Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent));

    #[derive(Eq, PartialEq, Debug, Clone, Copy)]
    #[repr(C)]
    pub enum SCStreamOutputType {
        Screen,
        Audio,
    }

    unsafe impl objc::Encode for SCStreamOutputType {
        fn encode() -> objc::Encoding {
            i8::encode()
        }
    }
    fn register_objc_class() {
        type StreamOutputMethod =
            extern "C" fn(&Object, Sel, *const c_void, *const c_void, SCStreamOutputType);
        extern "C" fn stream_output(
            this: &Object,
            _cmd: Sel,
            stream_ref: *const c_void,
            sample_buffer_ref: *const c_void,
            of_type: SCStreamOutputType,
        ) {
            unsafe {
                let s: &SCStream = &*(this.get_ivar::<*mut c_void>("_stream").cast());
                s.test();
            };
        }
        let mut decl =
            ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");

        unsafe {
            let output_handler: StreamOutputMethod = stream_output;
            decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
            decl.add_ivar::<*const c_void>("_stream");
            decl.register();
        }
    }
    fn add_output_handler(stream: &mut SCStream) {
        static REGISTER_UNSAFE_SC_OUTPUT_HANDLER: Once = Once::new();
        REGISTER_UNSAFE_SC_OUTPUT_HANDLER.call_once(register_objc_class);
        let output = unsafe { runtime::class_createInstance(class!(StreamOutput), 0) };
        unsafe {
            (*output).set_ivar(
                "_stream",
                ptr::addr_of!(stream) as *const _ as *const c_void,
            );
            let _: () = msg_send![*stream, addStreamOutput: output type: SCStreamOutputType::Screen sampleHandlerQueue: QUEUE.clone() error: ptr::null_mut::<CFErrorRef>()];

            let _: () = msg_send![*stream, addStreamOutput: output type: SCStreamOutputType::Audio sampleHandlerQueue: QUEUE.clone() error: ptr::null_mut::<CFErrorRef>()];
        }
    }

    impl From<*mut Object> for SCStream {
        fn from(obj: *mut Object) -> Self {
            let mut instance = Self {
                obj,
                // output_handler: vec![],
            };
            add_output_handler(&mut instance);
            instance
        }
    }

    impl Deref for SCStream {
        type Target = Object;

        fn deref(&self) -> &Object {
            unsafe { &*self.obj }
        }
    }

    impl DerefMut for SCStream {
        fn deref_mut(&mut self) -> &mut Object {
            unsafe { &mut *self.obj }
        }
    }

    impl Drop for SCStream {
        fn drop(&mut self) {
            unsafe {
                runtime::object_dispose(self.obj);
            }
        }
    }

    pub fn init_with_filter_and_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> SCStream {
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];
            instance.into()
        }
    }
    pub fn init_with_filter(
        filter: SCContentFilter,
        configuration: SCStreamConfiguration,
    ) -> SCStream {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(NoopDelegate)];
            instance.into()
        }
    }
    pub fn start_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![*stream, startCaptureWithCompletionHandler: handler];

            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
    pub fn stop_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![*stream, stopCaptureWithCompletionHandler: handler];

            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
}
pub use internal::SCStream;

impl SCStream {
    /// .
    pub fn new_with_error_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> Self {
        internal::init_with_filter_and_delegate(filter, configuration, stream_delegate)
    }

    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        internal::init_with_filter(filter.clone(), configuration.clone())
    }

    /// Returns the start capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn start_capture(&self) -> Result<(), CFError> {
        internal::start_capture(self)
    }
    /// Returns the stop capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn stop_capture(&self) -> Result<(), CFError> {
        internal::stop_capture(self)
    }
}

#[cfg(test)]
mod stream_test {
    use std::{error::Error, time::Duration};

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::SCStream;

    #[test]
    fn test_sc_stream() -> Result<(), Box<dyn Error>> {
        let stream = {
            let config = SCStreamConfiguration::new()
                .set_width(100)?
                .set_height(100)?
                .set_captures_audio(true)?;

            let display = SCShareableContent::get().unwrap().displays().remove(1);
            let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
            //
            // let a = TestStreamOutput { tx: tx_audio };
            SCStream::new(&filter, &config)
        };
        stream.start_capture().expect("Could not start capture");
        std::thread::sleep(Duration::from_secs(1));
        Ok(())
    }
}
