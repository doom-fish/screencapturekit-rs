use std::{ffi::c_void, sync::Once};

use core_foundation::error::CFError;
use objc::{
    class,
    declare::ClassDecl,
    runtime::{self, Object, Sel},
    sel, sel_impl,
};

use crate::{
    declare_trait_wrapper, stream::delegate_trait::SCStreamDelegateTrait,
    utils::objc::get_concrete_from_void,
};

use super::stream::SCStream;

declare_trait_wrapper!(StreamDelegateTraitWrapper, SCStreamDelegateTrait);

type DidStopWithErrorMethod = extern "C" fn(&Object, Sel, *const c_void, *const c_void);
extern "C" fn did_stop_with_error(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
    error: *const c_void,
) {
    let handler = unsafe { this.get_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    let error: CFError = unsafe { get_concrete_from_void(error) };
    handler.did_stop_with_error(stream, error);
}

type OutputVideoEffectDidStartForStreamMethod = extern "C" fn(&Object, Sel, *const c_void);
extern "C" fn output_video_effect_did_start_for_stream(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
) {
    let handler = unsafe { this.get_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    handler.output_video_effect_did_start_for_stream(stream);
}
type OutputVideoEffectDidStopForStreamMethod = extern "C" fn(&Object, Sel, *const c_void);
extern "C" fn output_video_effect_did_stop_for_stream(
    this: &Object,
    _cmd: Sel,
    stream_ref: *const c_void,
) {
    let handler = unsafe { this.get_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper") };
    let stream = unsafe { get_concrete_from_void::<SCStream>(stream_ref) };
    handler.output_video_effect_did_stop_for_stream(stream);
}

fn register() {
    let mut decl =
        ClassDecl::new("StreamDelegate", class!(NSObject)).expect("Could not register class");
    unsafe {
        decl.add_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper");
        decl.add_method(
            sel!(stream:didStopWithError:),
            did_stop_with_error as DidStopWithErrorMethod,
        );
        decl.add_method(
            sel!(outputVideoEffectDidStartForStream:),
            output_video_effect_did_start_for_stream as OutputVideoEffectDidStartForStreamMethod,
        );
        decl.add_method(
            sel!(outputVideoEffectDidStopForStream:),
            output_video_effect_did_stop_for_stream as OutputVideoEffectDidStopForStreamMethod,
        );
        decl.register();
    }
}

pub fn get_handler<'a>(handler: impl SCStreamDelegateTrait + 'a) -> *mut Object {
    static REGISTER_ONCE: Once = Once::new();
    REGISTER_ONCE.call_once(register);

    unsafe {
        let error_delegate = runtime::class_createInstance(class!(StreamDelegate), 0);
        let wrapper = StreamDelegateTraitWrapper::new(handler);
        (*error_delegate).set_ivar("stream_delegate_wrapper", wrapper);
        error_delegate
    }
}
