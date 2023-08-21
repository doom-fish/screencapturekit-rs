use screencapturekit_sys::stream_error_handler::UnsafeSCStreamError;

pub trait StreamErrorHandler {
    fn on_error(&self);
}

pub(crate) struct StreamErrorHandlerWrapper<T: StreamErrorHandler>(T);

impl<T: StreamErrorHandler> StreamErrorHandlerWrapper<T> {
    pub fn new(error_handler: T) -> Self {
        StreamErrorHandlerWrapper(error_handler)
    }
}

impl<T: StreamErrorHandler> UnsafeSCStreamError for StreamErrorHandlerWrapper<T> {
    fn handle_error(&self) {
        self.0.on_error();
    }
}
