use crate::{core_media::cm_sample_buffer::CMSampleBuffer, stream::sc_stream::SCStream};

mod internal {

    #![allow(non_snake_case)]

    use std::{
        collections::HashMap,
        error::Error,
        ffi::c_void,
        mem,
        ops::Deref,
        sync::{Once, RwLock},
    };

    use objc::{
        class,
        declare::ClassDecl,
        runtime,
        runtime::{Class, Object, Sel},
        sel, sel_impl,
    };
    use once_cell::sync::Lazy;

    use crate::{
        core_media::cm_sample_buffer::CMSampleBuffer,
        output::sc_stream_output::{SCStreamOutputTrait, SCStreamOutputType},
        stream::sc_stream::SCStream,
        utils::{hash::hash, objc::get_concrete_from_void},
    };

    static OUTPUT_HANDLERS: Lazy<RwLock<HashMap<u64, Box<dyn SCStreamOutputTrait + Send + Sync>>>> =
        Lazy::new(|| RwLock::new(HashMap::new()));

    pub struct SCStreamOutput(pub *mut Object);

    impl Deref for SCStreamOutput {
        type Target = Object;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }

    impl Drop for SCStreamOutput {
        fn drop(&mut self) {
            unsafe {
                if let Some(delegate) = OUTPUT_HANDLERS
                    .write()
                    .expect("could not obtain read lock for OUTPUT_HANDLERS")
                    .remove(&hash(self.0))
                {
                    mem::drop(delegate);
                }
                core_foundation::base::CFRelease(self.0 as *const c_void);
            }
        }
    }
    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
        extern "C" fn stream_output(
            this: &mut Object,
            _cmd: Sel,
            stream_ref: *const c_void,
            sample_buffer_ref: *const c_void,
            of_type: i8,
        ) {
            unsafe {
                if let Some(output_handler) = OUTPUT_HANDLERS
                    .read()
                    .expect("could not obtain read lock for OUTPUT_HANDLERS")
                    .get(&hash(this as *mut _))
                {
                    let stream: SCStream = get_concrete_from_void(stream_ref);
                    let sample_buffer: CMSampleBuffer = get_concrete_from_void(sample_buffer_ref);
                    output_handler.did_output_sample_buffer(
                        stream,
                        sample_buffer,
                        match of_type {
                            0 => SCStreamOutputType::Screen,
                            1 => SCStreamOutputType::Audio,
                            _ => unreachable!("Should not be possible!"),
                        },
                    );
                }
            };
        }
        let mut decl =
            ClassDecl::new("SCStreamOutput", class!(NSObject)).ok_or("Could not register class")?;

        unsafe {
            let stream_output_method: extern "C" fn(
                &mut Object,
                Sel,
                *const c_void,
                *const c_void,
                i8,
            ) = stream_output;

            decl.add_method(
                sel!(stream:didOutputSampleBuffer:ofType:),
                stream_output_method,
            );
            decl.register();

            Ok(class!(SCStreamOutput))
        }
    }
    pub fn new(sc_stream_output_trait: impl SCStreamOutputTrait) -> SCStreamOutput {
        static REGISTER_CLASS: Once = Once::new();
        REGISTER_CLASS.call_once(|| {
            register_objc_class().expect("Should register SCStreamOutput class");
        });
        unsafe {
            let instance_ptr = runtime::class_createInstance(class!(SCStreamOutput), 0);
            OUTPUT_HANDLERS
                .write()
                .expect("could not obtain write lock for ERROR_DELEGATES")
                .insert(hash(instance_ptr), Box::new(sc_stream_output_trait));
            SCStreamOutput(instance_ptr)
        }
    }
}
pub use internal::SCStreamOutput;

#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SCStreamOutputType {
    Screen = 0,
    Audio = 1,
}
pub trait SCStreamOutputTrait: Send + Sync + 'static {
    fn did_output_sample_buffer(
        &self,
        stream: SCStream,
        sample_buffer: CMSampleBuffer,
        of_type: SCStreamOutputType,
    );
}

impl SCStreamOutput {
    pub fn new(stream_output: impl SCStreamOutputTrait) -> Self {
        internal::new(stream_output)
    }
}

#[cfg(test)]
mod tests {

    use core_foundation::base::TCFType;
    use objc::{msg_send, sel, sel_impl};

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
        },
    };

    use super::*;

    struct TestStreamOutput {
        of_type: SCStreamOutputType,
    }

    impl Default for TestStreamOutput {
        fn default() -> Self {
            Self {
                of_type: SCStreamOutputType::Screen,
            }
        }
    }

    impl SCStreamOutputTrait for TestStreamOutput {
        fn did_output_sample_buffer(
            &self,
            _stream: SCStream,
            _sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            assert_eq!(of_type, self.of_type);
        }
    }
    #[test]
    fn test_sc_stream_output_did_output_sample_buffer() {
        let handle1 = SCStreamOutput::new(TestStreamOutput {
            of_type: SCStreamOutputType::Audio,
        });
        let handle2 = SCStreamOutput::new(TestStreamOutput {
            of_type: SCStreamOutputType::Screen,
        });

        let config = SCStreamConfiguration::new();
        let display = SCShareableContent::get().unwrap().displays().remove(0);

        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let stream = {
            let filter = &filter;
            let configuration = &config;
            SCStream::new(filter, configuration)
        };
        let sample_buffer = CMSampleBuffer::new_empty();
        unsafe {
            let _: () = msg_send![handle1, stream: stream.as_CFTypeRef() didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: 1];
            let _: () = msg_send![handle2, stream: stream.as_CFTypeRef() didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: 0];
        }
    }
}
