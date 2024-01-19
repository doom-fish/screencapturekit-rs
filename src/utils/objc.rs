use core_foundation::base::{TCFType, TCFTypeRef};
use objc::runtime::Object;

pub trait SendableObjc {
    fn to_sendable(&self) -> *mut Object;
}

impl<T: TCFType> SendableObjc for T {
    fn to_sendable(&self) -> *mut Object {
        self.as_CFTypeRef() as *mut Object
    }
}
pub trait SendableObjcRef {
    fn to_sendable(&self) -> *mut Object;
}

impl<T: TCFTypeRef> SendableObjcRef for T {
    fn to_sendable(&self) -> *mut Object {
        self as *const _ as *mut Object
    }
}

macro_rules! impl_deref {
    ($tftype:ident) => {
        impl Deref for $tftype {
            type Target = Object;

            fn deref(&self) -> &Object {
                unsafe { &*(self.as_CFTypeRef() as *mut Object) }
            }
        }

        impl DerefMut for $tftype {
            fn deref_mut(&mut self) -> &mut Object {
                unsafe { &mut *(self.as_CFTypeRef() as *mut Object) }
            }
        }
    };
}
pub(crate) use impl_deref;
