use core::fmt;
use std::sync::Once;

use core_foundation::error::CFError;

use crate::core_media::cm_sample_buffer::CMSampleBuffer;

use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
    sc_stream_delegate::SCStreamDelegateTrait,
};

mod internal {

    #![allow(non_snake_case)]
    #![allow(clippy::needless_pass_by_value)]
    use std::{
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
        runtime::{self, Object, Sel},
        sel, sel_impl,
    };
    use once_cell::sync::Lazy;

    use super::{SCStreamOutputTrait, SCStreamOutputType};

    #[derive(Debug)]
    #[repr(C)]
    pub struct SCStream<T: SCStreamOutputTrait> {
        obj: *mut Object,
        pub(crate) output_handlers: Vec<T>,
    }

    impl<T: SCStreamOutputTrait> SCStream<T> {
        fn test(&self) {
            println!("TEST! {self:?}");
        }
    }

    static QUEUE: Lazy<Queue> =
        Lazy::new(|| Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent));

    unsafe impl objc::Encode for SCStreamOutputType {
        fn encode() -> objc::Encoding {
            i8::encode()
        }
    }
    fn register_objc_class<T: SCStreamOutputTrait>() {
        type StreamOutputMethod =
            extern "C" fn(&Object, Sel, *const c_void, *const c_void, SCStreamOutputType);
        extern "C" fn stream_output<T: SCStreamOutputTrait>(
            this: &Object,
            _cmd: Sel,
            _stream_ref: *const c_void,
            _sample_buffer_ref: *const c_void,
            _of_type: SCStreamOutputType,
        ) {
            unsafe {
                let s: &SCStream<T> = ptr::read(this.get_ivar::<*mut c_void>("_stream").cast());
                s.test();
            };
        }
        let mut decl =
            ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");

        unsafe {
            let output_handler: StreamOutputMethod = stream_output::<T>;

            decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
            decl.add_ivar::<*const c_void>("_stream");
            decl.register();
        }
    }
    pub fn add_output_handler<T: SCStreamOutputTrait>(
        stream: &mut SCStream<T>,
        of_type: SCStreamOutputType,
    ) {
        register_objc_class::<T>();
        unsafe {
            let output = runtime::class_createInstance(class!(StreamOutput), 0);
            (*output).set_ivar("_stream", ptr::addr_of!(stream).cast::<c_void>());
            let _: () = msg_send![*stream, addStreamOutput: output type: of_type sampleHandlerQueue: QUEUE.clone() error: ptr::null_mut::<CFErrorRef>()];
        }
    }

    impl<T: SCStreamOutputTrait> Deref for SCStream<T> {
        type Target = Object;

        fn deref(&self) -> &Object {
            unsafe { &*self.obj }
        }
    }

    impl<T: SCStreamOutputTrait> DerefMut for SCStream<T> {
        fn deref_mut(&mut self) -> &mut Object {
            unsafe { &mut *self.obj }
        }
    }

    impl<T: SCStreamOutputTrait> Drop for SCStream<T> {
        fn drop(&mut self) {
            unsafe {
                runtime::object_dispose(self.obj);
            }
        }
    }

    fn new<T: SCStreamOutputTrait>(obj: *mut Object) -> SCStream<T> {
        SCStream {
            obj,
            output_handlers: Vec::new(),
        }
    }
    pub fn init_with_filter_and_delegate<T: SCStreamOutputTrait>(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> SCStream<T> {
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];
            new(instance)
        }
    }
    pub fn init_with_filter<T: SCStreamOutputTrait>(
        filter: SCContentFilter,
        configuration: SCStreamConfiguration,
    ) -> SCStream<T> {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(NoopDelegate)];
            new(instance)
        }
    }
    pub fn start_capture<T: SCStreamOutputTrait>(stream: &SCStream<T>) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![*stream, startCaptureWithCompletionHandler: handler];

            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
    pub fn stop_capture<T: SCStreamOutputTrait>(stream: &SCStream<T>) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![*stream, stopCaptureWithCompletionHandler: handler];

            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
}
pub use internal::SCStream;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(C)]
pub enum SCStreamOutputType {
    Screen,
    Audio,
}
pub trait SCStreamOutputTrait: Sized + fmt::Debug {
    fn did_output_sample_buffer(
        &self,
        stream: SCStream<Self>,
        sample_buffer: CMSampleBuffer,
        of_type: SCStreamOutputType,
    );
}
impl<T: SCStreamOutputTrait> SCStream<T> {
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
    pub fn add_output_handler(&mut self, output_trait: T, of_type: SCStreamOutputType) {
        static ONLY_ONCE_AUDIO: Once = Once::new();
        static ONLY_ONCE_SCREEN: Once = Once::new();
        match of_type {
            SCStreamOutputType::Screen => ONLY_ONCE_SCREEN.call_once(|| {
                internal::add_output_handler(self, SCStreamOutputType::Screen);
            }),
            SCStreamOutputType::Audio => ONLY_ONCE_AUDIO.call_once(|| {
                internal::add_output_handler(self, SCStreamOutputType::Audio);
            }),
        }
        self.output_handlers.push(output_trait);
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
        core_media::cm_sample_buffer::CMSampleBuffer,
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::{SCStream, SCStreamOutputTrait, SCStreamOutputType};
    #[derive(Debug)]
    struct TestStreamOutput;

    impl SCStreamOutputTrait for TestStreamOutput {
        fn did_output_sample_buffer(
            &self,
            _stream: SCStream<Self>,
            _sample_buffer: CMSampleBuffer,
            _of_type: SCStreamOutputType,
        ) {
        }
    }
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
            let mut s = SCStream::new(&filter, &config);
            s.add_output_handler(TestStreamOutput, SCStreamOutputType::Screen);
            s
        };
        stream.start_capture().expect("Could not start capture");
        std::thread::sleep(Duration::from_secs(1));
        Ok(())
    }
}
