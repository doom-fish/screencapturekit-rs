use core::fmt;
use std::fmt::Display;

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
        collections::HashMap,
        ffi::c_void,
        sync::{atomic::AtomicU32, Once, RwLock},
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
            objc::{create_concrete_from_void, get_concrete_from_void, MessageForTFType},
        },
    };
    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType,
        error::CFError,
        impl_TCFType,
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

    #[repr(C)]
    pub struct __SCStreamRef(c_void);
    pub type SCStreamRef = *mut __SCStreamRef;

    extern "C" {
        pub fn SCStreamGetTypeID() -> CFTypeID;
    }
    declare_TCFType! {SCStream, SCStreamRef}
    impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);

    unsafe impl objc::Encode for SCStreamOutputType {
        fn encode() -> objc::Encoding {
            i8::encode()
        }
    }

    type SenderMap = HashMap<u32, Box<dyn SCStreamOutputTrait + 'static + Send + Sync>>;
    static OUTPUT_HANDLERS: Lazy<RwLock<SenderMap>> = Lazy::new(|| RwLock::new(HashMap::new()));

    impl SCStream {
        fn internal_register_objc_class() {
            type StreamOutputMethod =
                extern "C" fn(&Object, Sel, *const c_void, *const c_void, SCStreamOutputType);
            extern "C" fn stream_output(
                this: &Object,
                _cmd: Sel,
                stream_ref: *const c_void,
                sample_buffer_ref: *const c_void,
                of_type: SCStreamOutputType,
            ) {
                let id: &u32 = unsafe { this.get_ivar("channel_hash") };
                let stream: SCStream = unsafe { get_concrete_from_void(stream_ref) };
                let cm_sample_buffer: CMSampleBuffer =
                    unsafe { get_concrete_from_void(sample_buffer_ref) };
                OUTPUT_HANDLERS
                    .read()
                    .map(|m| {
                        m.get(id)
                            .expect("should have a sender")
                            .did_output_sample_buffer(stream, cm_sample_buffer, of_type);
                    })
                    .expect("should be able to obtain lock");
            }
            let mut decl =
                ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");

            unsafe {
                let output_handler: StreamOutputMethod = stream_output;

                decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
                decl.add_ivar::<u32>("channel_hash");
                decl.register();
            }
        }
        /// .
        ///
        /// # Panics
        ///
        /// Panics if .
        pub fn internal_add_output_handler(
            &mut self,
            output: impl SCStreamOutputTrait,
            of_type: SCStreamOutputType,
        ) {
            static REGISTER_ONCE: Once = Once::new();
            static ID_COUNTER: AtomicU32 = AtomicU32::new(0);

            REGISTER_ONCE.call_once(Self::internal_register_objc_class);
            let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            unsafe {
                let output = runtime::class_createInstance(class!(StreamOutput), 0);
                (*output).set_ivar("channel_hash", id);
                let error = runtime::class_createInstance(class!(NSObject), 0);
                let queue = Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent);

                let _: () = msg_send![self.as_sendable(), addStreamOutput: output type: of_type sampleHandlerQueue: queue  error: error];
            }
            OUTPUT_HANDLERS
                .write()
                .expect("should be able to obtain lock")
                .insert(id, Box::new(output));
        }

        pub fn internal_init_with_filter_and_delegate(
            filter: &SCContentFilter,
            configuration: &SCStreamConfiguration,
            stream_delegate: impl SCStreamDelegateTrait,
        ) -> Self {
            unsafe {
                let instance: *mut Object = msg_send![class!(SCStream), alloc];
                let instance: *mut c_void = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];
                create_concrete_from_void(instance)
            }
        }
        pub fn internal_init_with_filter(
            filter: SCContentFilter,
            configuration: SCStreamConfiguration,
        ) -> Self {
            struct NoopDelegate;
            impl SCStreamDelegateTrait for NoopDelegate {
                fn did_stop_with_error(&self, _error: CFError) {}
            }
            unsafe {
                let instance: *mut Object = msg_send![class!(SCStream), alloc];
                let instance: *mut c_void = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: NoopDelegate];
                create_concrete_from_void(instance)
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
                let _: () =
                    msg_send![self.as_sendable(), startCaptureWithCompletionHandler: handler];

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

                let _: () =
                    msg_send![self.as_sendable(), stopCaptureWithCompletionHandler: handler];

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
pub trait SCStreamOutputTrait: 'static + Sync + Send {
    fn did_output_sample_buffer(
        &self,
        stream: SCStream,
        sample_buffer: CMSampleBuffer,
        of_type: SCStreamOutputType,
    );
}
impl SCStream {
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
    pub fn add_output_handler(
        &mut self,
        output_trait: impl SCStreamOutputTrait,
        of_type: SCStreamOutputType,
    ) {
        self.internal_add_output_handler(output_trait, of_type);
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

    use crate::{
        core_media::cm_sample_buffer::CMSampleBuffer,
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
            _stream: SCStream,
            _sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            self.sender
                .send(of_type)
                .expect("could not send from output buffer");
        }
    }
    #[test]
    fn test_sc_stream() -> Result<(), CFError> {
        for expected_type in [SCStreamOutputType::Screen, SCStreamOutputType::Audio] {
            let (tx, rx) = channel();
            let output_handler = TestStreamOutput { sender: tx };

            let stream = {
                let config = SCStreamConfiguration::new()
                    .set_captures_audio(true)?
                    .set_width(100)?
                    .set_height(100)?;

                let display = SCShareableContent::get().unwrap().displays().remove(1);
                let filter =
                    SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
                let mut stream = SCStream::new(&filter, &config);
                stream.add_output_handler(output_handler, expected_type);
                stream
            };
            stream.start_capture()?;
            let got_type = rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("could not receive from output_buffer");

            assert_eq!(got_type, expected_type);
            stream.stop_capture()?;
        }
        Ok(())
    }
}
