use crate::{core_media::cm_sample_buffer::CMSampleBuffer, stream::sc_stream::SCStream};

mod internal {

    #![allow(non_snake_case)]

    use std::{error::Error, ffi::c_void, ptr::addr_of, sync::Once};

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };
    use objc::{
        class,
        declare::ClassDecl,
        msg_send, runtime,
        runtime::{Class, Object, Sel},
        sel, sel_impl,
    };

    use crate::{
        core_media::cm_sample_buffer::CMSampleBuffer,
        output::sc_stream_output::{SCStreamOutputTrait, SCStreamOutputType},
        stream::sc_stream::SCStream,
        utils::objc::{create_concrete_from_void, get_concrete_from_void},
    };

    #[repr(C)]
    pub struct __SCStreamOutputRef(c_void);
    extern "C" {
        pub fn SCStreamOutputGetTypeID() -> CFTypeID;
    }

    pub type SCStreamOutputRef = *mut __SCStreamOutputRef;

    declare_TCFType! {SCStreamOutput, SCStreamOutputRef}
    impl_TCFType!(SCStreamOutput, SCStreamOutputRef, SCStreamOutputGetTypeID);

    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
        extern "C" fn trait_setter(this: &mut Object, _cmd: Sel, sc_stream_delegate_trait: usize) {
            unsafe {
                this.set_ivar::<usize>("_output_trait", sc_stream_delegate_trait);
            }
        }
        extern "C" fn trait_getter(this: &Object, _cmd: Sel) -> usize {
            unsafe { *this.get_ivar::<usize>("_output_trait") }
        }
        extern "C" fn stream_output(
            this: &mut Object,
            _cmd: Sel,
            stream_ref: *const c_void,
            sample_buffer_ref: *const c_void,
            of_type: i8,
        ) {
            unsafe {
                let ptr = *this.get_ivar::<usize>("_output_trait");
                let stream: SCStream = get_concrete_from_void(stream_ref);
                let sample_buffer: CMSampleBuffer = get_concrete_from_void(sample_buffer_ref);
                let stream_output = addr_of!(ptr) as *mut Box<&dyn SCStreamOutputTrait>;
                (*stream_output).did_output_sample_buffer(
                    stream,
                    sample_buffer,
                    match of_type {
                        0 => SCStreamOutputType::Screen,
                        1 => SCStreamOutputType::Audio,
                        _ => unreachable!("Should not be possible!"),
                    },
                );
            };
        }
        let mut decl =
            ClassDecl::new("SCStreamOutput", class!(NSObject)).ok_or("Could not register class")?;
        decl.add_ivar::<usize>("_output_trait");

        unsafe {
            let set_trait: extern "C" fn(&mut Object, Sel, usize) = trait_setter;
            let get_trait: extern "C" fn(&Object, Sel) -> usize = trait_getter;
            decl.add_method(sel!(setTrait:), set_trait);
            decl.add_method(sel!(getTrait), get_trait);

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
    pub fn new(sc_stream_output_trait: &impl SCStreamOutputTrait) -> SCStreamOutput {
        static REGISTER_CLASS: Once = Once::new();
        REGISTER_CLASS.call_once(|| {
            register_objc_class().expect("Should register SCStreamOutput class");
        });
        let stream_output: &dyn SCStreamOutputTrait = sc_stream_output_trait;
        unsafe {
            let obj: *mut Object = runtime::class_createInstance(class!(SCStreamOutput), 0);
            let trait_ptr = Box::into_raw(Box::new(stream_output));
            let _: () = msg_send![obj, setTrait: trait_ptr];
            create_concrete_from_void(obj as *const c_void)
        }
    }
}
pub use internal::{SCStreamOutput, SCStreamOutputRef};

#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SCStreamOutputType {
    Screen,
    Audio,
}
pub trait SCStreamOutputTrait {
    fn did_output_sample_buffer(
        &self,
        stream: SCStream,
        sample_buffer: CMSampleBuffer,
        of_type: SCStreamOutputType,
    );
}

impl SCStreamOutput {
    pub fn new(stream_output: &impl SCStreamOutputTrait) -> Self {
        internal::new(stream_output)
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use core_foundation::base::TCFType;
    use objc::{msg_send, sel, sel_impl};

    use crate::{
        shareable_content::sc_shareable_content::SCShareableContent,
        stream::{
            sc_content_filter::SCContentFilter, sc_stream_configuration::SCStreamConfiguration,
            sc_stream_delegate::SCStreamDelegateTrait,
        },
        utils::objc::MessageForTFType,
    };

    use super::*;
    #[derive(Clone)]
    struct StreamOutput {
        of_type: SCStreamOutputType,
    }

    impl Default for StreamOutput {
        fn default() -> Self {
            Self {
                of_type: SCStreamOutputType::Screen,
            }
        }
    }

    impl SCStreamOutputTrait for StreamOutput {
        fn did_output_sample_buffer(
            &self,
            _stream: SCStream,
            _sample_buffer: CMSampleBuffer,
            of_type: SCStreamOutputType,
        ) {
            assert_eq!(of_type, self.of_type);
        }
    }
    impl SCStreamDelegateTrait for StreamOutput {}
    #[test]
    fn test_sc_stream_delegate_did_stop_with_error() -> Result<(), Box<dyn Error>> {
        let mut output = StreamOutput::default();
        let handle = SCStreamOutput::new(&output);
        let config = SCStreamConfiguration::new();
        let display = SCShareableContent::get()?.displays().remove(0);

        let filter = SCContentFilter::new().with_with_display_excluding_windows(&display, &[]);
        let stream = SCStream::new(&filter, &config, output.clone());
        let sample_buffer = CMSampleBuffer::new_empty();
        unsafe {
            output.of_type = SCStreamOutputType::Audio;
            let _: () = msg_send![handle.as_sendable(), stream: stream.as_CFTypeRef() didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: 1];
            output.of_type = SCStreamOutputType::Screen;
            let _: () = msg_send![handle.as_sendable(), stream: stream.as_CFTypeRef() didOutputSampleBuffer: sample_buffer.as_CFTypeRef() ofType: 0];
        }

        Ok(())
    }
}
