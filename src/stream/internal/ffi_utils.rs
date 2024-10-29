#[macro_export]
macro_rules! declare_trait_wrapper {
    ($name: ident, $t:ident) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name<'a>(pub *mut Box<dyn $t + 'a>);

        impl<'a> $name<'a> {
            pub fn new(handler: impl $t + 'a) -> Self {
                Self(Box::into_raw(Box::new(Box::new(handler))))
            }
            #[allow(dead_code)]
            pub fn drop_trait(&self) {
                unsafe {
                    let _ = Box::from_raw(self.0);
                }
            }
        }
        impl<'a> std::ops::Deref for $name<'a> {
            type Target = Box<dyn $t>;
            fn deref(&self) -> &'a Self::Target {
                unsafe { &*self.0.cast() }
            }
        }

        impl<'a> std::ops::DerefMut for $name<'a> {
            fn deref_mut(&mut self) -> &'a mut Self::Target {
                unsafe { &mut *self.0.cast() }
            }
        }

        unsafe impl objc::Encode for $name<'_> {
            fn encode() -> objc::Encoding {
                unsafe { objc::Encoding::from_str("r^v") }
            }
        }
    };
}
