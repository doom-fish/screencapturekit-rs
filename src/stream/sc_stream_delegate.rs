use core_foundation::error::CFError;

pub use self::internal::SCStreamDelegate;

mod internal {

    #![allow(non_snake_case)]
    use once_cell::sync::Lazy;
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
        runtime::{self, Class, Object, Sel},
        sel, sel_impl,
    };

    use crate::utils::{self, hash::hash};

    use super::SCStreamDelegateTrait;

    static ERROR_DELEGATES: Lazy<
        RwLock<HashMap<u64, Box<dyn SCStreamDelegateTrait + Send + Sync>>>,
    > = Lazy::new(|| RwLock::new(HashMap::new()));

    pub struct SCStreamDelegate(pub *mut Object);

    impl Deref for SCStreamDelegate {
        type Target = Object;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }

    impl Drop for SCStreamDelegate {
        fn drop(&mut self) {
            unsafe {
                if let Some(delegate) = ERROR_DELEGATES
                    .write()
                    .expect("could not obtain read lock for ERROR_DELEGATES")
                    .remove(&hash(self.0))
                {
                    mem::drop(delegate);
                }
                core_foundation::base::CFRelease(self.0 as *const c_void);
            }
        }
    }

    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
        extern "C" fn stream_error(
            this: &mut Object,
            _cmd: Sel,
            _stream: *const c_void,
            error: *const c_void,
        ) {
            if let Some(stream_delegate) = ERROR_DELEGATES
                .read()
                .expect("could not obtain read lock for ERROR_DELEGATES")
                .get(&hash(this as *mut _))
            {
                stream_delegate
                    .did_stop_with_error(unsafe { utils::objc::get_concrete_from_void(error) });
            }
        }

        let mut decl = ClassDecl::new("SCStreamDelegate", class!(NSObject))
            .ok_or("Could not register class")?;

        unsafe {
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
            register_objc_class().expect("Should register SCStreamDelegate class");
        });
        unsafe {
            let instance_ptr = runtime::class_createInstance(class!(SCStreamDelegate), 0);
            ERROR_DELEGATES
                .write()
                .expect("could not obtain write lock for ERROR_DELEGATES")
                .insert(hash(instance_ptr), Box::new(sc_stream_delegate));
            SCStreamDelegate(instance_ptr)
        }
    }
}

pub trait SCStreamDelegateTrait: Send + Sync + 'static {
    fn did_stop_with_error(&self, _error: CFError) {}
}

impl SCStreamDelegate {
    pub fn new(sc_delegate_trait: impl SCStreamDelegateTrait) -> Self {
        internal::new(sc_delegate_trait)
    }
}

#[cfg(test)]
mod tests {

    use std::{
        ffi::c_void,
        ptr,
        sync::mpsc::{sync_channel, SyncSender},
        time::Duration,
    };

    use objc::{msg_send, sel, sel_impl};

    use crate::utils::error::internal::create_cf_error;

    use super::*;

    struct ErrorDelegate {
        tx: SyncSender<isize>,
    }
    impl SCStreamDelegateTrait for ErrorDelegate {
        fn did_stop_with_error(&self, error: CFError) {
            assert_eq!(error.domain(), "ERROR!");
            self.tx
                .send(error.code())
                .expect("could not use channel to send");
        }
    }

    #[test]
    fn test_sc_stream_delegate_did_stop_with_error() {
        let (tx, rx) = sync_channel(2);
        let handle = SCStreamDelegate::new(ErrorDelegate { tx });
        let err = create_cf_error("ERROR!", 4);
        let err2 = create_cf_error("ERROR!", 2);
        unsafe {
            let _: () = msg_send![handle, stream: ptr::null::<c_void>() didStopWithError: err];

            let _: () = msg_send![handle, stream: ptr::null::<c_void>() didStopWithError: err2];
        }
        let code = rx
            .recv_timeout(Duration::from_millis(10_000))
            .expect("could not receive from error delegate");

        assert_eq!(code, 4);
        let code = rx
            .recv_timeout(Duration::from_millis(10_000))
            .expect("could not receive from error delegate");

        assert_eq!(code, 2);
    }
}
