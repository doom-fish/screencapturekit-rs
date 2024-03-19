use core::fmt;
use std::fmt::Display;

use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;

use self::internal::OutputHandle;

use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
    sc_stream_delegate::SCStreamDelegateTrait,
};

mod internal {

    #![allow(non_snake_case)]
    #![allow(clippy::needless_pass_by_value)]
    use std::{ffi::c_void, marker::PhantomData, ops::Deref, ptr, sync::Once};

    use crate::{
        stream::{
            sc_content_filter::SCContentFilter,
            sc_stream_configuration::SCStreamConfiguration,
            sc_stream_delegate::{SCStreamDelegate, SCStreamDelegateTrait},
        },
        utils::{
            block::{new_void_completion_handler, VoidCompletionHandler},
            objc::get_concrete_from_void,
        },
    };
    use core_foundation::{base::TCFType, error::CFError};
    use core_media_rs::cm_sample_buffer::CMSampleBuffer;
    use dispatch::{Queue, QueueAttribute};
    use objc::{
        class,
        declare::ClassDecl,
        msg_send,
        runtime::{self, Object, Sel},
        sel, sel_impl, Message,
    };

    use super::{SCStreamOutputTrait, SCStreamOutputType};

    unsafe impl objc::Encode for SCStreamOutputType {
        fn encode() -> objc::Encoding {
            i8::encode()
        }
    }

    pub struct SCStream<T: SCStreamOutputTrait> {
        traits: Vec<TraitHolder<T>>,
        obj: *mut Object,
    }

    impl<T: SCStreamOutputTrait> Drop for SCStream<T> {
        fn drop(&mut self) {
            unsafe {
                self.traits.iter().for_each(|t| {
                    ptr::drop_in_place(t.trait_object);
                });

                runtime::object_dispose(self.obj);
            }
        }
    }
    #[repr(C)]
    struct TraitHolder<T: SCStreamOutputTrait> {
        pub trait_object: *mut Box<T>,
        data: PhantomData<T>,
    }

