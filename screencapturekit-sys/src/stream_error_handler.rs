use std::{ptr::addr_of, sync::Once};

use objc::{
    class,
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
    Message, *,
};
use objc_foundation::INSObject;
use objc_id::Id;

pub trait UnsafeSCStreamError {
    fn handle_error(&self);
}

#[repr(C)]
pub(crate) struct UnsafeSCStreamErrorHandler {}

unsafe impl Message for UnsafeSCStreamErrorHandler {}

impl INSObject for UnsafeSCStreamErrorHandler {
    fn class() -> &'static Class {
        static REGISTER_UNSAFE_SC_ERROR_HANDLER: Once = Once::new();
        REGISTER_UNSAFE_SC_ERROR_HANDLER.call_once(|| {
            let mut decl = ClassDecl::new("SCStreamErrorHandler", class!(NSObject)).unwrap();
            decl.add_ivar::<usize>("_trait");

            extern "C" fn stream_error(
                this: &mut Object,
                _cmd: Sel,
                _stream: *mut Object,
                _error: *mut Object,
            ) {
                unsafe {
                    let ptr = *this.get_ivar::<usize>("_trait");
                    let error_handler = addr_of!(ptr) as *mut Box<&dyn UnsafeSCStreamError>;
                    (*error_handler).handle_error();
                };
            }
            unsafe {
                let stream_error_method: extern "C" fn(&mut Object, Sel, *mut Object, *mut Object) =
                    stream_error;

                decl.add_method(sel!(stream:didStopWithError:), stream_error_method);
            }

            decl.register();
        });
        class!(SCStreamErrorHandler)
    }
}

impl UnsafeSCStreamErrorHandler {
    fn store_error_handler(&mut self, error_handler: &dyn UnsafeSCStreamError) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut Object);
            let trait_ptr = Box::into_raw(Box::new(error_handler));
            obj.set_ivar("_trait", trait_ptr as usize);
        }
    }
    pub fn init(error_handler: impl UnsafeSCStreamError) -> Id<Self> {
        let mut handle = Self::new();
        handle.store_error_handler(&error_handler);
        handle
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[repr(C)]
    struct TestHandler {}
    impl UnsafeSCStreamError for TestHandler {
        fn handle_error(&self) {
            eprintln!("ERROR!");
        }
    }

    #[test]
    fn test_sc_stream_error_handler() {
        let handle = UnsafeSCStreamErrorHandler::init(TestHandler {});
        unsafe {
            msg_send![handle, stream: ptr::null_mut::<Object>() didStopWithError: ptr::null_mut::<Object>()]
        }
    }
}
