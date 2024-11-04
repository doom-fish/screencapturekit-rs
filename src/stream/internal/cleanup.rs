use objc::{
    runtime::{self, Object},
    Encoding,
};

use super::{output_handler::OutputTraitWrapper, delegate::StreamDelegateTraitWrapper};

const MAX_HANDLERS: usize = 10;

#[repr(C)]
pub struct Cleanup([*mut Object; MAX_HANDLERS], *mut Object);

impl Cleanup {
    pub const fn new(delegate: *mut Object) -> Self {
        Self([std::ptr::null_mut(); MAX_HANDLERS], delegate)
    }
    pub fn add_handler(&mut self, handler: *mut Object) {
        let index = self.0.iter().position(|&x| x.is_null()).unwrap();
        self.0[index] = handler;
    }

    fn iter(&self) -> impl Iterator<Item = &*mut Object> {
        self.0.iter().take_while(|&&x| !x.is_null())
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn drop_handlers(&mut self) {
        if !self.1.is_null() {
            unsafe {
                (*self.1)
                    .get_mut_ivar::<StreamDelegateTraitWrapper>("stream_delegate_wrapper")
                    .drop_trait();
                runtime::object_dispose(self.1);
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
}

unsafe impl objc::Encode for Cleanup {
    fn encode() -> objc::Encoding {
        unsafe { Encoding::from_str(format!("[^v{MAX_HANDLERS}]").as_str()) }
    }
}
