use core::fmt;
use std::{fmt::Display, sync::Once};

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
    };

    use crate::{
        core_media::cm_sample_buffer::CMSampleBuffer,
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
        pub(crate) audio_handlers: Vec<T>,
        pub(crate) screen_handlers: Vec<T>,
    }

    static QUEUE: Lazy<Queue> =
        Lazy::new(|| Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent));

    unsafe impl objc::Encode for SCStreamOutputType {
        fn encode() -> objc::Encoding {
            i8::encode()
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

    impl<T: SCStreamOutputTrait> SCStream<T> {
        fn register_objc_class() {
            type StreamOutputMethod =
                extern "C" fn(&Object, Sel, *const c_void, *const c_void, SCStreamOutputType);
            extern "C" fn stream_output<T: SCStreamOutputTrait>(
                this: &Object,
                _cmd: Sel,
                _stream_ref: *const c_void,
                sample_buffer_ref: *const c_void,
                of_type: SCStreamOutputType,
            ) {
                unsafe {
                    let stream: &SCStream<T> = ptr::read(
                        this.get_ivar::<*mut c_void>(&format!("_stream_{of_type}"))
                            .cast(),
                    );
                    let sample_buffer: CMSampleBuffer = get_concrete_from_void(sample_buffer_ref);
                    stream.internal_handle_output(of_type, sample_buffer);
                };
            }
            let mut decl =
                ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");

            unsafe {
                let output_handler: StreamOutputMethod = stream_output::<T>;

                decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
                decl.add_ivar::<*const c_void>(&format!("_stream_{}", SCStreamOutputType::Screen));
                decl.add_ivar::<*const c_void>(&format!("_stream_{}", SCStreamOutputType::Audio));
                decl.register();
            }
        }
        pub fn internal_handle_output(
            &self,
            of_type: SCStreamOutputType,
            sample_buffer: CMSampleBuffer,
        ) {
            match of_type {
                SCStreamOutputType::Screen => {
                    self.screen_handlers.iter().for_each(|handler| {
                        handler.did_output_sample_buffer(self, sample_buffer.clone(), of_type);
                    });
                }
                SCStreamOutputType::Audio => {
                    self.audio_handlers.iter().for_each(|handler| {
                        handler.did_output_sample_buffer(self, sample_buffer.clone(), of_type);
                    });
                }
            }
        }

        pub fn internal_add_output_handler(&mut self, of_type: SCStreamOutputType) {
            Self::register_objc_class();
            unsafe {
                let output = runtime::class_createInstance(class!(StreamOutput), 0);
                (*output).set_ivar(
                    &format!("_stream_{of_type}"),
                    ptr::addr_of!(self).cast::<c_void>(),
                );
                let _: () = msg_send![*self, addStreamOutput: output type: of_type sampleHandlerQueue: QUEUE.clone() error: ptr::null_mut::<CFErrorRef>()];
            }
        }

        const fn internal_new(obj: *mut Object) -> Self {
            Self {
                obj,
                audio_handlers: Vec::new(),
                screen_handlers: Vec::new(),
            }
        }
        pub fn internal_init_with_filter_and_delegate(
            filter: &SCContentFilter,
            configuration: &SCStreamConfiguration,
            stream_delegate: impl SCStreamDelegateTrait,
        ) -> Self {
            unsafe {
                let instance: *mut Object = msg_send![class!(SCStream), alloc];
                let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];
                Self::internal_new(instance)
            }
        }
        pub fn internal_init_with_filter(
            filter: SCContentFilter,
            configuration: SCStreamConfiguration,
        ) -> Self {
            struct NoopDelegate;
            impl SCStreamDelegateTrait for NoopDelegate {}
            unsafe {
                let instance: *mut Object = msg_send![class!(SCStream), alloc];
                let instance: *mut Object = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(NoopDelegate)];
                Self::internal_new(instance)
            }
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
        pub fn internal_start_capture(&self) -> Result<(), CFError> {
            unsafe {
                let VoidCompletionHandler(handler, rx) = new_void_completion_handler();
                let _: () = msg_send![*self, startCaptureWithCompletionHandler: handler];

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
        pub fn internal_stop_capture(&self) -> Result<(), CFError> {
            unsafe {
                let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

                let _: () = msg_send![*self, stopCaptureWithCompletionHandler: handler];

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
pub trait SCStreamOutputTrait: Sized + fmt::Debug {
    fn did_output_sample_buffer(
        &self,
        stream: &SCStream<Self>,
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
        Self::internal_init_with_filter_and_delegate(filter, configuration, stream_delegate)
    }

    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        Self::internal_init_with_filter(filter.clone(), configuration.clone())
    }
    pub fn add_output_handler(&mut self, output_trait: T, of_type: SCStreamOutputType) {
        static ONLY_ONCE_AUDIO: Once = Once::new();
        static ONLY_ONCE_SCREEN: Once = Once::new();
        match of_type {
            SCStreamOutputType::Screen => {
                ONLY_ONCE_SCREEN.call_once(|| {
                    self.internal_add_output_handler(SCStreamOutputType::Screen);
                });
                self.screen_handlers.push(output_trait);
            }
            SCStreamOutputType::Audio => {
                ONLY_ONCE_AUDIO.call_once(|| {
                    self.internal_add_output_handler(SCStreamOutputType::Audio);
                });
                self.audio_handlers.push(output_trait);
            }
        }
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
            _stream: &SCStream<Self>,
            _sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            println!("Received a sample buffer from {of_type:?}");
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
            let mut screen = SCStream::new(&filter, &config);
            screen.add_output_handler(TestStreamOutput, SCStreamOutputType::Screen);
            screen
        };
        stream.start_capture().expect("Could not start capture");
        std::thread::sleep(Duration::from_secs(1));
        stream.stop_capture().expect("Could not stop capture");
        Ok(())
    }
}