    impl<T: SCStreamOutputTrait> Deref for TraitHolder<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.trait_object }
        }
    }

    impl<T: SCStreamOutputTrait> Clone for TraitHolder<T> {
        fn clone(&self) -> Self {
            Self {
                trait_object: self.trait_object,
                data: PhantomData,
            }
        }
    }

    unsafe impl<T: SCStreamOutputTrait> objc::Encode for TraitHolder<T> {
        fn encode() -> objc::Encoding {
            unsafe { objc::Encoding::from_str("^v") }
        }
    }
    unsafe impl<T: SCStreamOutputTrait> Message for TraitHolder<T> {}
    impl<T: SCStreamOutputTrait> TraitHolder<T> {
        pub fn from(output: T) -> Self {
            Self {
                trait_object: Box::into_raw(Box::new(Box::new(output))),
                data: PhantomData,
            }
        }
    }

    pub type OutputHandle = *mut Object;
    impl<T: SCStreamOutputTrait> SCStream<T> {
        fn internal_register_output_class() {
            type StreamOutputMethod =
                extern "C" fn(&Object, Sel, *mut Object, *const c_void, SCStreamOutputType);

            extern "C" fn stream_output<T: SCStreamOutputTrait>(
                this: &Object,
                _cmd: Sel,
                _stream_ref: *mut Object,
                sample_buffer_ref: *const c_void,
                of_type: SCStreamOutputType,
            ) {
                let holder = unsafe { this.get_ivar::<TraitHolder<T>>("trait_holder") };
                let sample_buffer: CMSampleBuffer =
                    unsafe { get_concrete_from_void(sample_buffer_ref) };
                holder.did_output_sample_buffer(sample_buffer, of_type);
            }
            let mut decl =
                ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");
            unsafe {
                let output_handler: StreamOutputMethod = stream_output::<T>;
                decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
                decl.add_ivar::<TraitHolder<T>>("trait_holder");
                decl.register();
            }
        }

        pub(crate) fn internal_remove_output_handler(
            &mut self,
            output_handle: OutputHandle,
            of_type: SCStreamOutputType,
        ) {
            unsafe {
                let error = runtime::class_createInstance(class!(NSObject), 0);
                let _: () = msg_send![self.obj, removeStreamOutput: output_handle type: of_type error:error];
            }
        }

        pub(crate) fn internal_add_output_handler(
            &mut self,
            outputTrait: T,
            of_type: SCStreamOutputType,
        ) -> OutputHandle {
            static REGISTER_ONCE: Once = Once::new();
            REGISTER_ONCE.call_once(Self::internal_register_output_class);

            unsafe {
                let output = runtime::class_createInstance(class!(StreamOutput), 0);
                let t = TraitHolder::from(outputTrait);
                (*output).set_ivar("trait_holder", t.clone());
                let error = runtime::class_createInstance(class!(NSObject), 0);

                let queue = Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent);

                let _: () = msg_send![self.obj, addStreamOutput: output type: of_type sampleHandlerQueue: queue  error: error];
                self.traits.push(t);
                output
            }
        }

        pub(crate) fn internal_init_with_filter_and_delegate(
            filter: &SCContentFilter,
            configuration: &SCStreamConfiguration,
            stream_delegate: impl SCStreamDelegateTrait,
        ) -> Self {
            unsafe {
                let obj: *mut Object = msg_send![class!(SCStream), alloc];
                let obj: *mut Object = msg_send![obj, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];
                Self {
                    obj,
                    traits: vec![],
                }
            }
        }
        pub(crate) fn internal_init_with_filter(
            filter: SCContentFilter,
            configuration: SCStreamConfiguration,
        ) -> Self {
            struct NoopDelegate;
            impl SCStreamDelegateTrait for NoopDelegate {
                fn did_stop_with_error(&self, _error: CFError) {}
            }
            Self::internal_init_with_filter_and_delegate(&filter, &configuration, NoopDelegate)
        }
        /// Returns the internal start capture of this [`SCStream<T>`].
        ///
        /// # Panics
        ///
        /// Panics if .
        ///
        /// # Errors
        ///
        /// This function will return an error if .
        pub(crate) fn internal_start_capture(&self) -> Result<(), CFError> {
            unsafe {
                let VoidCompletionHandler(handler, rx) = new_void_completion_handler();
                let _: () = msg_send![self.obj, startCaptureWithCompletionHandler: handler];

                rx.recv()
                    .expect("Should receive a return from completion handler")
            }
        }
        /// Returns the internal stop capture of this [`SCStream<T>`].
        ///
        /// # Panics
        ///
        /// Panics if .
        ///
        /// # Errors
        ///
        /// This function will return an error if .
        pub(crate) fn internal_stop_capture(&self) -> Result<(), CFError> {
            unsafe {
                let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

                let _: () = msg_send![self.obj, stopCaptureWithCompletionHandler: handler];

                rx.recv()
                    .expect("Should receive a return from completion handler")
            }
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
impl Display for SCStreamOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Screen => write!(f, "Screen"),
            Self::Audio => write!(f, "Audio"),
        }
    }
}
pub trait SCStreamOutputTrait: Sync + Send {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType);
}
impl<T: SCStreamOutputTrait> SCStream<T> {
    pub fn new_with_error_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> Self {
        Self::internal_init_with_filter_and_delegate(filter, configuration, stream_delegate)
    }

    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        Self::internal_init_with_filter(filter.clone(), configuration.clone())
    }

    pub fn add_output_handler(
        &mut self,
        output_trait: T,
        of_type: SCStreamOutputType,
    ) -> OutputHandle {
        self.internal_add_output_handler(output_trait, of_type)
    }

    pub fn remove_output_handler(
        &mut self,
        outout_handle: OutputHandle,
        of_type: SCStreamOutputType,
    ) {
        self.internal_remove_output_handler(outout_handle, of_type);
    }

    /// Returns the start capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn start_capture(&self) -> Result<(), CFError> {
        self.internal_start_capture()
    }
    /// Returns the stop capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn stop_capture(&self) -> Result<(), CFError> {
        self.internal_stop_capture()
    }
}

#[cfg(test)]
mod stream_test {

    use std::sync::mpsc::{channel, Sender};

    use core_foundation::error::CFError;
    use core_media_rs::cm_sample_buffer::CMSampleBuffer;

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::{SCStream, SCStreamOutputTrait, SCStreamOutputType};

    #[derive(Debug)]
    struct TestStreamOutput {
        sender: Sender<SCStreamOutputType>,
    }

    impl SCStreamOutputTrait for TestStreamOutput {
        fn did_output_sample_buffer(
            &self,
            _sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            self.sender
                .send(of_type)
                .expect("could not send from output buffer");
        }
    }
    #[test]
    fn test_remove_output_handler() -> Result<(), CFError> {
        let c = channel();
        let output_handler = TestStreamOutput { sender: c.0 };
        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        let display = SCShareableContent::get().unwrap().displays().remove(0);
        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        let id = stream.add_output_handler(output_handler, SCStreamOutputType::Screen);
        stream.remove_output_handler(id, SCStreamOutputType::Screen);
        drop(stream);
        Ok(())
    }

    #[test]
    fn test_sc_stream() -> Result<(), CFError> {
        for expected_type in [SCStreamOutputType::Screen, SCStreamOutputType::Audio] {
            let (tx, rx) = channel();

            let stream = {
                let config = SCStreamConfiguration::new()
                    .set_captures_audio(true)?
                    .set_width(100)?
                    .set_height(100)?;

                let display = SCShareableContent::get().unwrap().displays().remove(0);
                let filter =
                    SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
                let mut stream = SCStream::new(&filter, &config);
                stream.add_output_handler(TestStreamOutput { sender: tx.clone() }, expected_type);
                stream
            };
            stream.start_capture()?;
            let got_type = rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("could not receive from output_buffer");
            assert_eq!(got_type, expected_type);
            stream.stop_capture()?;
            drop(stream);
        }
        Ok(())
    }
}
