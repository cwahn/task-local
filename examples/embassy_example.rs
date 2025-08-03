//! Example: Using task-local with Embassy
//!
//! This example demonstrates how to use task-local storage in an Embassy-based
//! embedded application. This is a simplified version that works in no_std.
//!
//! To build for no_std:  
//! ```bash
//! cargo check --example embassy_example --no-default-features
//! ```

#![no_std]
#![no_main]
#![allow(unused_variables)]

use task_local::task_local;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

// Simulate Embassy components for compilation testing
pub struct Duration(pub u64);
pub struct Timer;

impl Timer {
    pub fn after(_duration: Duration) -> DelayFuture {
        DelayFuture { completed: false }
    }
}

pub struct DelayFuture {
    completed: bool,
}

impl Future for DelayFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            Poll::Ready(())
        } else {
            self.completed = true;
            Poll::Ready(()) // Complete immediately for testing
        }
    }
}

impl Duration {
    pub const fn from_millis(ms: u64) -> Self {
        Self(ms)
    }
}

// Define task-local storage for different contexts
task_local! {
    /// Current sensor reading context
    static SENSOR_CONTEXT: SensorContext;
    
    /// Task priority level
    static TASK_PRIORITY: u8;
    
    /// Device ID for logging/debugging
    static DEVICE_ID: u32;
    
    /// Current operation mode
    static OPERATION_MODE: OperationMode;
}

#[derive(Clone, Copy, Debug)]
struct SensorContext {
    sensor_id: u8,
    sampling_rate: u32,
    calibration_offset: i16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OperationMode {
    Normal,
    PowerSave,
    Calibration,
    Emergency,
}

/// Main Embassy application entry point
#[no_mangle]
pub fn embassy_main() {
    // This would be called by your Embassy executor
    // For now, we'll just demonstrate the API usage
    demo_task_local_usage();
}

/// Demonstrate task-local usage patterns that work in Embassy
fn demo_task_local_usage() {
    // Example 1: Basic synchronous scope
    DEVICE_ID.sync_scope(0x12345678, || {
        OPERATION_MODE.sync_scope(OperationMode::Normal, || {
            let device_id = DEVICE_ID.get();
            let mode = OPERATION_MODE.get();
            
            // In real Embassy, you might log this via RTT or UART
            // rtt_println!("Device {:08X} in {:?} mode", device_id, mode);
            
            // Demonstrate nested scopes
            TASK_PRIORITY.sync_scope(1, || {
                let priority = TASK_PRIORITY.get();
                // Process with normal priority
                
                TASK_PRIORITY.sync_scope(3, || {
                    let high_priority = TASK_PRIORITY.get(); 
                    // Process with high priority
                    // assert_eq!(high_priority, 3);
                });
                
                // Back to normal priority
                let normal_priority = TASK_PRIORITY.get();
                // assert_eq!(normal_priority, 1);
            });
        });
    });
    
    // Example 2: Sensor context usage
    let temp_sensor = SensorContext {
        sensor_id: 1,
        sampling_rate: 1000,
        calibration_offset: -2,
    };
    
    SENSOR_CONTEXT.sync_scope(temp_sensor, || {
        let context = SENSOR_CONTEXT.get();
        let raw_reading = 250; // Simulated sensor value
        let calibrated = raw_reading + context.calibration_offset as i32;
        
        // In real Embassy:
        // rtt_println!("Sensor {}: {} -> {}", context.sensor_id, raw_reading, calibrated);
    });
    
    // Example 3: Error handling
    match DEVICE_ID.try_with(|id| *id) {
        Ok(_id) => {
            // Device ID is available
        }
        Err(_) => {
            // Device ID not set in current context
        }
    }
}

/// Async function demonstrating scope usage (for Embassy tasks)
async fn async_sensor_task() {
    let sensor_context = SensorContext {
        sensor_id: 2,
        sampling_rate: 500,
        calibration_offset: 5,
    };
    
    SENSOR_CONTEXT.scope(sensor_context, async {
        TASK_PRIORITY.scope(1, async {
            // Simulate sensor readings
            for _i in 0..5 {
                let context = SENSOR_CONTEXT.get();
                let priority = TASK_PRIORITY.get();
                
                // Simulate sensor reading
                let raw_value = simulate_sensor_reading().await;
                let calibrated = raw_value + context.calibration_offset as i32;
                
                // In real Embassy, you might:
                // - Send data via channel to another task
                // - Log via RTT
                // - Update hardware registers
                // rtt_println!("[P{}] Sensor {}: {}", priority, context.sensor_id, calibrated);
                
                // Wait for next reading
                Timer::after(Duration::from_millis(100)).await;
            }
        }).await;
    }).await;
}

/// Simulate reading from a sensor
async fn simulate_sensor_reading() -> i32 {
    // In real Embassy, this would:
    // - Configure I2C/SPI peripheral
    // - Read from actual sensor
    // - Handle errors appropriately
    
    Timer::after(Duration::from_millis(10)).await; // Simulate I2C delay
    42 // Dummy sensor value
}

/// Example of how you might structure Embassy tasks with task-locals
async fn embassy_system_task() {
    DEVICE_ID.scope(0xDEADBEEF, async {
        OPERATION_MODE.scope(OperationMode::Normal, async {
            
            // High priority initialization
            TASK_PRIORITY.scope(3, async {
                initialize_hardware().await;
            }).await;
            
            // Normal operation
            TASK_PRIORITY.scope(1, async {
                run_main_loop().await;
            }).await;
            
            // Emergency handling
            OPERATION_MODE.scope(OperationMode::Emergency, async {
                TASK_PRIORITY.scope(3, async {
                    handle_emergency().await;
                }).await;
            }).await;
            
        }).await;
    }).await;
}

async fn initialize_hardware() {
    // Initialize clocks, peripherals, etc.
    Timer::after(Duration::from_millis(50)).await;
}

async fn run_main_loop() {
    // Main application logic
    for _cycle in 0..3 {
        Timer::after(Duration::from_millis(100)).await;
    }
}

async fn handle_emergency() {
    // Emergency shutdown or recovery
    Timer::after(Duration::from_millis(10)).await;
}

/// Panic handler for no_std (only when actually in no_std embedded environment)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In real Embassy applications:
    // - Log panic info via RTT/UART
    // - Safely shutdown peripherals
    // - Reset the system
    loop {}
}

// Note: In a real Embassy application, you would:
//
// 1. Use the Embassy executor:
//    ```rust
//    #[embassy_executor::main]
//    async fn main(spawner: Spawner) {
//        spawner.spawn(async_sensor_task()).unwrap();
//        spawner.spawn(embassy_system_task()).unwrap();
//    }
//    ```
//
// 2. Include proper hardware initialization
// 3. Use real Embassy timers and peripherals
// 4. Handle interrupts and hardware events
// 5. Use Embassy channels for task communication
// 6. Implement proper error handling and recovery
