use std::{
    os::raw::c_void,
    sync::mpsc::{channel, Receiver},
};

use block::{ConcreteBlock, RcBlock};
use core_foundation::{
    base::TCFType,
    error::{CFError, CFErrorRef},
};

pub struct CompletionHandler<Concrete: TCFType>(
    pub RcBlock<(Concrete::Ref, CFErrorRef), ()>,
    pub Receiver<Result<Concrete, CFError>>,
);

pub fn new_completion_handler<ConcreteCFType>() -> CompletionHandler<ConcreteCFType>
where
    ConcreteCFType: TCFType + 'static,
{
    let (tx, rx) = channel();
    let handler = ConcreteBlock::new(move |ret: ConcreteCFType::Ref, error: CFErrorRef| {
        if error.is_null() {
            let wrapped = unsafe { ConcreteCFType::wrap_under_get_rule(ret) };
            tx.send(Ok(wrapped)).expect("should work");
        } else {
            let wrapped_error = unsafe { CFError::wrap_under_get_rule(error) };
            tx.send(Err(wrapped_error)).expect("should work");
        }
    });
    CompletionHandler(handler.copy(), rx)
}
pub struct VoidCompletionHandler(
    pub RcBlock<(*mut c_void, CFErrorRef), ()>,
    pub Receiver<Result<(), CFError>>,
);

pub fn new_void_completion_handler() -> VoidCompletionHandler {
    let (tx, rx) = channel();
    let handler = ConcreteBlock::new(move |_: *mut c_void, error: CFErrorRef| {
        if error.is_null() {
            tx.send(Ok(())).expect("should work");
        } else {
            let wrapped_error = unsafe { CFError::wrap_under_get_rule(error) };
            tx.send(Err(wrapped_error)).expect("should work");
        }
    });
    VoidCompletionHandler(handler.copy(), rx)
}
