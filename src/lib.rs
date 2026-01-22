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
