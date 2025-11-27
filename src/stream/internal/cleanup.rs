use objc::{
    runtime::{self, Object},
    Encoding,
};
use std::sync::atomic::{self, Ordering};

use super::{delegate::StreamDelegateTraitWrapper, output_handler::OutputTraitWrapper};

const MAX_HANDLERS: usize = 10;

#[repr(C)]
pub struct Cleanup {
    handlers: [*mut Object; MAX_HANDLERS],
    internal: *mut Object,
    rc: atomic::AtomicUsize,
}

impl Cleanup {
    pub const fn new(delegate: *mut Object) -> Self {
        Self {
            handlers: [std::ptr::null_mut(); MAX_HANDLERS],
            internal: delegate,
            rc: atomic::AtomicUsize::new(1),
        }
    }
    pub fn add_handler(&mut self, handler: *mut Object) {
        let index = self.handlers.iter().position(|&x| x.is_null()).unwrap();
        self.handlers[index] = handler;
    }

    fn iter(&self) -> impl Iterator<Item = &*mut Object> {
        self.handlers.iter().take_while(|&&x| !x.is_null())
    }

    pub fn drop_handlers(&mut self) {
        if self.rc.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        /*
         * See https://github.com/rust-lang/rust/blob/e1884a8e3c3e813aada8254edfa120e85bf5ffca/library/alloc/src/sync.rs#L1440-L1467
         * why the fence is needed.
         */
        atomic::fence(Ordering::Acquire);

        if !self.internal.is_null() {
            unsafe {
                (*self.internal)
                    .get_mut_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper")
                    .drop_trait();
                runtime::object_dispose(self.internal);
            };
        }
        self.iter().for_each(|handler| {
            unsafe {
                (**handler)
                    .get_mut_ivar::<OutputTraitWrapper>("output_handler_wrapper")
                    .drop_trait();
                runtime::object_dispose(*handler);
            };
        });
    }

    pub fn retain(&self) {
        let old_rc = self.rc.fetch_add(1, Ordering::Relaxed);
        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }
    }
}

unsafe impl objc::Encode for Cleanup {
    fn encode() -> objc::Encoding {
        unsafe { Encoding::from_str(format!("[^v{MAX_HANDLERS}]").as_str()) }
    }
}
