//! Minimal Embassy example showing task-local usage
//!
//! This demonstrates the basic usage patterns with Embassy async tasks.
//! 
//! In a real Embassy project, you would:
//! 1. Add this to Cargo.toml: task-local = { version = "0.1", default-features = false }
//! 2. Use this pattern in your Embassy tasks

use task_local::task_local;

// Define task-locals for Embassy context
task_local! {
    static DEVICE_ID: u32;
    static SENSOR_VALUE: i32;
}

/// This is how you'd use task-local in a real Embassy task
/// 
/// ```rust
/// #[embassy_executor::task]
/// async fn sensor_task() {
///     embassy_sensor_example().await;
/// }
/// ```
pub async fn embassy_sensor_example() {
    // Set device context for this task
    DEVICE_ID.scope(0x1234, async {
        
        // Simulate sensor reading with context
        SENSOR_VALUE.scope(42, async {
            let _device = DEVICE_ID.get();
            let _value = SENSOR_VALUE.get();
            
            // In real Embassy:
            // Timer::after(Duration::from_millis(100)).await;
            // let reading = i2c.read_sensor().await.unwrap();
            // channel.send((device, reading)).await;
            
        }).await;
        
    }).await;
}

/// Synchronous version for initialization
pub fn embassy_init_example() {
    DEVICE_ID.sync_scope(0x5678, || {
        let _device = DEVICE_ID.get();
        
        // In real Embassy:
        // configure_hardware(device);
        // setup_interrupts();
    });
}

/// Entry point for testing (not needed in real Embassy)
fn main() {
    println!("Embassy minimal example");
    
    // Demo the sync version
    embassy_init_example();
    
    println!("Embassy example completed successfully!");
    
    // In real Embassy, you'd spawn the async task:
    // spawner.spawn(sensor_task()).unwrap();
}

// Real Embassy usage would look like this:
/*
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use task_local::task_local;

task_local! {
    static DEVICE_ID: u32;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(sensor_task()).unwrap();
}

#[embassy_executor::task]
async fn sensor_task() {
    DEVICE_ID.scope(0x1234, async {
        loop {
            let device = DEVICE_ID.get();
            
            // Your sensor logic here
            Timer::after(Duration::from_millis(1000)).await;
        }
    }).await;
}
*/
