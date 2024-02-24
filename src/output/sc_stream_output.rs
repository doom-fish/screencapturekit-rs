use crate::{core_media::cm_sample_buffer::CMSampleBuffer, stream::sc_stream::SCStream};

mod internal {

    #![allow(non_snake_case)]

    use std::{
        collections::HashMap,
        error::Error,
        ffi::c_void,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Once, RwLock,
        },
    };

    use objc::{
        class,
        declare::ClassDecl,
        runtime::{Class, Object, Sel},
        sel, sel_impl,
    };
    use once_cell::sync::Lazy;

    use crate::{
        core_media::cm_sample_buffer::CMSampleBuffer,
        output::sc_stream_output::{SCStreamOutputTrait, SCStreamOutputType},
        stream::sc_stream::SCStream,
        utils::objc::get_concrete_from_void,
    };

    static OUTPUT_HANDLERS: Lazy<RwLock<HashMap<OpaqueIdentifier, Box<dyn SCStreamOutputTrait>>>> =
        Lazy::new(|| RwLock::new(HashMap::new()));

    #[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
    #[repr(transparent)]
    pub struct OpaqueIdentifier(usize);
    unsafe impl objc::Encode for OpaqueIdentifier {
        fn encode() -> objc::Encoding {
            usize::encode()
        }
    }
    impl OpaqueIdentifier {
        pub fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);

            Self(COUNTER.fetch_add(1, Ordering::Relaxed))
        }
    }

    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
        extern "C" fn dealloc(this: &mut Object, _cmd: Sel) {
            println!("Deallocating SCStreamOutput");
            unsafe {
                let handler_key: &OpaqueIdentifier = this.get_ivar("_output_handler");
                OUTPUT_HANDLERS
                    .write()
                    .expect("could not obtain write lock for OUTPUT_HANDLERS")
                    .remove(handler_key);
            }
        }
        extern "C" fn stream_output(
            this: &mut Object,
            _cmd: Sel,
            stream_ref: *const c_void,
            sample_buffer_ref: *const c_void,
            of_type: SCStreamOutputType,
        ) {
            unsafe {
                let handler_key: &OpaqueIdentifier = this.get_ivar("_output_handler");

                if let Some(output_handler) = OUTPUT_HANDLERS
                    .read()
                    .expect("could not obtain read lock for OUTPUT_HANDLERS")
                    .get(handler_key)
                {
                    let stream: SCStream = get_concrete_from_void(stream_ref);

                    let sample_buffer: CMSampleBuffer = get_concrete_from_void(sample_buffer_ref);
                    output_handler.did_output_sample_buffer(stream, sample_buffer, of_type);
                }
            };
        }
        let mut decl =
            ClassDecl::new("StreamOutput", class!(NSObject)).ok_or("Could not register class")?;
        decl.add_ivar::<OpaqueIdentifier>("_output_handler");

        unsafe {
            let stream_output_method: extern "C" fn(
                &mut Object,
                Sel,
                *const c_void,
                *const c_void,
                SCStreamOutputType,
            ) = stream_output;
            let dealloc_method: extern "C" fn(&mut Object, Sel) = dealloc;

            decl.add_method(sel!(dealloc), dealloc_method);

            decl.add_method(
                sel!(stream:didOutputSampleBuffer:ofType:),
                stream_output_method,
            );
            decl.register();

            Ok(class!(StreamOutput))
        }
    }
    pub fn store_output_trait(sc_stream_output_trait: impl SCStreamOutputTrait) -> *mut Object {
        static REGISTER_CLASS: Once = Once::new();
        REGISTER_CLASS.call_once(|| {
            register_objc_class().expect("Should register SCStreamOutput class");
        });
        unsafe {
            let opaque_identifier = OpaqueIdentifier::new();
            let obj = objc::runtime::class_createInstance(class!(StreamOutput), 0);

            (*obj).set_ivar("_output_handler", opaque_identifier);

            OUTPUT_HANDLERS
                .write()
                .expect("could not obtain write lock for ERROR_DELEGATES")
                .insert(opaque_identifier, Box::new(sc_stream_output_trait));
            obj
        }
    }
}
use internal::store_output_trait;


pub trait SCStreamOutputTrait: 'static + Send + Sync {
    fn did_output_sample_buffer(
        &self,
        stream: SCStream,
        sample_buffer: CMSampleBuffer,
        of_type: SCStreamOutputType,
    );
    fn store(self) -> *mut objc::runtime::Object
    where
        Self: Sized,
    {
        store_output_trait(self)
    }
}

#[cfg(test)]
mod tests {

    use std::mem;

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
    fn test_calling_object_dispose_on_handler() {
        let handler = TestStreamOutput::default();
        let obj = handler.store();
        unsafe { objc::runtime::objc_release(obj) };
    }

    #[test]
    fn test_sc_stream_output_did_output_sample_buffer() {
        // let handle1 = store_output_trait(TestStreamOutput {
        //     of_type: SCStreamOutputType::Audio,
        // });
        // let handle2 = store_output_trait(TestStreamOutput {
        //     of_type: SCStreamOutputType::Screen,
        // });
        //
        let config = SCStreamConfiguration::new();
        let display = SCShareableContent::get().unwrap().displays().remove(0);

        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let stream = {
            let filter = &filter;
            let configuration = &config;
            SCStream::new(filter, configuration)
        };
        // let sample_buffer = CMSampleBuffer::new_empty();
        let clo = stream.clone();
        mem::forget(clo)
        // unsafe {
        //     let _: () = msg_send![handle1, stream: clo didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: SCStreamOutputType::Audio];
        //     // let _: () = msg_send![handle2, stream: stream didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: SCStreamOutputType::Screen];
        // }
    }
}
