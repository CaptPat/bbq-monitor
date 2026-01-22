// src/lib.rs
pub mod config;
pub mod database;
pub mod device_capabilities;
pub mod protocol;
pub mod web_server;
pub mod premium;
#[cfg(feature = "aws")]
pub mod aws_client;

pub use config::*;
pub use database::*;
pub use device_capabilities::*;
pub use protocol::*;
pub use web_server::*;
pub use premium::*;
#[cfg(feature = "aws")]
pub use aws_client::*;

// FFI exports for Flutter integration
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Duration;

/// Validates a license key from Flutter/Dart via FFI
/// Returns 1 if valid, 0 if invalid
#[no_mangle]
pub extern "C" fn validate_license(key_ptr: *const c_char) -> i8 {
    if key_ptr.is_null() {
        return 0;
    }
    
    let c_str = unsafe { CStr::from_ptr(key_ptr) };
    let key = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let validator = LicenseValidator::new();
    match validator.validate(key) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Gets license information as JSON string
/// Returns JSON string pointer (must be freed with free_license_json)
#[no_mangle]
pub extern "C" fn get_license_info(key_ptr: *const c_char) -> *mut c_char {
    if key_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(key_ptr) };
    let key = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let validator = LicenseValidator::new();
    match validator.validate(key) {
        Ok(license) => {
            let json = serde_json::json!({
                "tier": format!("{:?}", license.tier),
                "features": {
                    "cloud_sync": license.features.cloud_sync,
                    "unlimited_history": license.features.unlimited_history,
                    "cook_profiles": license.features.cook_profiles,
                    "advanced_analytics": license.features.advanced_analytics,
                    "alerts": license.features.alerts,
                },
                "expires_at": license.expires_at,
            });
            
            match CString::new(json.to_string()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a JSON string allocated by get_license_info
#[no_mangle]
pub extern "C" fn free_license_json(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// BLE FFI exports for device scanning and management

use once_cell::sync::Lazy;
use std::sync::Mutex;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;

// Global BLE state
static BLE_MANAGER: Lazy<Mutex<Option<Manager>>> = Lazy::new(|| Mutex::new(None));
static BLE_DEVICES: Lazy<Mutex<Vec<serde_json::Value>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Initialize the BLE manager (must be called first)
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub extern "C" fn ble_initialize() -> i8 {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return 0,
    };
    
    rt.block_on(async {
        match Manager::new().await {
            Ok(manager) => {
                let mut mgr = BLE_MANAGER.lock().unwrap();
                *mgr = Some(manager);
                1
            }
            Err(_) => 0,
        }
    })
}

/// Start scanning for BBQ devices
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub extern "C" fn ble_start_scan() -> i8 {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return 0,
    };
    
    rt.block_on(async {
        let mgr = BLE_MANAGER.lock().unwrap();
        let manager = match mgr.as_ref() {
            Some(m) => m,
            None => return 0,
        };
        
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(_) => return 0,
        };
        
        if adapters.is_empty() {
            return 0;
        }
        
        let adapter = &adapters[0];
        match adapter.start_scan(ScanFilter::default()).await {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

/// Stop scanning for devices
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub extern "C" fn ble_stop_scan() -> i8 {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return 0,
    };
    
    rt.block_on(async {
        let mgr = BLE_MANAGER.lock().unwrap();
        let manager = match mgr.as_ref() {
            Some(m) => m,
            None => return 0,
        };
        
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(_) => return 0,
        };
        
        if adapters.is_empty() {
            return 0;
        }
        
        let adapter = &adapters[0];
        match adapter.stop_scan().await {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

/// Get scanned devices as JSON array string
/// Returns JSON string pointer (must be freed with ble_free_devices_json)
#[no_mangle]
pub extern "C" fn ble_get_devices() -> *mut c_char {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    
    rt.block_on(async {
        let mgr = BLE_MANAGER.lock().unwrap();
        let manager = match mgr.as_ref() {
            Some(m) => m,
            None => return std::ptr::null_mut(),
        };
        
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(_) => return std::ptr::null_mut(),
        };
        
        if adapters.is_empty() {
            return std::ptr::null_mut();
        }
        
        let adapter = &adapters[0];
        let peripherals = match adapter.peripherals().await {
            Ok(p) => p,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let mut devices = Vec::new();
        
        for peripheral in peripherals {
            if let Ok(Some(properties)) = peripheral.properties().await {
                let name = properties.local_name.unwrap_or_else(|| "Unknown".to_string());
                let address = properties.address.to_string();
                
                // Filter for BBQ devices
                let name_lower = name.to_lowercase();
                let is_bbq_device = name.starts_with("cA00") || 
                                   name.starts_with("cA02") || 
                                   name.starts_with("Y0C") ||
                                   name_lower.contains("meater") ||
                                   name_lower.contains("igrill") ||
                                   name_lower.contains("weber") ||
                                   name_lower.contains("inkbird") ||
                                   name_lower.contains("thermoworks");
                
                if is_bbq_device || !name.is_empty() {
                    devices.push(serde_json::json!({
                        "id": address,
                        "name": name,
                        "rssi": properties.rssi.unwrap_or(0),
                        "isConnected": false,
                    }));
                }
            }
        }
        
        // Store devices for later use
        let mut stored_devices = BLE_DEVICES.lock().unwrap();
        *stored_devices = devices.clone();
        
        let json = serde_json::to_string(&devices).unwrap_or_else(|_| "[]".to_string());
        match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Free devices JSON string
#[no_mangle]
pub extern "C" fn ble_free_devices_json(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// Database query FFI exports for Flutter to read data

/// Get all devices from database as JSON array
/// Returns JSON string pointer (must be freed with db_free_json)
#[no_mangle]
pub extern "C" fn db_get_devices(db_path_ptr: *const c_char) -> *mut c_char {
    if db_path_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(db_path_ptr) };
    let db_path = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    
    rt.block_on(async {
        let db = match Database::new(db_path).await {
            Ok(db) => db,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let devices = match db.get_all_devices().await {
            Ok(d) => d,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let json = match serde_json::to_string(&devices) {
            Ok(j) => j,
            Err(_) => return std::ptr::null_mut(),
        };
        
        match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Get recent temperature readings for a device as JSON array
/// limit: number of readings to return (0 = all)
/// Returns JSON string pointer (must be freed with db_free_json)
#[no_mangle]
pub extern "C" fn db_get_readings(
    db_path_ptr: *const c_char,
    device_id_ptr: *const c_char,
    limit: i32,
) -> *mut c_char {
    if db_path_ptr.is_null() || device_id_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str_path = unsafe { CStr::from_ptr(db_path_ptr) };
    let db_path = match c_str_path.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let c_str_id = unsafe { CStr::from_ptr(device_id_ptr) };
    let device_id = match c_str_id.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    
    rt.block_on(async {
        let db = match Database::new(db_path).await {
            Ok(db) => db,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let readings = match db.get_device_readings(device_id, limit as usize).await {
            Ok(r) => r,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let json = match serde_json::to_string(&readings) {
            Ok(j) => j,
            Err(_) => return std::ptr::null_mut(),
        };
        
        match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Get latest reading for a device as JSON object
/// Returns JSON string pointer (must be freed with db_free_json)
#[no_mangle]
pub extern "C" fn db_get_latest_reading(
    db_path_ptr: *const c_char,
    device_id_ptr: *const c_char,
) -> *mut c_char {
    if db_path_ptr.is_null() || device_id_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str_path = unsafe { CStr::from_ptr(db_path_ptr) };
    let db_path = match c_str_path.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let c_str_id = unsafe { CStr::from_ptr(device_id_ptr) };
    let device_id = match c_str_id.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    
    rt.block_on(async {
        let db = match Database::new(db_path).await {
            Ok(db) => db,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let reading = match db.get_latest_reading(device_id).await {
            Ok(r) => r,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let json = match serde_json::to_string(&reading) {
            Ok(j) => j,
            Err(_) => return std::ptr::null_mut(),
        };
        
        match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Get temperature history for a device within a time range as JSON array
/// start_time: ISO 8601 timestamp string (e.g., "2026-01-20T00:00:00Z")
/// end_time: ISO 8601 timestamp string
/// Returns JSON string pointer (must be freed with db_free_json)
#[no_mangle]
pub extern "C" fn db_get_history(
    db_path_ptr: *const c_char,
    device_id_ptr: *const c_char,
    start_time_ptr: *const c_char,
    end_time_ptr: *const c_char,
) -> *mut c_char {
    if db_path_ptr.is_null() || device_id_ptr.is_null() || 
       start_time_ptr.is_null() || end_time_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let db_path = match unsafe { CStr::from_ptr(db_path_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let device_id = match unsafe { CStr::from_ptr(device_id_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let start_time_str = match unsafe { CStr::from_ptr(start_time_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let end_time_str = match unsafe { CStr::from_ptr(end_time_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let start_time = match chrono::DateTime::parse_from_rfc3339(start_time_str) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return std::ptr::null_mut(),
    };
    let end_time = match chrono::DateTime::parse_from_rfc3339(end_time_str) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return std::ptr::null_mut(),
    };
    
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return std::ptr::null_mut(),
    };
    
    rt.block_on(async {
        let db = match Database::new(db_path).await {
            Ok(db) => db,
            Err(_) => return std::ptr::null_mut(),
        };
        let readings = match db.get_readings_in_range(device_id, start_time, end_time).await {
            Ok(r) => r,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let json = match serde_json::to_string(&readings) {
            Ok(j) => j,
            Err(_) => return std::ptr::null_mut(),
        };
        match CString::new(json) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Free JSON string allocated by database query functions
#[no_mangle]
pub extern "C" fn db_free_json(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// Background task management

static BLE_TASK_RUNNING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Start background BLE monitoring task
/// This will continuously scan for devices, connect, and write data to SQLite
/// Also starts AWS sync if configured
/// Returns 1 on success, 0 on failure
#[no_mangle]
pub extern "C" fn start_background_monitor(
    db_path_ptr: *const c_char,
    config_path_ptr: *const c_char,
) -> i8 {
    if db_path_ptr.is_null() || config_path_ptr.is_null() {
        return 0;
    }
    
    let mut running = BLE_TASK_RUNNING.lock().unwrap();
    if *running {
        return 0; // Already running
    }
    
    let db_path = match unsafe { CStr::from_ptr(db_path_ptr) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let config_path = match unsafe { CStr::from_ptr(config_path_ptr) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    // Spawn background thread
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return,
        };
        
        rt.block_on(async {
            // Load config
            let config = match Config::load_from_path(&config_path) {
                Ok(c) => c,
                Err(_) => return,
            };
            
            // Initialize database
            let db = match Database::new(&db_path).await {
                Ok(db) => Arc::new(db),
                Err(_) => return,
            };
            
            // Validate license
            let validator = LicenseValidator::new();
            #[allow(unused_variables)]
            let license = match validator.validate(&config.premium.license_key) {
                Ok(l) => {
                    let lic = Arc::new(l);
                    println!("License validated: expires {:?}", lic.expires_at);
                    lic
                },
                Err(_) => return,
            };
            
            // Start AWS sync if enabled
            #[cfg(feature = "aws")]
            let _aws_task = if config.aws.enabled && license.features.cloud_sync {
                let aws_config = bbq_monitor::aws_client::AwsConfig {
                    region: config.aws.region.clone(),
                    thing_name: config.aws.thing_name.clone(),
                    table_name: config.aws.table_name.clone(),
                    sync_interval_secs: config.aws.sync_interval_secs,
                };
                
                if let Ok(client) = AwsClient::new(aws_config, db.clone()).await {
                    let client = Arc::new(client);
                    let (tx, rx) = broadcast::channel::<()>(1);
                    tokio::spawn(async move {
                        client.start_sync_task(rx).await;
                    });
                    Some(tx)
                } else {
                    None
                }
            } else {
                None
            };
            
            // BLE monitoring loop
            loop {
                if let Err(e) = run_ble_scan_cycle(&db, &config).await {
                    eprintln!("BLE scan cycle error: {}", e);
                }
                
                // Wait before next scan
                tokio::time::sleep(Duration::from_secs(config.device.scan_duration + 5)).await;
            }
        });
    });
    
    *running = true;
    1
}

async fn run_ble_scan_cycle(db: &Database, config: &Config) -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    
    if adapters.is_empty() {
        return Ok(());
    }
    
    let adapter = &adapters[0];
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(Duration::from_secs(config.device.scan_duration)).await;
    
    let peripherals = adapter.peripherals().await?;
    
    for peripheral in peripherals {
        if let Ok(Some(properties)) = peripheral.properties().await {
            let name = properties.local_name.unwrap_or_default();
            let address = properties.address.to_string();
            
            // Check if BBQ device
            if !is_bbq_device_name(&name) {
                continue;
            }
            
            // Try to connect and read data
            if peripheral.connect().await.is_ok() {
                peripheral.discover_services().await?;
                
                // Read temperature and store in DB
                // (Simplified - full implementation would handle all characteristics)
                let services = peripheral.services();
                for service in &services {
                    if service.uuid == MEATSTICK_SERVICE {
                        for characteristic in &service.characteristics {
                            if characteristic.uuid == MEATSTICK_CHAR {
                                if let Ok(data) = peripheral.read(characteristic).await {
                                    if let Ok(temps) = MeatStickProtocol::parse_temperature_data(&data) {
                                        let timestamp = chrono::Utc::now();
                                        let ambient = MeatStickProtocol::get_ambient_temp(&temps);
                                        
                                        for (idx, &temp) in temps.iter().enumerate() {
                                            let _ = db.insert_reading(
                                                &address,
                                                timestamp,
                                                idx,
                                                temp,
                                                ambient,
                                                None,
                                                0,
                                            ).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let _ = peripheral.disconnect().await;
            }
        }
    }
    
    adapter.stop_scan().await?;
    Ok(())
}

fn is_bbq_device_name(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name.starts_with("cA00") || 
    name.starts_with("cA02") || 
    name.starts_with("Y0C") ||
    name_lower.contains("meater") ||
    name_lower.contains("igrill") ||
    name_lower.contains("weber") ||
    name_lower.contains("inkbird") ||
    name_lower.contains("thermoworks")
}
