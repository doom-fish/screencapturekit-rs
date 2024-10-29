use std::{ffi::c_void, sync::Once};

use objc::{
    class,
    declare::ClassDecl,
    runtime::{self, Object, Sel},
    sel, sel_impl,
};

use crate::{
    declare_trait_wrapper, stream::{
        sc_stream_output_trait::SCStreamOutputTrait, sc_stream_output_type::SCStreamOutputType,
    }, utils::objc::get_concrete_from_void
};

declare_trait_wrapper!(OutputTraitWrapper, SCStreamOutputTrait);


type StreamOutputMethod =
    extern "C" fn(&Object, Sel, *mut Object, *const c_void, SCStreamOutputType);
extern "C" fn stream_output(
    this: &Object,
    _cmd: Sel,
    _stream_ref: *mut Object,
    sample_buffer_ref: *const c_void,
    of_type: SCStreamOutputType,
) {
    let stream_output: &OutputTraitWrapper = unsafe { this.get_ivar("output_handler_wrapper") };
    let sample_buffer = unsafe { get_concrete_from_void(sample_buffer_ref) };
    stream_output.did_output_sample_buffer(sample_buffer, of_type);
}

fn register() {
    let mut decl =
        ClassDecl::new("StreamOutput", class!(NSObject)).expect("Could not register class");
    unsafe {
        let output_handler: StreamOutputMethod = stream_output;
        decl.add_ivar::<OutputTraitWrapper>("output_handler_wrapper");
        decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
        decl.register();
    }
}
pub type SCStreamOutput = *mut Object;
pub fn get_handler<T: SCStreamOutputTrait>(handler: T) -> SCStreamOutput {
    static REGISTER_ONCE: Once = Once::new();
    REGISTER_ONCE.call_once(register);

    unsafe {
        let sc_handler = runtime::class_createInstance(class!(StreamOutput), 0);
        let wrapper = OutputTraitWrapper::new(handler);
        (*sc_handler).set_ivar("output_handler_wrapper", wrapper);
        sc_handler
    }
}
