use core_foundation::base::TCFType;
use objc::runtime::Object;

pub trait SendableObjc {
    fn to_sendable(&self) -> *mut Object;
}

impl<T: TCFType> SendableObjc for T {
    fn to_sendable(&self) -> *mut Object {
        self.as_CFTypeRef() as *mut Object
    }
}
