use std::{ffi::c_void, sync::Once};

use objc::{
    class,
    declare::ClassDecl,
    runtime::{self, Object, Sel},
    sel, sel_impl,
};

use crate::{declare_trait_wrapper, stream::sc_stream_delegate_trait::SCStreamDelegateTrait};

declare_trait_wrapper!(StreamDelegateTraitWrapper, SCStreamDelegateTrait);

type DidStopWithErrorMethod = extern "C" fn(&Object, Sel, *const c_void, *const c_void);
const extern "C" fn did_stop_with_error(
    _this: &Object,
    _cmd: Sel,
    _stream_ref: *const c_void,
    _error: *const c_void,
) {
    todo!();
}

fn register() {
    let mut decl =
        ClassDecl::new("StreamDelegate", class!(NSObject)).expect("Could not register class");
    unsafe {
        let output_handler: DidStopWithErrorMethod = did_stop_with_error;
        decl.add_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper");
        decl.add_method(sel!(stream:didOutputSampleBuffer:ofType:), output_handler);
        decl.register();
    }
}

pub fn get_handler<'a>(handler: impl SCStreamDelegateTrait + 'a) -> *mut Object {
    static REGISTER_ONCE: Once = Once::new();
    REGISTER_ONCE.call_once(register);

    unsafe {
        let sc_handler = runtime::class_createInstance(class!(StreamOutput), 0);
        let wrapper = StreamDelegateTraitWrapper::new(handler);
        (*sc_handler).set_ivar("stream_delegate_wrapper", wrapper);
        sc_handler
    }
}
