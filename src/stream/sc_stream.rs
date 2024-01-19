use std::sync::mpsc::{channel, Receiver};

use block::{ConcreteBlock, RcBlock};
use objc::{
    runtime::{Class, Object},
    Message, *,
};

use crate::{
    stream_error_handler::{UnsafeSCStreamError, UnsafeSCStreamErrorHandler},
    stream_output_handler::{UnsafeSCStreamOutput, UnsafeSCStreamOutputHandler},
};

use super::{
    content_filter::UnsafeContentFilter, stream_configuration::UnsafeStreamConfigurationRef,
};
use dispatch::{Queue, QueueAttribute};

mod internal {

    #![allow(non_snake_case)]

    use std::{
        error::Error,
        ffi::c_void,
        ops::{Deref, DerefMut},
        ptr::addr_of,
        sync::Once,
    };

    use core_foundation::{
        base::*,
        declare_TCFType,
        error::{CFError, CFErrorRef},
        impl_TCFType,
    };
    use objc::{
        declare::ClassDecl,
        runtime::{Class, Object, Sel},
        *,
    };

    use crate::utils::objc::impl_deref;

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
    impl_deref!(SCStreamDelegate, SCStreamDelegateRef);

    fn register_objc_class() -> Result<&'static Class, Box<dyn Error>> {
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
            register_objc_class().expect("Should register SCStreamDelegate class");
        });
        let obj = unsafe { runtime::class_createInstance(class!(SCStreamDelegate), 0) };
        unsafe {
            let delegate: &dyn SCStreamDelegateTrait = &sc_stream_delegate;
            let trait_ptr = Box::into_raw(Box::new(delegate));
            let _: () = msg_send![obj, setTrait: trait_ptr];
            SCStreamDelegate::wrap_under_create_rule(SCStreamDelegateRef::from_void_ptr(
                obj as *mut c_void,
            ))
        }
    }
}
// impl UnsafeSCStream {
//     pub fn init(
//         filter: Id<UnsafeContentFilter>,
//         config: Id<UnsafeStreamConfigurationRef>,
//         error_handler: impl UnsafeSCStreamError,
//     ) -> Id<Self> {
//         let instance = UnsafeSCStream::new();
//
//         unsafe {
//             let _: () = msg_send![instance, initWithFilter: filter  configuration: config delegate: UnsafeSCStreamErrorHandler::init(error_handler)];
//         }
//         instance
//     }
//
//     pub fn start_capture(&self) -> Result<(), String> {
//         unsafe {
//             let (handler, rx) = Self::new_completion_handler();
//             let _: () = msg_send!(self, startCaptureWithCompletionHandler: handler);
//             rx.recv()
//                 .expect("Should receive a return from completion handler")
//         }
//     }
//     pub fn stop_capture(&self) -> Result<(), String> {
//         unsafe {
//             let (handler, rx) = Self::new_completion_handler();
//             let _: () = msg_send!(self, stopCaptureWithCompletionHandler: handler);
//             rx.recv()
//                 .expect("Should receive a return from completion handler")
//         }
//     }
//     pub fn add_stream_output(&self, handle: impl UnsafeSCStreamOutput, output_type: u8) {
//         let queue = Queue::create("fish.doom.screencapturekit", QueueAttribute::Concurrent);
//
//         let a = UnsafeSCStreamOutputHandler::init(handle);
//         unsafe {
//             let _: () = msg_send!(self, addStreamOutput: a type: output_type sampleHandlerQueue: queue error: NSObject::new());
//         }
//     }
// }
//
// impl Drop for UnsafeSCStream {
//     fn drop(&mut self) {
//         if let Err(err) = self.stop_capture() {
//             eprintln!("Cannot stop capture: {:?}", err)
//         }
//     }
// }
//
#[cfg(test)]
mod stream_test {
    use std::sync::mpsc::{sync_channel, SyncSender};

    use super::{UnsafeSCStream, UnsafeSCStreamError};
    use crate::{
        cm_sample_buffer_ref::CMSampleBufferRef,
        content_filter::{UnsafeContentFilter, UnsafeInitParams::Display},
        shareable_content::UnsafeSCShareableContent,
        stream_configuration::UnsafeStreamConfiguration,
        stream_output_handler::UnsafeSCStreamOutput,
    };
    struct ErrorHandler {}
    #[repr(C)]
    struct OutputHandler {
        tx: SyncSender<Id<CMSampleBufferRef>>,
    }
    impl Drop for OutputHandler {
        fn drop(&mut self) {
            println!("DROPPP");
        }
    }
    impl UnsafeSCStreamError for ErrorHandler {
        fn handle_error(&self) {
            eprintln!("ERROR!");
        }
    }
    impl UnsafeSCStreamOutput for OutputHandler {
        fn did_output_sample_buffer(&self, sample: Id<CMSampleBufferRef>, _of_type: u8) {
            self.tx.send(sample).unwrap();
        }
    }
    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_sc_stream() {
        let display = UnsafeSCShareableContent::get()
            .unwrap()
            .displays()
            .pop()
            .expect("could not get display");

        let filter = UnsafeContentFilter::init(Display(display));
        let config = UnsafeStreamConfiguration {
            width: 100,
            height: 100,
            ..Default::default()
        };
        let (tx, rx) = sync_channel(1);
        let stream = UnsafeSCStream::init(filter, config.into(), ErrorHandler {});
        let a = OutputHandler { tx };

        println!("ADDING OUTPUT");
        stream.add_stream_output(a, 0);
        println!("start capture");
        stream.start_capture().expect("start");
        println!("{:?}", rx.recv().unwrap());
        stream.stop_capture().expect("stop");
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_sc_stream_error_handling() {
        let display = UnsafeSCShareableContent::get()
            .unwrap()
            .displays()
            .pop()
            .expect("could not get display");

        let filter = UnsafeContentFilter::init(Display(display));
        let config = UnsafeStreamConfiguration {
            width: 100,
            height: 100,
            ..Default::default()
        };
        let stream = UnsafeSCStream::init(filter, config.into(), ErrorHandler {});

        println!("start capture");
        assert!(stream.start_capture().is_ok());
        assert!(stream.start_capture().is_err()); // already started error
        assert!(stream.stop_capture().is_ok());
        assert!(stream.stop_capture().is_err()); // already stopped error
    }
}
