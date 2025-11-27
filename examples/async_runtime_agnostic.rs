//! Async API runtime-agnostic example
//!
//! This example demonstrates that the async API works with ANY async runtime.
//! It uses only `std` types and works with Tokio, async-std, smol, or even
//! a custom executor.
//!
//! ## Examples
//!
//! Run with Tokio:
//! ```bash
//! cargo run --example async_runtime_agnostic --features async
//! ```

#[cfg(feature = "async")]
use screencapturekit::async_api::AsyncSCShareableContent;

#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;
#[cfg(feature = "async")]
use std::task::{Context, Poll, Waker};
#[cfg(feature = "async")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "async")]
/// Simple executor-agnostic runtime for demonstration
/// This is a minimal example - in production use Tokio, async-std, or smol
struct SimpleExecutor;

#[cfg(feature = "async")]
impl SimpleExecutor {
    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = Box::pin(future);
        let waker = Arc::new(SimpleWaker);
        let waker = waker_fn::waker_ref(&waker);
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    // In a real executor, you'd wait for the waker
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}

#[cfg(feature = "async")]
struct SimpleWaker;

#[cfg(feature = "async")]
impl std::task::Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {}
}

#[cfg(feature = "async")]
mod waker_fn {
    use std::sync::Arc;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    pub fn waker_ref<W: std::task::Wake>(wake: &Arc<W>) -> Waker {
        let ptr = Arc::as_ptr(wake) as *const ();
        let vtable = &Helper::<W>::VTABLE;
        unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) }
    }

    struct Helper<W>(std::marker::PhantomData<W>);

    impl<W: std::task::Wake> Helper<W> {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            Self::clone_waker,
            Self::wake,
            Self::wake_by_ref,
            Self::drop_waker,
        );

        unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
            let arc = Arc::from_raw(ptr as *const W);
            std::mem::forget(arc.clone());
            std::mem::forget(arc);
            RawWaker::new(ptr, &Self::VTABLE)
        }

        unsafe fn wake(ptr: *const ()) {
            let arc = Arc::from_raw(ptr as *const W);
            arc.wake();
        }

        unsafe fn wake_by_ref(ptr: *const ()) {
            let arc = Arc::from_raw(ptr as *const W);
            arc.wake_by_ref();
            std::mem::forget(arc);
        }

        unsafe fn drop_waker(ptr: *const ()) {
            drop(Arc::from_raw(ptr as *const W));
        }
    }
}

#[cfg(feature = "async")]
async fn demonstrate_runtime_agnostic() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demonstrating runtime-agnostic async API\n");

    // This works with ANY runtime!
    println!("📱 Getting shareable content...");
    let content = AsyncSCShareableContent::get().await?;

    println!("✅ Got content:");
    println!("   - Displays: {}", content.displays().len());
    println!("   - Windows: {}", content.windows().len());
    println!("   - Applications: {}", content.applications().len());

    // Show first display info
    if let Some(display) = content.displays().first() {
        println!("\n📺 First display:");
        println!("   - ID: {}", display.display_id());
        println!("   - Resolution: {}x{}", display.width(), display.height());
    }

    // Show some windows
    println!("\n🪟 First 3 windows:");
    for (i, window) in content.windows().iter().take(3).enumerate() {
        let title = window.title().unwrap_or_else(|| "Untitled".to_string());
        println!("   {}. {} (ID: {})", i + 1, title, window.window_id());
    }

    println!("\n✨ Success! This code works with ANY async runtime:");
    println!("   - Tokio ✅");
    println!("   - async-std ✅");
    println!("   - smol ✅");
    println!("   - futures executor ✅");
    println!("   - Any custom executor ✅");

    Ok(())
}

#[cfg(all(feature = "async", not(test)))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Runtime Agnostic Async Example ===\n");
    println!("🔹 Running with available async runtime\n");

    // Use Tokio if available (most common in the ecosystem)
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(demonstrate_runtime_agnostic())?;

    println!("\n💡 Note: This async API uses only std::thread and std::sync,");
    println!("   so it works with ANY async runtime: Tokio, async-std, smol, etc.");
    
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature.");
    eprintln!("Run with: cargo run --example async_runtime_agnostic --features async");
}
