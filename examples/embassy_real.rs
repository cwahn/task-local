//! Real Embassy executor test
//!
//! Tests task-local storage with actual Embassy executor.
//!
//! Run with: cargo run --example embassy_real

use task_local::task_local;
use embassy_executor::{Spawner, SendSpawner};

// Define task-locals for testing
task_local! {
    static TASK_VALUE: u32;
    static SHARED_STATE: &'static str;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    println!("Testing task-local storage with REAL Embassy executor");
    
    // Spawn the same task multiple times with different contexts
    spawner.spawn(sensor_task(1, 100)).unwrap();
    spawner.spawn(sensor_task(2, 200)).unwrap();
    
    // Get SendSpawner and spawn coordinator task with it
    let send_spawner = spawner.make_send();
    spawner.spawn(coordinator_task(send_spawner)).unwrap();
    
    // Keep main running
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task(pool_size = 4)]
async fn sensor_task(id: u8, value: u32) {
    TASK_VALUE.scope(value, async {
        SHARED_STATE.scope("Sensor", async {
            
            println!("Sensor {}: TASK_VALUE = {}", id, TASK_VALUE.get());
            println!("Sensor {}: SHARED_STATE = {}", id, SHARED_STATE.get());
            
            embassy_time::Timer::after(embassy_time::Duration::from_millis(300 * id as u64)).await;
            
            println!("Sensor {} after await: TASK_VALUE = {}", id, TASK_VALUE.get());
            println!("Sensor {} after await: SHARED_STATE = {}", id, SHARED_STATE.get());
            
        }).await;
    }).await;
}

#[embassy_executor::task]
async fn coordinator_task(send_spawner: SendSpawner) {
    TASK_VALUE.scope(999, async {
        SHARED_STATE.scope("Coordinator", async {
            
            println!("Coordinator: TASK_VALUE = {}", TASK_VALUE.get());
            println!("Coordinator: SHARED_STATE = {}", SHARED_STATE.get());
            
            // Use the SendSpawner parameter to spawn nested tasks
            send_spawner.spawn(nested_task(1)).unwrap();
            send_spawner.spawn(nested_task(2)).unwrap();
            
            embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
            
            println!("Coordinator after spawning: TASK_VALUE = {}", TASK_VALUE.get());
            println!("Coordinator after spawning: SHARED_STATE = {}", SHARED_STATE.get());
            
        }).await;
    }).await;
}

#[embassy_executor::task(pool_size = 4)]
async fn nested_task(id: u8) {
    TASK_VALUE.scope(500 + id as u32, async {
        SHARED_STATE.scope("Nested", async {
            
            println!("  Nested {}: TASK_VALUE = {}", id, TASK_VALUE.get());
            println!("  Nested {}: SHARED_STATE = {}", id, SHARED_STATE.get());
            
            embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;
            
            println!("  Nested {} after await: TASK_VALUE = {}", id, TASK_VALUE.get());
            println!("  Nested {} after await: SHARED_STATE = {}", id, SHARED_STATE.get());
            
        }).await;
    }).await;
}
