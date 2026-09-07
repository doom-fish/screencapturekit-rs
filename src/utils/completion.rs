//! Completion handles for Swift bridge callbacks.
//!
//! Synchronous waits are bounded by default. Callback contexts are opaque
//! monotonic tokens backed by a process registry, not addresses. A callback
//! that arrives after timeout/cancellation, or fires more than once, therefore
//! finds no registry entry and returns without touching freed memory.

use std::any::Any;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, PoisonError};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use crate::utils::panic_safe::catch_user_panic;

/// Default bound for synchronous waits and async completion futures.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Overrides [`DEFAULT_TIMEOUT`] in whole seconds; `0` disables the bound.
pub const TIMEOUT_ENV_VAR: &str = "SCREENCAPTUREKIT_COMPLETION_TIMEOUT_SECS";

/// Stable prefix for timeout errors.
pub const TIMEOUT_MESSAGE_PREFIX: &str = "screencapturekit: completion callback did not fire";

/// Whether an error came from a bounded wait expiring.
#[must_use]
pub fn is_timeout_error(message: &str) -> bool {
    message.starts_with(TIMEOUT_MESSAGE_PREFIX)
}

/// Effective process-wide wait bound.
#[must_use]
pub fn default_timeout() -> Option<Duration> {
    static TIMEOUT: OnceLock<Option<Duration>> = OnceLock::new();
    *TIMEOUT.get_or_init(|| {
        std::env::var(TIMEOUT_ENV_VAR).map_or(Some(DEFAULT_TIMEOUT), |raw| {
            raw.trim()
                .parse::<u64>()
                .map_or(Some(DEFAULT_TIMEOUT), |seconds| {
                    (seconds != 0).then(|| Duration::from_secs(seconds))
                })
        })
    })
}

/// Number of synchronous operations that reached their wait deadline.
#[must_use]
pub fn timed_out_context_count() -> usize {
    TIMED_OUT_CONTEXTS.load(Ordering::Relaxed)
}

/// Legacy name for [`timed_out_context_count`].
#[deprecated(
    note = "timed-out callback contexts are now reclaimed safely; use timed_out_context_count"
)]
#[must_use]
pub fn abandoned_context_count() -> usize {
    timed_out_context_count()
}

static TIMED_OUT_CONTEXTS: AtomicUsize = AtomicUsize::new(0);
static NEXT_CONTEXT_ID: AtomicUsize = AtomicUsize::new(1);
static CONTEXTS: Mutex<Option<HashMap<usize, Box<dyn Any + Send>>>> = Mutex::new(None);
static NEXT_TIMEOUT_ID: AtomicUsize = AtomicUsize::new(1);
static TIMEOUT_SCHEDULER: OnceLock<Arc<TimeoutScheduler>> = OnceLock::new();

/// Opaque value passed through FFI callbacks.
pub type SyncCompletionPtr = *mut c_void;

#[allow(clippy::significant_drop_tightening)]
fn register_context<T>(context: T) -> (SyncCompletionPtr, usize)
where
    T: Any + Send,
{
    let mut contexts = CONTEXTS.lock().unwrap_or_else(PoisonError::into_inner);
    let contexts = contexts.get_or_insert_with(HashMap::new);

    loop {
        let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 && !contexts.contains_key(&id) {
            contexts.insert(id, Box::new(context));
            return (id as SyncCompletionPtr, id);
        }
    }
}

fn take_context<T>(context: SyncCompletionPtr) -> Option<T>
where
    T: Any + Send,
{
    let id = context as usize;
    if id == 0 {
        return None;
    }

    let entry = CONTEXTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()?
        .remove(&id)?;
    entry.downcast::<T>().ok().map(|entry| *entry)
}

fn remove_context(id: usize) -> bool {
    let removed = {
        let mut contexts = CONTEXTS.lock().unwrap_or_else(PoisonError::into_inner);
        contexts.as_mut().and_then(|contexts| contexts.remove(&id))
    };
    let existed = removed.is_some();
    drop(removed);
    existed
}

fn timeout_message(timeout: Duration) -> String {
    format!("{TIMEOUT_MESSAGE_PREFIX} within {timeout:?}")
}

struct TimeoutEntry {
    id: usize,
    deadline: Instant,
    action: Option<Box<dyn FnOnce() + Send>>,
}

struct TimeoutScheduler {
    entries: Mutex<Vec<TimeoutEntry>>,
    changed: Condvar,
}

