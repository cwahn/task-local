//! Simple Embassy compatibility demo
//!
//! This demonstrates that the task-local API works correctly in no_std
//! environments like Embassy. This compiles to show API compatibility.

#![no_std]
#![allow(unused_variables)]

use task_local::task_local;

// Define task-locals that would be useful in Embassy applications
task_local! {
    /// Device configuration context
    static DEVICE_CONFIG: DeviceConfig;
    
    /// Current task priority
    static TASK_PRIORITY: u8;
    
    /// Sensor calibration data
    static SENSOR_CAL: SensorCalibration;
}

#[derive(Clone, Copy)]
struct DeviceConfig {
    device_id: u32,
    firmware_version: u16,
    power_mode: PowerMode,
}

#[derive(Clone, Copy)]
enum PowerMode {
    Normal,
    LowPower,
    Sleep,
}

#[derive(Clone, Copy)]
struct SensorCalibration {
    offset: i16,
    scale: u16,
}

/// Demo function showing Embassy-style usage patterns
pub fn embassy_demo() {
    // Device initialization with configuration
    let config = DeviceConfig {
        device_id: 0x12345678,
        firmware_version: 0x0100,
        power_mode: PowerMode::Normal,
    };
    
    DEVICE_CONFIG.sync_scope(config, || {
        // High priority system initialization
        TASK_PRIORITY.sync_scope(3, || {
            // Initialize hardware
            init_hardware();
        });
        
        // Normal priority sensor setup
        TASK_PRIORITY.sync_scope(1, || {
            let cal = SensorCalibration {
                offset: -10,
                scale: 1000,
            };
            
            SENSOR_CAL.sync_scope(cal, || {
                // Process sensor data
                let raw_reading = 2048; // Simulated ADC reading
                let calibrated = apply_calibration(raw_reading);
                
                // In Embassy, you'd typically:
                // - Send via channel to another task
                // - Store in a shared resource
                // - Log via RTT
                
                assert_eq!(calibrated, 2038); // 2048 + (-10)
            });
        });
        
        // Demonstrate nested scopes for power management
        DEVICE_CONFIG.sync_scope(
            DeviceConfig {
                device_id: config.device_id,
                firmware_version: config.firmware_version,
                power_mode: PowerMode::LowPower,
            },
            || {
                TASK_PRIORITY.sync_scope(2, || {
                    enter_low_power_mode();
                });
            }
        );
    });
}

fn init_hardware() {
    let config = DEVICE_CONFIG.get();
    let priority = TASK_PRIORITY.get();
    
    // In Embassy:
    // - Configure clocks based on config
    // - Initialize peripherals
    // - Setup interrupt priorities based on task priority
}

fn apply_calibration(raw_value: i32) -> i32 {
    let cal = SENSOR_CAL.get();
    let config = DEVICE_CONFIG.get();
    
    // Apply calibration with device-specific adjustments
    let calibrated = raw_value + cal.offset as i32;
    
    // In Embassy, you might log this:
    // rtt_println!("Device {:08X}: {} -> {}", config.device_id, raw_value, calibrated);
    
    calibrated
}

fn enter_low_power_mode() {
    let config = DEVICE_CONFIG.get();
    let priority = TASK_PRIORITY.get();
    
    // In Embassy:
    // - Reduce clock speeds
    // - Disable unused peripherals  
    // - Configure wake-up sources
}

/// Async version showing how it would work with Embassy futures
/// (This demonstrates the API, but can't actually run without Embassy executor)
async fn embassy_async_demo() {
    let config = DeviceConfig {
        device_id: 0xCAFEBABE,
        firmware_version: 0x0200,
        power_mode: PowerMode::Normal,
    };
    
    DEVICE_CONFIG.scope(config, async {
        TASK_PRIORITY.scope(1, async {
            // This would work with Embassy's Timer, I2C, SPI, etc.
            
            let cal = SensorCalibration {
                offset: 5,
                scale: 500,
            };
            
            SENSOR_CAL.scope(cal, async {
                // Simulate async sensor reading
                // Timer::after(Duration::from_millis(100)).await;
                // let reading = i2c.read_sensor().await?;
                // let calibrated = apply_calibration(reading);
                // 
                // channel.send(calibrated).await;
            }).await;
        }).await;
    }).await;
}

/// Test function to verify the API works
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embassy_compatibility() {
        embassy_demo();
        
        // Verify error handling
        let result = DEVICE_CONFIG.try_with(|config| config.device_id);
        assert!(result.is_err()); // Should fail when not in scope
    }
}

fn main() {
    embassy_demo();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
