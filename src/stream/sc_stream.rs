use core_foundation::error::CFError;

use crate::output::sc_stream_output::{SCStreamOutputTrait, SCStreamOutputType};

use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
    sc_stream_delegate::SCStreamDelegateTrait,
};

mod internal {

    #![allow(non_snake_case)]

    use std::ffi::c_void;

    use crate::{
        output::sc_stream_output::{SCStreamOutput, SCStreamOutputTrait, SCStreamOutputType},
        stream::{
            sc_content_filter::SCContentFilter,
            sc_stream_configuration::SCStreamConfiguration,
            sc_stream_delegate::{SCStreamDelegate, SCStreamDelegateTrait},
        },
        utils::{
            block::{new_void_completion_handler, VoidCompletionHandler},
            objc::MessageForTFType,
        },
    };
    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType,
        error::{CFError, CFErrorRef},
        impl_TCFType,
    };
    use dispatch::{Queue, QueueAttribute};
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};

    #[repr(C)]
    pub struct __SCStreamRef(c_void);
    extern "C" {
        pub fn SCStreamGetTypeID() -> CFTypeID;
    }

    pub type SCStreamRef = *mut __SCStreamRef;

    declare_TCFType! {SCStream, SCStreamRef}
    impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);

    pub fn init_with_filter_and_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> SCStream {
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: SCStreamRef = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(stream_delegate)];

            SCStream::wrap_under_create_rule(instance)
        }
    }
    pub fn init_with_filter(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> SCStream {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        unsafe {
            let instance: *mut Object = msg_send![class!(SCStream), alloc];
            let instance: SCStreamRef = msg_send![instance, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: SCStreamDelegate::new(NoopDelegate)];

            SCStream::wrap_under_create_rule(instance)
        }
    }
    pub fn start_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![stream.as_sendable(), startCaptureWithCompletionHandler: handler];
            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
    pub fn stop_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![stream.as_sendable(), stopCaptureWithCompletionHandler: handler];
            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }

    pub fn add_stream_output(
        stream: &mut SCStream,
        stream_output: impl SCStreamOutputTrait,
        output_type: SCStreamOutputType,
    ) {
        let queue = Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent);
        let stream_output = SCStreamOutput::new(stream_output);

        unsafe {
            let _: () = msg_send![stream.as_sendable(), addStreamOutput: stream_output type: output_type sampleHandlerQueue: queue error: std::ptr::null::<CFErrorRef>()];
        };
    }
}
pub use internal::{SCStream, SCStreamRef};

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
        internal::init_with_filter(filter, configuration)
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

    pub fn add_stream_output(
        &mut self,
        stream_output: impl SCStreamOutputTrait,
        output_type: SCStreamOutputType,
    ) {
        internal::add_stream_output(self, stream_output, output_type);
    }
}

#[cfg(test)]
mod stream_test {
    use std::{
        error::Error,
        sync::{
            mpsc::{sync_channel, SyncSender},
            Arc,
        },
        time::Duration,
    };

    use crate::{
        output::sc_stream_output::{SCStreamOutputTrait, SCStreamOutputType},
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::SCStream;

    #[test]
    fn test_sc_stream() -> Result<(), Box<dyn Error>> {
        struct TestStreamOutput {
            tx: SyncSender<()>,
        }
        impl SCStreamOutputTrait for TestStreamOutput {
            fn did_output_sample_buffer(
                &self,
                _stream: SCStream,
                _sample_buffer: crate::core_media::cm_sample_buffer::CMSampleBuffer,
                _of_type: SCStreamOutputType,
            ) {
                self.tx.send(()).expect("could not send");
            }
        }

        // let (tx_screen, rx_screen) = sync_channel(2);
        let (tx_audio, rx_audio) = sync_channel(1);

        let config = SCStreamConfiguration::new().set_captures_audio(true)?;
        let display = SCShareableContent::get().unwrap().displays().remove(1);
        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        // stream.add_stream_output(
        //     TestStreamOutput { tx: tx_screen },
        //     SCStreamOutputType::Screen,
        // );
        stream.add_stream_output(TestStreamOutput { tx: tx_audio }, SCStreamOutputType::Audio);
        stream.start_capture()?;
        // rx_screen
        //     .recv_timeout(Duration::from_secs(2))
        //     .expect("could not receive");
        rx_audio
            .recv_timeout(Duration::from_secs(5))
            .expect("could not receive");
        stream.stop_capture()?;
        Ok(())
    }
}