impl TimeoutScheduler {
    fn shared() -> &'static Arc<Self> {
        TIMEOUT_SCHEDULER.get_or_init(|| {
            let scheduler = Arc::new(Self {
                entries: Mutex::new(Vec::new()),
                changed: Condvar::new(),
            });
            let worker = Arc::clone(&scheduler);
            std::thread::Builder::new()
                .name("screencapturekit-completions".to_string())
                .spawn(move || worker.run())
                .expect("failed to start completion timeout thread");
            scheduler
        })
    }

    fn schedule(timeout: Duration, action: impl FnOnce() + Send + 'static) -> usize {
        let scheduler = Self::shared();
        let mut entries = scheduler
            .entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let id = loop {
            let id = NEXT_TIMEOUT_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 && !entries.iter().any(|entry| entry.id == id) {
                break id;
            }
        };
        entries.push(TimeoutEntry {
            id,
            deadline: Instant::now() + timeout,
            action: Some(Box::new(action)),
        });
        drop(entries);
        scheduler.changed.notify_one();
        id
    }

    fn cancel(id: usize) {
        if id == 0 {
            return;
        }
        let Some(scheduler) = TIMEOUT_SCHEDULER.get() else {
            return;
        };
        let removed = {
            let mut entries = scheduler
                .entries
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            entries
                .iter()
                .position(|entry| entry.id == id)
                .map(|index| entries.swap_remove(index))
        };
        drop(removed);
        scheduler.changed.notify_one();
    }

    fn run(self: Arc<Self>) -> ! {
        loop {
            let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
            while entries.is_empty() {
                entries = self
                    .changed
                    .wait(entries)
                    .unwrap_or_else(PoisonError::into_inner);
            }

            let (index, deadline) = entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.deadline)
                .map(|(index, entry)| (index, entry.deadline))
                .expect("the timeout queue is non-empty");
            let now = Instant::now();
            if deadline > now {
                let (guard, _) = self
                    .changed
                    .wait_timeout(entries, deadline - now)
                    .unwrap_or_else(PoisonError::into_inner);
                drop(guard);
                continue;
            }

            let mut entry = entries.swap_remove(index);
            drop(entries);
            if let Some(action) = entry.action.take() {
                catch_user_panic("async completion timeout", action);
            }
        }
    }
}

fn cancel_async_timeout<T>(inner: &AsyncCompletionInner<T>) {
    let timeout_id = inner.timeout_id.swap(0, Ordering::AcqRel);
    TimeoutScheduler::cancel(timeout_id);
}

struct SyncCompletionInner<T> {
    result: Mutex<Option<Result<T, String>>>,
    cvar: Condvar,
}

/// A blocking completion handler for asynchronous FFI callbacks.
pub struct SyncCompletion<T: Send + 'static> {
    inner: Arc<SyncCompletionInner<T>>,
    context_id: usize,
}

impl<T: Send + 'static> std::fmt::Debug for SyncCompletion<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let completed = self
            .inner
            .result
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some();
        f.debug_struct("SyncCompletion")
            .field("completed", &completed)
            .finish_non_exhaustive()
    }
}

impl<T: Send + 'static> SyncCompletion<T> {
    /// Create a completion handle and its opaque callback context.
    #[must_use]
    pub fn new() -> (Self, SyncCompletionPtr) {
        let inner = Arc::new(SyncCompletionInner {
            result: Mutex::new(None),
            cvar: Condvar::new(),
        });
        let (context, context_id) = register_context(Arc::clone(&inner));
        (Self { inner, context_id }, context)
    }

    /// Wait until completion or the process-wide default deadline.
    ///
    /// # Errors
    ///
    /// Returns the callback error or a timeout error.
    pub fn wait(self) -> Result<T, String> {
        match default_timeout() {
            Some(timeout) => self.wait_timeout(timeout),
            None => self.wait_forever(),
        }
    }

    /// Wait with an explicit deadline.
    ///
    /// # Errors
    ///
    /// Returns the callback error or a timeout error.
    #[allow(clippy::significant_drop_tightening)]
    pub fn wait_timeout(self, timeout: Duration) -> Result<T, String> {
        let guard = self
            .inner
            .result
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let (mut guard, wait_result) = self
            .inner
            .cvar
            .wait_timeout_while(guard, timeout, |result| result.is_none())
            .unwrap_or_else(PoisonError::into_inner);

        if wait_result.timed_out() && guard.is_none() {
            TIMED_OUT_CONTEXTS.fetch_add(1, Ordering::Relaxed);
            return Err(timeout_message(timeout));
        }

        guard
            .take()
            .unwrap_or_else(|| Err("completion signalled without a result".to_string()))
    }

    /// Wait without a deadline.
    ///
    /// # Errors
    ///
    /// Returns the callback error.
    #[allow(clippy::significant_drop_tightening)]
    pub fn wait_forever(self) -> Result<T, String> {
        let guard = self
            .inner
            .result
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut guard = self
            .inner
            .cvar
            .wait_while(guard, |result| result.is_none())
            .unwrap_or_else(PoisonError::into_inner);
        guard
            .take()
            .unwrap_or_else(|| Err("completion signalled without a result".to_string()))
    }

    /// Complete successfully.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::new`] for this
    /// concrete `T`.
    pub unsafe fn complete_ok(context: SyncCompletionPtr, value: T) {
        unsafe { Self::complete_with_result(context, Ok(value)) };
    }

    /// Complete with an error.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::new`] for this
    /// concrete `T`.
    pub unsafe fn complete_err(context: SyncCompletionPtr, error: String) {
        unsafe { Self::complete_with_result(context, Err(error)) };
    }

    /// Complete with a result.
    ///
    /// Duplicate, late, or cancelled callbacks are ignored safely.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::new`] for this
    /// concrete `T`.
    pub unsafe fn complete_with_result(context: SyncCompletionPtr, result: Result<T, String>) {
        let Some(inner) = take_context::<Arc<SyncCompletionInner<T>>>(context) else {
            return;
        };

        {
            let mut slot = inner.result.lock().unwrap_or_else(PoisonError::into_inner);
            *slot = Some(result);
        }
        inner.cvar.notify_all();
    }
}

