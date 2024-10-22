use std::{ffi::c_void, sync::mpsc::Receiver};

use block2::StackBlock;
use core_foundation::{base::TCFType, error::CFError};

use super::objc::get_concrete_from_void;

pub struct CompletionHandler<'a, ResType, TArgs, TFN>(
    pub StackBlock<'a, TArgs, (), TFN>,
    pub Receiver<Result<ResType, CFError>>,
);

/// .
///
/// # Panics
///
/// Panics if .
pub fn new_completion_handler<'a, ConcreteCFType>() -> CompletionHandler<
    'a,
    ConcreteCFType,
    (*const c_void, *const c_void),
    impl Fn(*const c_void, *const c_void),
>
where
    ConcreteCFType: TCFType + 'a,
{
    let (sender, receiver) = std::sync::mpsc::sync_channel(0);
    // Handle memory manually as the block will leak any closed variables
    let sender_ptr = Box::into_raw(Box::new(sender));
    let handler = StackBlock::new(move |ret: *const c_void, error: *const c_void| {
        let sender = unsafe { Box::from_raw(sender_ptr) };
        if error.is_null() {
            let wrapped: ConcreteCFType = unsafe { get_concrete_from_void(ret) };
            sender.send(Ok(wrapped)).expect("should work");
        } else {
            let wrapped_error: CFError = unsafe { get_concrete_from_void(error) };
            sender.send(Err(wrapped_error)).expect("should work");
        }
    });
    CompletionHandler(handler, receiver)
}

/// .
///
/// # Panics
///
/// Panics if .
pub fn new_void_completion_handler<'a>(
) -> CompletionHandler<'a, (), (*const c_void,), impl Fn(*const c_void)> {
    // Handle memory manually as the block will leak any closed variables
    let (sender, receiver) = std::sync::mpsc::sync_channel(0);
    let c_ptr = Box::into_raw(Box::new(sender));
    let handler = StackBlock::new(move |error: *const c_void| {
        let sender = unsafe { Box::from_raw(c_ptr) };
        if error.is_null() {
            sender.send(Ok(())).expect("should work");
        } else {
            let wrapped_error: CFError = unsafe { get_concrete_from_void(error) };
            sender.send(Err(wrapped_error)).expect("should work");
        }
    });
    CompletionHandler(handler, receiver)
}
