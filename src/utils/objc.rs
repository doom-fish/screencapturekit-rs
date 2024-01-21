macro_rules! impl_deref {
    ($tftype:ident) => {
        impl std::ops::Deref for $tftype {
            type Target = objc::runtime::Object;

            fn deref(&self) -> &objc::runtime::Object {
                unsafe { &*(self.as_CFTypeRef() as *mut objc::runtime::Object) }
            }
        }

        impl std::ops::DerefMut for $tftype {
            fn deref_mut(&mut self) -> &mut objc::runtime::Object {
                unsafe { &mut *(self.as_CFTypeRef() as *mut objc::runtime::Object) }
            }
        }
    };
}
pub(crate) use impl_deref;
