use core_foundation::error::CFError;

use self::internal::SCStream;

use super::{
    sc_content_filter::SCContentFilter, sc_stream_configuration::SCConfiguration,
    sc_stream_delegate::SCStreamDelegateTrait,
};

mod internal {

    #![allow(non_snake_case)]

    use std::ffi::c_void;

    use crate::{
        stream::{
            sc_content_filter::SCContentFilter,
            sc_stream_delegate::{SCStreamDelegate, SCStreamDelegateTrait},
        },
        utils::{
            block::{new_completion_handler, CompletionHandler},
            objc::impl_deref,
        },
    };
    use core_foundation::{base::*, declare_TCFType, error::CFError, impl_TCFType};
    use dispatch::{Queue, QueueAttribute};
    use objc::*;

    use super::SCConfiguration;

    #[repr(C)]
    pub struct __SCStreamRef(c_void);
    extern "C" {
        pub fn SCStreamGetTypeID() -> CFTypeID;
    }

    pub type SCStreamRef = *mut __SCStreamRef;

    declare_TCFType! {SCStream, SCStreamRef}
    impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);
    impl_deref!(SCStream);

    pub(crate) fn init_with_filter(
        filter: &SCContentFilter,
        configuration: &SCConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> SCStream {
        unsafe {
            let instance: SCStreamRef = msg_send![class!(SCStream), alloc];
            let instance = msg_send![instance, initWithFilter: *filter  configuration: *configuration delegate: SCStreamDelegate::new(stream_delegate)];
            SCStream::wrap_under_create_rule(instance)
        }
    }
    pub(crate) fn start_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_completion_handler();
            let _: () = msg_send![*stream, startCaptureWithCompletionHandler: handler];
            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }
    pub(crate) fn stop_capture(stream: &SCStream) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_completion_handler();
            let _: () = msg_send![*stream, stopCaptureWithCompletionHandler: handler];
            rx.recv()
                .expect("Should receive a return from completion handler")
        }
    }

    pub(crate) fn add_stream_output() {
        let queue = Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent);
        todo!()
        // let a = UnsafeSCStreamOutputHandler::init(handle);
        // unsafe {
        //     let _: () = msg_send!(self, addStreamOutput: a type: output_type sampleHandlerQueue: queue error: NSObject::new());
        // }
    }
}

impl SCStream {
    pub fn new(
        filter: &SCContentFilter,
        configuration: &SCConfiguration,
        stream_delegate: impl SCStreamDelegateTrait,
    ) -> Self {
        internal::init_with_filter(filter, configuration, stream_delegate)
    }

    pub fn start_capture(&self) -> Result<(), CFError> {
        internal::start_capture(self)
    }
    pub fn stop_capture(&self) -> Result<(), CFError> {
        internal::stop_capture(self)
    }
    pub fn add_stream_output(&self, handle: impl UnsafeSCStreamOutput, output_type: u8) {}
}

impl Drop for SCStream {
    fn drop(self) {
        if let Err(err) = self.stop_capture() {
            eprintln!("Cannot stop capture: {:?}", err)
        }
    }
}

#[cfg(test)]
mod stream_test {
    //
    // use super::{UnsafeSCStream, UnsafeSCStreamError};
    // use crate::{
    //     cm_sample_buffer_ref::CMSampleBufferRef,
    //     content_filter::{UnsafeContentFilter, UnsafeInitParams::Display},
    //     shareable_content::UnsafeSCShareableContent,
    //     stream_configuration::UnsafeStreamConfiguration,
    //     stream_output_handler::UnsafeSCStreamOutput,
    // };
    // struct ErrorHandler {}
    // #[repr(C)]
    // struct OutputHandler {
    //     tx: SyncSender<Id<CMSampleBufferRef>>,
    // }
    // impl Drop for OutputHandler {
    //     fn drop(&mut self) {
    //         println!("DROPPP");
    //     }
    // }
    // impl UnsafeSCStreamError for ErrorHandler {
    //     fn handle_error(&self) {
    //         eprintln!("ERROR!");
    //     }
    // }
    // impl UnsafeSCStreamOutput for OutputHandler {
    //     fn did_output_sample_buffer(&self, sample: Id<CMSampleBufferRef>, _of_type: u8) {
    //         self.tx.send(sample).unwrap();
    //     }
    // }
    // #[test]
    // #[cfg_attr(feature = "ci", ignore)]
    // fn test_sc_stream() {
    //     let display = UnsafeSCShareableContent::get()
    //         .unwrap()
    //         .displays()
    //         .pop()
    //         .expect("could not get display");
    //
    //     let filter = UnsafeContentFilter::init(Display(display));
    //     let config = UnsafeStreamConfiguration {
    //         width: 100,
    //         height: 100,
    //         ..Default::default()
    //     };
    //     let (tx, rx) = sync_channel(1);
    //     let stream = UnsafeSCStream::init(filter, config.into(), ErrorHandler {});
    //     let a = OutputHandler { tx };
    //
    //     println!("ADDING OUTPUT");
    //     stream.add_stream_output(a, 0);
    //     println!("start capture");
    //     stream.start_capture().expect("start");
    //     println!("{:?}", rx.recv().unwrap());
    //     stream.stop_capture().expect("stop");
    // }
    //
    // #[test]
    // #[cfg_attr(feature = "ci", ignore)]
    // fn test_sc_stream_error_handling() {
    //     let display = UnsafeSCShareableContent::get()
    //         .unwrap()
    //         .displays()
    //         .pop()
    //         .expect("could not get display");
    //
    //     let filter = UnsafeContentFilter::init(Display(display));
    //     let config = UnsafeStreamConfiguration {
    //         width: 100,
    //         height: 100,
    //         ..Default::default()
    //     };
    //     let stream = UnsafeSCStream::init(filter, config.into(), ErrorHandler {});
    //
    //     println!("start capture");
    //     assert!(stream.start_capture().is_ok());
    //     assert!(stream.start_capture().is_err()); // already started error
    //     assert!(stream.stop_capture().is_ok());
    //     assert!(stream.stop_capture().is_err()); // already stopped error
    // }
}
