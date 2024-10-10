use std::{ffi::c_void, marker::PhantomPinned, ptr, sync::Once};

use crate::{
    stream::{
        sc_content_filter::SCContentFilter,
        sc_stream_configuration::SCStreamConfiguration,
        sc_stream_delegate::{SCStreamDelegate, SCStreamDelegateTrait},
    },
    utils::{
        block::{new_void_completion_handler, VoidCompletionHandler},
        objc::{create_concrete_from_void, get_concrete_from_void},
    },
};
use core_foundation::base::TCFType;
use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use dispatch::{Queue, QueuePriority};

use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{self, Object, Sel},
    sel, sel_impl, Encode,
};

use super::{
    sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
};

unsafe impl Encode for &mut SCStream<'_> {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("^v") }
    }
}

pub struct SCStream<'a> {
    inner: *mut Object,
    output_screen: Vec<Box<dyn SCStreamOutputTrait + 'a>>,
    output_audio: Vec<Box<dyn SCStreamOutputTrait + 'a>>,
    output_handle: *mut Object,
    _pin: PhantomPinned,
    _delegate: Box<dyn SCStreamDelegateTrait + 'static>,
}

impl Drop for SCStream<'_> {
    fn drop(&mut self) {
        unsafe {
            runtime::objc_release(self.inner);
            runtime::objc_release(self.output_handle);
        }
    }
}

impl<'a> SCStream<'a> {
    fn call_handlers(&self, sample_buffer: &CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Audio => {
                self.output_audio.iter().for_each(|handler| {
                    handler.did_output_sample_buffer(sample_buffer.clone(), of_type);
                });
            }
            SCStreamOutputType::Screen => {
                self.output_screen.iter().for_each(|handler| {
                    handler.did_output_sample_buffer(sample_buffer.clone(), of_type);
                });
            }
        }
    }
    pub fn internal_init_with_filter_and_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: impl SCStreamDelegateTrait,
    ) -> Self {
        Self {
            inner: unsafe {
                let inner: *mut Object = msg_send![class!(SCStream), alloc];
                let inner: *mut Object = msg_send![inner, initWithFilter: filter.as_CFTypeRef()  configuration: configuration.as_CFTypeRef() delegate: ptr::null_mut::<SCStreamDelegate>()];
                inner
            },
            output_handle: ptr::null_mut(),
            output_audio: vec![],
            output_screen: vec![],
            _delegate: Box::new(delegate),
            _pin: PhantomPinned,
        }
    }

    fn internal_register_output_handler(&mut self) {
        type StreamOutputMethod =
            extern "C" fn(&Object, Sel, *mut Object, *const c_void, SCStreamOutputType);

        extern "C" fn stream_output(
            this: &Object,
            _cmd: Sel,
            _stream_ref: *mut Object,
            sample_buffer_ref: *const c_void,
            of_type: SCStreamOutputType,
        ) {
            let sc_stream: &&mut SCStream = unsafe { this.get_ivar("super") };
            let sample_buffer: CMSampleBuffer =
                unsafe { get_concrete_from_void(sample_buffer_ref) };
            sc_stream.call_handlers(&sample_buffer, of_type);
        }
        let mut decl =
            ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");
        unsafe {
            let output_handler: StreamOutputMethod = stream_output;
            decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
            decl.add_ivar::<&mut Self>("super");
            decl.register();
            self.output_handle = runtime::class_createInstance(class!(StreamOutput), 0);
            let error = runtime::class_createInstance(class!(NSObject), 0);
            let _: () = msg_send![self.inner, addStreamOutput: self.output_handle type: SCStreamOutputType::Audio  sampleHandlerQueue: Queue::global(QueuePriority::High) error: error];
            let _: () = msg_send![self.inner, addStreamOutput: self.output_handle type: SCStreamOutputType::Screen sampleHandlerQueue: Queue::global(QueuePriority::High) error: error];
            (*self.output_handle).set_ivar("super", self);
        }
    }
    pub fn run_register(&mut self) {
        static REGISTER_ONCE: Once = Once::new();
        REGISTER_ONCE.call_once(|| Self::internal_register_output_handler(self));
    }
    pub fn internal_remove_output_handler(&mut self, index: usize, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Audio => self.output_audio.remove(index),
            SCStreamOutputType::Screen => self.output_screen.remove(index),
        };
    }
    pub fn internal_add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait + 'a,
        of_type: SCStreamOutputType,
    ) -> usize {
        self.run_register();
        match of_type {
            SCStreamOutputType::Audio => {
                self.output_audio.push(Box::new(handler));
                self.output_audio.len() - 1
            }
            SCStreamOutputType::Screen => {
                self.output_screen.push(Box::new(handler));
                self.output_screen.len() - 1
            }
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
            let VoidCompletionHandler(handler, rx) = new_void_completion_handler();

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
            sc_stream_delegate::SCStreamDelegateTrait, sc_stream_output_trait::SCStreamOutputTrait,
            sc_stream_output_type::SCStreamOutputType,
        },
    };

    use super::SCStream;

    struct OutputHandler<'a> {
        pub test_ref: &'a str,
        pub output: String,
    }
    impl<'a> SCStreamOutputTrait for OutputHandler<'a> {
        fn did_output_sample_buffer(
            &self,
            sample_buffer: core_media_rs::cm_sample_buffer::CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            println!("Output: {}", self.output);
            println!("Test ref2: {:?}", self.test_ref);
            println!("Sample buffer: {sample_buffer:?}");
            println!("Output type: {of_type:?}");
        }
    }
    struct OutputHandler2 {
        pub output: String,
    }
    impl SCStreamOutputTrait for OutputHandler2 {
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
    struct NoopDelegate;
    impl SCStreamDelegateTrait for NoopDelegate {}

    #[test]
    fn create() -> Result<(), CFError> {
        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        let display = SCShareableContent::get()?.displays().remove(0);

        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let s = String::from("Screen Ref");
        let mut stream =
            SCStream::internal_init_with_filter_and_delegate(&filter, &config, NoopDelegate);

        stream.internal_add_output_handler(
            OutputHandler {
                test_ref: &s,
                output: "Screen".to_string(),
            },
            SCStreamOutputType::Screen,
        );
        stream.internal_add_output_handler(
            OutputHandler2 {
                output: "Audio".to_string(),
            },
            SCStreamOutputType::Audio,
        );

        stream.internal_start_capture()?;

        thread::sleep(Duration::from_secs(1));
        stream.internal_stop_capture()
    }
}
