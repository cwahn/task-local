#![no_std]
#![no_main]

// This example demonstrates no_std usage
// Note: This won't actually run since it needs a panic handler and allocator,
// but it shows that the library compiles in no_std environments

use task_local::task_local;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

task_local! {
    static COUNTER: u32;
    static MESSAGE: &'static str;
}

// A minimal future for demonstration
struct SimpleFuture {
    completed: bool,
}

impl SimpleFuture {
    fn new() -> Self {
        Self { completed: false }
    }
}

impl Future for SimpleFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            Poll::Ready(42)
        } else {
            self.completed = true;
            // Access the task-local value
            let value = COUNTER.get();
            Poll::Ready(value)
        }
    }
}

// This would be called by your embedded runtime/executor
#[no_mangle]
pub fn demo_function() {
    // Synchronous scope
    COUNTER.sync_scope(10, || {
        let value = COUNTER.get();
        assert_eq!(value, 10);
    });

    MESSAGE.sync_scope("hello", || {
        let msg = MESSAGE.get();
        assert_eq!(msg, "hello");
    });

    // For async scope, you would need an executor like Embassy
    // let future = COUNTER.scope(20, SimpleFuture::new());
    // executor.run(future);
}

// Required for no_std no_main
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