impl<T: Send + 'static> Default for SyncCompletion<T> {
    fn default() -> Self {
        Self::new().0
    }
}

impl<T: Send + 'static> Drop for SyncCompletion<T> {
    fn drop(&mut self) {
        remove_context(self.context_id);
    }
}

struct AsyncCompletionState<T> {
    result: Option<Result<T, String>>,
    waker: Option<Waker>,
    completion_hook: Option<AsyncCompletionHook<T>>,
}

type AsyncCompletionHook<T> = Box<dyn FnOnce(&Result<T, String>) + Send>;

struct AsyncCompletionInner<T> {
    state: Mutex<AsyncCompletionState<T>>,
    timeout_id: AtomicUsize,
}

/// Factory for future-based FFI completion handles.
pub struct AsyncCompletion<T: Send + 'static> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + 'static> std::fmt::Debug for AsyncCompletion<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncCompletion").finish_non_exhaustive()
    }
}

/// Future returned by [`AsyncCompletion::create`].
pub struct AsyncCompletionFuture<T: Send + 'static> {
    inner: Arc<AsyncCompletionInner<T>>,
    context_id: usize,
    cancel_on_drop: bool,
}

impl<T: Send + 'static> std::fmt::Debug for AsyncCompletionFuture<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncCompletionFuture")
            .finish_non_exhaustive()
    }
}

impl<T: Send + 'static> AsyncCompletion<T> {
    /// Create a future and its opaque callback context.
    ///
    /// The future resolves with a timeout error if the native callback does
    /// not arrive within [`default_timeout`].
    #[must_use]
    pub fn create() -> (AsyncCompletionFuture<T>, SyncCompletionPtr) {
        Self::create_inner(None, true, default_timeout())
    }

    #[cfg(feature = "async")]
    pub(crate) fn create_with_hook(
        hook: impl FnOnce(&Result<T, String>) + Send + 'static,
    ) -> (AsyncCompletionFuture<T>, SyncCompletionPtr) {
        Self::create_inner(Some(Box::new(hook)), false, default_timeout())
    }

    #[cfg(feature = "async")]
    pub(crate) fn create_unbounded() -> (AsyncCompletionFuture<T>, SyncCompletionPtr) {
        Self::create_inner(None, true, None)
    }

    fn create_inner(
        completion_hook: Option<AsyncCompletionHook<T>>,
        cancel_on_drop: bool,
        timeout: Option<Duration>,
    ) -> (AsyncCompletionFuture<T>, SyncCompletionPtr) {
        let inner = Arc::new(AsyncCompletionInner {
            state: Mutex::new(AsyncCompletionState {
                result: None,
                waker: None,
                completion_hook,
            }),
            timeout_id: AtomicUsize::new(0),
        });
        let (context, context_id) = register_context(Arc::clone(&inner));
        if let Some(timeout) = timeout {
            let context_address = context as usize;
            let timeout_id = TimeoutScheduler::schedule(timeout, move || unsafe {
                Self::complete_err(
                    context_address as SyncCompletionPtr,
                    timeout_message(timeout),
                );
            });
            inner.timeout_id.store(timeout_id, Ordering::Release);
        }
        (
            AsyncCompletionFuture {
                inner,
                context_id,
                cancel_on_drop,
            },
            context,
        )
    }

    /// Complete successfully.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::create`] for this
    /// concrete `T`.
    pub unsafe fn complete_ok(context: SyncCompletionPtr, value: T) {
        unsafe { Self::complete_with_result(context, Ok(value)) };
    }

    /// Complete with an error.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::create`] for this
    /// concrete `T`.
    pub unsafe fn complete_err(context: SyncCompletionPtr, error: String) {
        unsafe { Self::complete_with_result(context, Err(error)) };
    }

    /// Complete with a result. Duplicate, late, or cancelled callbacks are
    /// ignored safely.
    ///
    /// # Safety
    ///
    /// `context` must be the opaque token returned by [`Self::create`] for this
    /// concrete `T`.
    pub unsafe fn complete_with_result(context: SyncCompletionPtr, result: Result<T, String>) {
        let Some(inner) = take_context::<Arc<AsyncCompletionInner<T>>>(context) else {
            return;
        };
        cancel_async_timeout(&inner);

        let completion_hook = {
            let mut state = inner.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.completion_hook.take()
        };
        if let Some(completion_hook) = completion_hook {
            catch_user_panic("async completion hook", || completion_hook(&result));
        }
        let waker = {
            let mut state = inner.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.result = Some(result);
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<T: Send + 'static> Future for AsyncCompletionFuture<T> {
    type Output = Result<T, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker().clone();
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        if let Some(result) = state.result.take() {
            drop(state);
            drop(waker);
            return Poll::Ready(result);
        }

        let replaced = match state.waker.as_ref() {
            Some(existing) if existing.will_wake(&waker) => Some(waker),
            _ => state.waker.replace(waker),
        };
        drop(state);
        drop(replaced);
        Poll::Pending
    }
}

