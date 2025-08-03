// Example: Basic usage that works in both std and no_std environments
//
// This example demonstrates the basic functionality that works in both
// std and no_std environments. For no_std usage, compile with:
// cargo run --example no_std_basic --no-default-features

use task_local::task_local;

task_local! {
    static COUNTER: u32;
    static MESSAGE: &'static str;
}

fn main() {
    println!("Task-local example (works in both std and no_std)");

    // Basic usage with sync_scope
    COUNTER.sync_scope(42, || {
        println!("Counter value: {}", COUNTER.get());
        assert_eq!(COUNTER.get(), 42);
    });

    MESSAGE.sync_scope("Hello, World!", || {
        println!("Message: {}", MESSAGE.get());
        assert_eq!(MESSAGE.get(), "Hello, World!");
    });

    // Nested scopes
    COUNTER.sync_scope(1, || {
        println!("Outer counter: {}", COUNTER.get());
        
        COUNTER.sync_scope(2, || {
            println!("Inner counter: {}", COUNTER.get());
            assert_eq!(COUNTER.get(), 2);
        });
        
        println!("Back to outer counter: {}", COUNTER.get());
        assert_eq!(COUNTER.get(), 1);
    });

    // Testing error case
    let result = COUNTER.try_with(|_| ());
    assert!(result.is_err());
    println!("Correctly got error when trying to access unset task-local");

    println!("All tests passed!");
}

// For embedded/no_std usage, you might do something like:
// #[no_std]
// #[no_main]
// 
// // Your Embassy or other embedded setup here
// 
// #[embassy_executor::task]
// async fn my_task() {
//     COUNTER.scope(100, async {
//         // Your async task-local code here
//         let value = COUNTER.get();
//         // ... do something with value
//     }).await;
// }
