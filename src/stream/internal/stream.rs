use crate::{
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait, internal::delegate,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType,
    },
    utils::{
        block::{new_void_completion_handler, CompletionHandler},
        error::create_sc_error,
    },
};
use core_foundation::{base, error::CFError};
use core_foundation::{
    base::{CFTypeID, TCFType, TCFTypeRef},
    impl_TCFType,
};
use dispatch::ffi::{dispatch_get_global_queue, DISPATCH_QUEUE_PRIORITY_DEFAULT};
use std::{ffi::c_void, ptr};

use objc::{class, declare::ClassDecl, msg_send, runtime::Object, sel, sel_impl};

use super::{
    cleanup::Cleanup,
    output_handler::{self, SCStreamOutput},
};

#[repr(C)]
pub struct __SCStreamRef(c_void);

extern "C" {
    pub fn SCStreamGetTypeID() -> CFTypeID;
}

pub type SCStreamRef = *mut __SCStreamRef;
#[allow(clippy::module_name_repetitions)]
pub struct SCStream(SCStreamRef);

impl_TCFType!(SCStream, SCStreamRef, SCStreamGetTypeID);

unsafe impl Send for SCStream {}

impl Drop for SCStream {
    fn drop(&mut self) {
        unsafe {
            (*self.as_concrete_TypeRef().cast::<Object>())
                .get_mut_ivar::<Cleanup>("cleanup")
                .drop_handlers();

            base::CFRelease(self.as_CFTypeRef());
        }
    }
}
fn register() {
    let mut decl =
        ClassDecl::new("SCStreamWithHandlers", class!(SCStream)).expect("Could not register class");
    decl.add_ivar::<Cleanup>("cleanup");
    decl.register();
}

impl SCStream {
    pub fn store_cleanup(&self, handler: *mut Object) {
        unsafe {
            let obj = self.as_concrete_TypeRef().cast::<Object>();
            (*obj)
                .get_mut_ivar::<Cleanup>("cleanup")
                .add_handler(handler);
        };
    }
    pub fn internal_init_with_filter(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Self {
        struct NoopDelegate;
        impl SCStreamDelegateTrait for NoopDelegate {}
        Self::internal_init_with_filter_and_delegate(filter, configuration, None::<NoopDelegate>)
    }
    pub fn internal_init_with_filter_and_delegate<T: SCStreamDelegateTrait>(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: Option<T>,
    ) -> Self {
        static REGISTER_ONCE: std::sync::Once = std::sync::Once::new();
        REGISTER_ONCE.call_once(register);
        unsafe {
            let delegate = delegate.map_or(ptr::null_mut(), delegate::get_handler);
            let inner: *mut Object = msg_send![class!(SCStreamWithHandlers), alloc];
            (*inner).set_ivar("cleanup", Cleanup::new(delegate));
            let inner: SCStreamRef = msg_send![inner, initWithFilter: filter.clone().as_CFTypeRef()  configuration: configuration.clone().as_CFTypeRef() delegate: delegate];
            Self::wrap_under_create_rule(inner)
        }
    }

    pub fn internal_remove_output_handler(
        &mut self,
        handler: SCStreamOutput,
        of_type: SCStreamOutputType,
    ) -> bool {
        let error: *mut Object = ptr::null_mut();
        unsafe {
            msg_send![self.as_CFTypeRef().cast::<Object>(), removeStreamOutput: handler type: of_type error: error]
        }
    }

    pub fn internal_add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait,
        of_type: SCStreamOutputType,
    ) -> Option<SCStreamOutput> {
        unsafe {
            let error: *mut Object = ptr::null_mut();
            let handler = output_handler::get_handler(handler);
            let queue = dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0);
            let success: bool = match of_type {
                SCStreamOutputType::Screen => {
                    msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Screen sampleHandlerQueue: queue error: error]
                }
                SCStreamOutputType::Audio => {
                    msg_send![self.as_CFTypeRef().cast::<Object>(), addStreamOutput: handler type: SCStreamOutputType::Audio sampleHandlerQueue: queue error: error]
                }
            };

            self.store_cleanup(handler);

            if success {
                Some(handler)
            } else {
                None
            }
        }
    }
    /// Returns the internal start capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn internal_start_capture(&self) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_void_completion_handler();
            let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), startCaptureWithCompletionHandler: handler];

            rx.recv()
                .map_err(|_| create_sc_error("Could not receive from completion handler"))?
        }
    }
    /// Returns the internal stop capture of this [`SCStream`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn internal_stop_capture(&self) -> Result<(), CFError> {
        unsafe {
            let CompletionHandler(handler, rx) = new_void_completion_handler();

            let _: () = msg_send![self.as_CFTypeRef().cast::<Object>(), stopCaptureWithCompletionHandler: handler];

            rx.recv()
                .map_err(|_| create_sc_error("Could not receive from completion handler"))?
        }
    }

    pub fn internal_clone(&self) -> Self {
        unsafe {
            (*self.as_concrete_TypeRef().cast::<Object>())
                .get_mut_ivar::<Cleanup>("cleanup")
                .retain();
        }
        Clone::clone(&self)
    }
}

pub unsafe fn get_concrete_stream_from_void(void_ptr: *const c_void) -> SCStream {
    let stream = SCStream::wrap_under_get_rule(SCStreamRef::from_void_ptr(void_ptr));
    (*stream.as_concrete_TypeRef().cast::<Object>())
        .get_mut_ivar::<Cleanup>("cleanup")
        .retain();
    stream
}