impl<T: Send + 'static> Drop for AsyncCompletionFuture<T> {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            remove_context(self.context_id);
            cancel_async_timeout(&self.inner);
            return;
        }

        let waker = self
            .inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .waker
            .take();
        drop(waker);
    }
}

/// Convert an optional NUL-terminated C string into an owned Rust string.
///
/// # Safety
///
/// `msg` must be null or point to a valid NUL-terminated string.
#[must_use]
pub unsafe fn error_from_cstr(msg: *const i8) -> String {
    if msg.is_null() {
        "Unknown error".to_string()
    } else {
        unsafe { CStr::from_ptr(msg) }
            .to_str()
            .map_or_else(|_| "Unknown error".to_string(), String::from)
    }
}

/// Completion for operations that return only success or an error.
pub type UnitCompletion = SyncCompletion<()>;

impl UnitCompletion {
    /// C callback for `(context, success, error_message)` operations.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub extern "C" fn callback(context: SyncCompletionPtr, success: bool, msg: *const i8) {
        catch_user_panic("UnitCompletion::callback", || {
            if success {
                unsafe { Self::complete_ok(context, ()) };
            } else {
                let error = unsafe { error_from_cstr(msg) };
                unsafe { Self::complete_err(context, error) };
            }
        });
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn completion_hook_runs_after_future_is_dropped() {
        let called = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&called);
        let (future, context) = AsyncCompletion::<()>::create_with_hook(move |result| {
            assert!(result.is_ok());
            observed.store(true, Ordering::Release);
        });

        drop(future);
        unsafe { AsyncCompletion::complete_ok(context, ()) };

        assert!(called.load(Ordering::Acquire));
    }

    #[test]
    fn async_completion_times_out_and_wakes() {
        struct WakeSignal(std::sync::mpsc::Sender<()>);
        impl std::task::Wake for WakeSignal {
            fn wake(self: Arc<Self>) {
                self.0
                    .send(())
                    .expect("timeout wake receiver should remain available");
            }
        }

        let (future, _context) =
            AsyncCompletion::<()>::create_inner(None, true, Some(Duration::from_millis(100)));
        let mut future = Box::pin(future);
        let (wake_sender, wake_receiver) = std::sync::mpsc::channel();
        let waker = Waker::from(Arc::new(WakeSignal(wake_sender)));
        let mut context = Context::from_waker(&waker);
        assert!(future.as_mut().poll(&mut context).is_pending());

        wake_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("completion did not wake after its deadline");
        let Poll::Ready(Err(error)) = future.as_mut().poll(&mut context) else {
            panic!("completion did not resolve after its deadline");
        };
        assert!(is_timeout_error(&error));
    }

    #[test]
    fn unbounded_completion_does_not_register_a_deadline() {
        let (future, context) = AsyncCompletion::<()>::create_unbounded();
        assert_eq!(future.inner.timeout_id.load(Ordering::Acquire), 0);

        drop(future);
        unsafe { AsyncCompletion::complete_ok(context, ()) };
    }
}
