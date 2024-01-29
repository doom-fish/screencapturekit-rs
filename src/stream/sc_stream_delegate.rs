use core_foundation::error::CFError;

pub use self::internal::SCStreamDelegate;

mod internal {

    #![allow(non_snake_case)]

    use std::{error::Error, ffi::c_void, ptr::addr_of, sync::Once};

    use core_foundation::{
        base::{CFTypeID, TCFType, TCFTypeRef},
        declare_TCFType,
        error::{CFError, CFErrorRef},
        impl_TCFType,
    };
    use objc::{
        class,
        declare::ClassDecl,
        msg_send,
        runtime::{self, Class, Object, Sel},
        sel, sel_impl,
    };

    use super::SCStreamDelegateTrait;
    #[repr(C)]
    pub struct __SCStreamDelegateRef(c_void);
    extern "C" {
        pub fn SCStreamDelegateGetTypeID() -> CFTypeID;
    }

    pub type SCStreamDelegateRef = *mut __SCStreamDelegateRef;

    declare_TCFType! {SCStreamDelegate, SCStreamDelegateRef}
    impl_TCFType!(
        SCStreamDelegate,
        SCStreamDelegateRef,
        SCStreamDelegateGetTypeID
    );

    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
        extern "C" fn trait_setter(this: &mut Object, _cmd: Sel, sc_stream_delegate_trait: usize) {
            unsafe {
                this.set_ivar::<usize>("_trait", sc_stream_delegate_trait);
            }
        }
        extern "C" fn trait_getter(this: &Object, _cmd: Sel) -> usize {
            unsafe { *this.get_ivar::<usize>("_trait") }
        }
        extern "C" fn stream_error(
            this: &mut Object,
            _cmd: Sel,
            _stream: *const c_void,
            error: *const c_void,
        ) {
            unsafe {
                let ptr = *this.get_ivar::<usize>("_trait");
                let stream_delegate = addr_of!(ptr) as *mut Box<&dyn SCStreamDelegateTrait>;
                let error = CFError::wrap_under_get_rule(CFErrorRef::from_void_ptr(error));
                (*stream_delegate).did_stop_with_error(error);
            };
        }

        let mut decl = ClassDecl::new("SCStreamDelegate", class!(NSObject))
            .ok_or("Could not register class")?;
        decl.add_ivar::<usize>("_trait");

        unsafe {
            let set_trait: extern "C" fn(&mut Object, Sel, usize) = trait_setter;
            let get_trait: extern "C" fn(&Object, Sel) -> usize = trait_getter;
            decl.add_method(sel!(setTrait:), set_trait);
            decl.add_method(sel!(trait), get_trait);
            let stream_error_method: extern "C" fn(&mut Object, Sel, *const c_void, *const c_void) =
                stream_error;

            decl.add_method(sel!(stream:didStopWithError:), stream_error_method);
        }
        decl.register();

        Ok(class!(SCStreamDelegate))
    }
    pub fn new(sc_stream_delegate: &impl SCStreamDelegateTrait) -> SCStreamDelegate {
        static REGISTER_CLASS: Once = Once::new();

        REGISTER_CLASS.call_once(|| {
            register_objc_class().expect("Should register SCStreamDelegate class");
        });
        let delegate: &dyn SCStreamDelegateTrait = sc_stream_delegate;
        let obj = unsafe { runtime::class_createInstance(class!(SCStreamDelegate), 0) };
        unsafe {
            let trait_ptr = Box::into_raw(Box::new(delegate));
            let _: () = msg_send![obj, setTrait: trait_ptr];
            SCStreamDelegate::wrap_under_create_rule(SCStreamDelegateRef::from_void_ptr(
                obj as *const c_void,
            ))
        }
    }
}
pub trait SCStreamDelegateTrait {
    fn did_stop_with_error(&self, _error: CFError) {}
}

impl SCStreamDelegate {
    pub fn new(sc_delegate_trait: &impl SCStreamDelegateTrait) -> Self {
        internal::new(sc_delegate_trait)
    }
}

// #[cfg(test)]
// mod tests {
//
//     use crate::utils::error::internal::create_cf_error;
//
//     use super::*;
//     struct ErrorDelegate;
//     impl SCStreamDelegateTrait for ErrorDelegate {
//         fn did_stop_with_error(&self, error: CFError) {
//             assert_eq!(error.code(), 4);
//             assert_eq!(error.domain(), "NSMachErrorDomain");
//         }
//     }
//
//     #[test]
//     fn test_sc_stream_delegate_did_stop_with_error() {
//         let handle = SCStreamDelegate::new(&ErrorDelegate);
//         let err = create_cf_error("ERROR!", 4);
//     }
// }
