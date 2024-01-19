use core_foundation::error::CFError;

pub use self::internal::SCStreamDelegate;

mod internal {

    #![allow(non_snake_case)]

    use std::{error::Error, ffi::c_void, mem::size_of, ptr::addr_of, sync::Once};

    use core_foundation::{
        base::*,
        error::{CFError, CFErrorRef},
    };
    use objc::{
        declare::ClassDecl,
        runtime::{Class, Object, Sel},
        *,
    };

    use super::SCStreamDelegateTrait;

    #[repr(C)]
    pub struct SCStreamDelegate(*mut Object);
    fn new_class() -> Result<&'static Class, Box<dyn Error>> {
        let mut decl = ClassDecl::new("SCStreamDelegate", class!(NSObject))
            .ok_or("Could not register class")?;
        decl.add_ivar::<usize>("_trait");

        extern "C" fn trait_setter(this: &mut Object, _cmd: Sel, sc_stream_delegate_trait: usize) {
            unsafe {
                this.set_ivar::<usize>("_trait", sc_stream_delegate_trait);
            }
        }
        extern "C" fn trait_getter(this: &Object, _cmd: Sel) -> usize {
            unsafe { *this.get_ivar::<usize>("_trait") }
        }
        unsafe {
            let set_trait: extern "C" fn(&mut Object, Sel, usize) = trait_setter;
            let get_trait: extern "C" fn(&Object, Sel) -> usize = trait_getter;
            decl.add_method(sel!(setTrait:), set_trait);
            decl.add_method(sel!(trait), get_trait);
            extern "C" fn stream_error(
                this: &mut Object,
                _cmd: Sel,
                _stream: *const c_void,
                error: *const c_void,
            ) {
                unsafe {
                    let ptr = *this.get_ivar::<usize>("_trait");
                    let error_handler = addr_of!(ptr) as *mut Box<&dyn SCStreamDelegateTrait>;
                    let error = CFError::wrap_under_get_rule(CFErrorRef::from_void_ptr(error));
                    (*error_handler).did_stop_with_error(error);
                };
            }
            let stream_error_method: extern "C" fn(&mut Object, Sel, *const c_void, *const c_void) =
                stream_error;

            decl.add_method(sel!(stream:didStopWithError:), stream_error_method);
        }
        decl.register();

        Ok(class!(SCStreamDelegate))
    }
    pub fn new(sc_stream_delegate: impl SCStreamDelegateTrait) -> SCStreamDelegate {
        static REGISTER_CLASS: Once = Once::new();
        REGISTER_CLASS.call_once(|| {
            new_class().expect("Should register SCStreamDelegate class");
        });
        let obj = unsafe { runtime::class_createInstance(class!(SCStreamDelegate), 0) };
        unsafe {
            println!("{:?}", size_of::<&dyn SCStreamDelegateTrait>());
            println!("{:?}", size_of::<(usize, usize)>());
            let delegate: &dyn SCStreamDelegateTrait = &sc_stream_delegate;
            let trait_ptr = Box::into_raw(Box::new(delegate));
            let _: () = msg_send![obj, setTrait: trait_ptr];
        }
        SCStreamDelegate(obj)
    }
}
pub trait SCStreamDelegateTrait {
    fn did_stop_with_error(&self, error: CFError);
}

impl SCStreamDelegate {
    pub fn new(sc_delegate_trait: impl SCStreamDelegateTrait) -> Self {
        internal::new(sc_delegate_trait)
    }
}

// impl INSObject for UnsafeSCStreamErrorHandler {
//     fn class() -> &'static Class {
//         static REGISTER_UNSAFE_SC_ERROR_HANDLER: Once = Once::new();
//         REGISTER_UNSAFE_SC_ERROR_HANDLER.call_once(|| {
//             let mut decl = ClassDecl::new("SCStreamErrorHandler", class!(NSObject)).unwrap();
//             decl.add_ivar::<usize>("_trait");
//
//             extern "C" fn stream_error(
//                 this: &mut Object,
//                 _cmd: Sel,
//                 _stream: *mut Object,
//                 _error: *mut Object,
//             ) {
//                 unsafe {
//                     let ptr = *this.get_ivar::<usize>("_trait");
//                     let error_handler = addr_of!(ptr) as *mut Box<&dyn UnsafeSCStreamError>;
//                     (*error_handler).handle_error();
//                 };
//             }
//             unsafe {
//                 let stream_error_method: extern "C" fn(&mut Object, Sel, *mut Object, *mut Object) =
//                     stream_error;
//
//                 decl.add_method(sel!(stream:didStopWithError:), stream_error_method);
//             }
//
//             decl.register();
//         });
//         class!(SCStreamErrorHandler)
//     }
// }
//
// impl UnsafeSCStreamErrorHandler {
//     fn store_error_handler(&mut self, error_handler: &dyn UnsafeSCStreamError) {
//         unsafe {
//             let obj = &mut *(self as *mut _ as *mut Object);
//             let trait_ptr = Box::into_raw(Box::new(error_handler));
//             obj.set_ivar("_trait", trait_ptr as usize);
//         }
//     }
//     pub fn init(error_handler: impl UnsafeSCStreamError) -> Id<Self> {
//         let mut handle = Self::new();
//         handle.store_error_handler(&error_handler);
//         handle
//     }
// }
//
#[cfg(test)]
mod tests {

    use super::*;
    use objc::*;
    #[repr(C)]
    struct TestHandler {}
    impl SCStreamDelegateTrait for TestHandler {}

    #[test]
    fn test_sc_stream_error_handler() {
        let handle = SCStreamDelegate::new(TestHandler {});
    }
}
