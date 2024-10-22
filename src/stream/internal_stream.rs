use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
    ptr,
    sync::Once,
};

use crate::{
    stream::{sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration},
    utils::{
        block::{new_void_completion_handler, CompletionHandler},
        objc::get_concrete_from_void,
    },
};
use core_foundation::base::TCFType;
use core_foundation::error::CFError;
use dispatch::{Queue, QueuePriority};

use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{self, Object, Sel},
    sel, sel_impl, Encode,
};

use super::{
    sc_stream_delegate::SCStreamDelegate, sc_stream_output_trait::SCStreamOutputTrait,
    sc_stream_output_type::SCStreamOutputType,
};

#[repr(transparent)]
#[derive(Debug)]
struct TraitTransfer<'a>(*mut Box<dyn SCStreamOutputTrait + 'a>);

impl<'a> TraitTransfer<'a> {
    pub fn new(handler: impl SCStreamOutputTrait + 'a) -> Self {
        Self(Box::into_raw(Box::new(Box::new(handler))))
    }
    pub fn drop_handler(&self) {
        unsafe {
            drop(Box::from_raw(self.0));
        }
    }
}

impl<'a> Deref for TraitTransfer<'a> {
    type Target = Box<dyn SCStreamOutputTrait>;
    fn deref(&self) -> &'a Self::Target {
        unsafe { &*self.0.cast() }
    }
}

impl<'a> DerefMut for TraitTransfer<'a> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { &mut *self.0.cast() }
    }
}

impl Clone for TraitTransfer<'_> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

unsafe impl Encode for TraitTransfer<'_> {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("r^v") }
    }
}

pub struct SCStream<'a> {
    inner: *mut Object,
    handlers: Vec<TraitTransfer<'a>>,
    cleanup: Vec<*mut Object>,
}

impl Drop for SCStream<'_> {
    fn drop(&mut self) {
        unsafe {
            runtime::objc_release(self.inner);
        }
        self.handlers.iter().for_each(|h| {
            h.drop_handler();
        });
        self.cleanup.iter().for_each(|o| unsafe {
            runtime::objc_release(*o);
        });
    }
}

impl<'a> SCStream<'a> {
    pub fn internal_init_with_filter_and_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Self {
        Self {
            inner: unsafe {
                let inner: *mut Object = msg_send![class!(SCStream), alloc];
                let inner: *mut Object = msg_send![inner, initWithFilter: filter.clone().as_CFTypeRef()  configuration: configuration.clone().as_CFTypeRef() delegate: ptr::null_mut::<SCStreamDelegate>()];
                inner
            },
            handlers: vec![],
            cleanup: vec![],
        }
    }

    fn internal_register_output_handler() {
        type StreamOutputMethod =
            extern "C" fn(&Object, Sel, *mut Object, *const c_void, SCStreamOutputType);

        extern "C" fn stream_output(
            this: &Object,
            _cmd: Sel,
            _stream_ref: *mut Object,
            sample_buffer_ref: *const c_void,
            of_type: SCStreamOutputType,
        ) {
            let stream_output: &TraitTransfer = unsafe { this.get_ivar("rust_handler") };
            let sample_buffer = unsafe { get_concrete_from_void(sample_buffer_ref) };

            stream_output.did_output_sample_buffer(sample_buffer, of_type);
            // println!("Output: {stream_output:p}");
        }
        let mut decl =
            ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");
        unsafe {
            let output_handler: StreamOutputMethod = stream_output;
            decl.add_ivar::<TraitTransfer>("rust_handler");
            decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
            decl.register();
        }
    }
    pub fn run_register(&mut self) {
        static REGISTER_ONCE: Once = Once::new();
        REGISTER_ONCE.call_once(|| {
            Self::internal_register_output_handler();
        });
    }
    pub fn internal_remove_output_handler(&mut self, _index: usize, _of_type: SCStreamOutputType) {}

    pub fn internal_add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait + 'a,
        of_type: SCStreamOutputType,
    ) -> usize {
        self.run_register();

        unsafe {
            let error: *mut Object = ptr::null_mut();
            let sc_handler = runtime::class_createInstance(class!(StreamOutput), 0);

            let inner = TraitTransfer::new(handler);

            (*sc_handler).set_ivar("rust_handler", inner.clone());
            let stream_queue = Queue::global(QueuePriority::Low);

            match of_type {
                SCStreamOutputType::Screen => {
                    let _: () = msg_send![self.inner, addStreamOutput: sc_handler type: SCStreamOutputType::Screen sampleHandlerQueue: stream_queue error: error];
                }
                SCStreamOutputType::Audio => {
                    let _: () = msg_send![self.inner, addStreamOutput: sc_handler type: SCStreamOutputType::Audio sampleHandlerQueue: stream_queue error: error];
                }
            }
            self.handlers.push(inner);
            self.cleanup.push(sc_handler);
        };

        0
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
            let CompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![self.inner, startCaptureWithCompletionHandler: handler];

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
            let CompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![self.inner, stopCaptureWithCompletionHandler: handler];

            rx.recv()
                .expect("Should receive a return from completion handler")
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
        let mut stream = SCStream::internal_init_with_filter_and_delegate(&filter, &config);

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
