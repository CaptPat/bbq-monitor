// src/device_capabilities.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import service UUIDs from protocol module
use crate::protocol::{MEATSTICK_SERVICE, COMBUSTION_UART_SERVICE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeBrand {
    MeatStickV1,
    MeatStickV2,
    MeatStickV,      // Latest 6-sensor model
    MeaterOriginal,
    MeaterPlus,
    MeaterBlock,
    WeberIGrill,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCapabilities {
    pub brand: ProbeBrand,
    pub model: String,
    pub sensor_count: usize,
    pub max_ambient_temp_f: f32,
    pub max_internal_temp_f: f32,
    pub battery_life_hours: Option<u32>,
    pub range_feet: Option<u32>,
    pub has_repeater: bool,
    pub service_uuids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyStatus {
    Safe,
    WarningAmbientHigh,
    WarningInternalHigh,
    DangerousAmbient,
    DangerousInternal,
    DeviceOffline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFreshness {
    Live(u64),           // Age in seconds
    Recent(u64),         // Lost connection, age since last reading
    Stale(u64),          // Old data, decreasing reliability
    Dead(u64),           // Too old to be useful
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeReading {
    pub probe_id: String,
    pub device_address: String,
    pub timestamp: DateTime<Utc>,
    pub temperatures: Vec<f32>,  // Multiple sensors for MeatStick V
    pub ambient_temp: Option<f32>,
    pub battery_level: Option<u8>,
    pub signal_strength: i16,
    pub freshness: DataFreshness,
    pub confidence: f32,         // 1.0 = live, decays over time
    pub safety_status: SafetyStatus,
}

impl ProbeCapabilities {
    pub fn detect_from_device(device_name: &str, _mac_address: &str, services: &[String]) -> Self {
        // Convert service strings to lowercase for comparison
        let has_meatstick_service = services.iter().any(|s| {
            s.to_lowercase() == MEATSTICK_SERVICE.to_string().to_lowercase()
        });
        let has_uart_service = services.iter().any(|s| {
            s.to_lowercase() == COMBUSTION_UART_SERVICE.to_string().to_lowercase()
        });
        
        match device_name {
            // MeatStick device detection
            name if name.starts_with("cA00") => {
                if has_meatstick_service || has_uart_service {
                    // MeatStick V has 6 sensors (or 8 for Combustion models)
                    Self {
                        brand: ProbeBrand::MeatStickV,
                        model: name.to_string(),
                        sensor_count: 8, // Updated to 8 for Combustion protocol
                        max_ambient_temp_f: 1000.0,
                        max_internal_temp_f: 200.0,
                        battery_life_hours: Some(24),
                        range_feet: Some(650),
                        has_repeater: false,
                        service_uuids: services.to_vec(),
                    }
                } else {
                    // Older MeatStick models
                    Self {
                        brand: ProbeBrand::MeatStickV1,
                        model: name.to_string(),
                        sensor_count: 2,
                        max_ambient_temp_f: 600.0,
                        max_internal_temp_f: 200.0,
                        battery_life_hours: Some(8),
                        range_feet: Some(165),
                        has_repeater: false,
                        service_uuids: services.to_vec(),
                    }
                }
            }
            
            // MeatStick base stations
            name if name.starts_with("cA02") => {
                Self {
                    brand: ProbeBrand::MeatStickV,
                    model: format!("{}_BASE", name),
                    sensor_count: 0,
                    max_ambient_temp_f: 0.0,
                    max_internal_temp_f: 0.0,
                    battery_life_hours: None, // Plugged in
                    range_feet: Some(650),
                    has_repeater: true,
                    service_uuids: services.to_vec(),
                }
            }
            
            // Meater devices
            name if name.to_uppercase().contains("MEATER") => {
                if name.contains("BLOCK") || name.contains("Block") {
                    Self {
                        brand: ProbeBrand::MeaterBlock,
                        model: name.to_string(),
                        sensor_count: 0, // Base station for up to 4 probes
                        max_ambient_temp_f: 0.0,
                        max_internal_temp_f: 0.0,
                        battery_life_hours: None,
                        range_feet: Some(165),
                        has_repeater: true,
                        service_uuids: services.to_vec(),
                    }
                } else if name.contains("PLUS") || name.contains("Plus") {
                    Self {
                        brand: ProbeBrand::MeaterPlus,
                        model: name.to_string(),
                        sensor_count: 2,
                        max_ambient_temp_f: 527.0,
                        max_internal_temp_f: 212.0,
                        battery_life_hours: Some(24),
                        range_feet: Some(165),
                        has_repeater: false,
                        service_uuids: services.to_vec(),
                    }
                } else {
                    Self {
                        brand: ProbeBrand::MeaterOriginal,
                        model: name.to_string(),
                        sensor_count: 2,
                        max_ambient_temp_f: 527.0,
                        max_internal_temp_f: 212.0,
                        battery_life_hours: Some(8),
                        range_feet: Some(33),
                        has_repeater: false,
                        service_uuids: services.to_vec(),
                    }
                }
            }
            
            _ => Self {
                brand: ProbeBrand::Unknown(device_name.to_string()),
                model: device_name.to_string(),
                sensor_count: 1,
                max_ambient_temp_f: 500.0, // Conservative default
                max_internal_temp_f: 200.0,
                battery_life_hours: Some(8),
                range_feet: Some(30),
                has_repeater: false,
                service_uuids: services.to_vec(),
            }
        }
    }
}

impl ProbeReading {
    pub fn new(probe_id: String, device_address: String, capabilities: &ProbeCapabilities) -> Self {
        Self {
            probe_id,
            device_address,
            timestamp: Utc::now(),
            temperatures: vec![0.0; capabilities.sensor_count.max(1)],
            ambient_temp: None,
            battery_level: None,
            signal_strength: 0,
            freshness: DataFreshness::Live(0),
            confidence: 1.0,
            safety_status: SafetyStatus::DeviceOffline,
        }
    }
    
    pub fn update_safety_status(&mut self, capabilities: &ProbeCapabilities) {
        // Check ambient temperature safety
        if let Some(ambient) = self.ambient_temp {
            if ambient > capabilities.max_ambient_temp_f {
                self.safety_status = SafetyStatus::DangerousAmbient;
                return;
            } else if ambient > capabilities.max_ambient_temp_f * 0.9 {
                self.safety_status = SafetyStatus::WarningAmbientHigh;
            }
        }
        
        // Check internal temperature safety
        for &temp in &self.temperatures {
            if temp > capabilities.max_internal_temp_f {
                self.safety_status = SafetyStatus::DangerousInternal;
                return;
            } else if temp > capabilities.max_internal_temp_f * 0.9 {
                self.safety_status = SafetyStatus::WarningInternalHigh;
            }
        }
        
        // Update confidence based on age
        self.update_confidence();
        
        if self.confidence > 0.1 {
            self.safety_status = SafetyStatus::Safe;
        } else {
            self.safety_status = SafetyStatus::DeviceOffline;
        }
    }
    
    pub fn update_confidence(&mut self) {
        let age_seconds = (Utc::now() - self.timestamp).num_seconds() as u64;
        
        self.confidence = match age_seconds {
            0..=30 => 1.0,           // Live data: 100% confidence
            31..=120 => 0.8,         // Recent: 80% confidence  
            121..=300 => 0.5,        // Stale: 50% confidence
            301..=600 => 0.2,        // Very stale: 20% confidence
            _ => 0.0,                // Dead: 0% confidence
        };
        
        self.freshness = match age_seconds {
            0..=30 => DataFreshness::Live(age_seconds),
            31..=300 => DataFreshness::Recent(age_seconds),
            301..=600 => DataFreshness::Stale(age_seconds),
            _ => DataFreshness::Dead(age_seconds),
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub devices: HashMap<String, ProbeCapabilities>,
    pub readings: HashMap<String, ProbeReading>,
    pub signal_map: HashMap<String, Vec<(DateTime<Utc>, i16)>>, // RSSI history
    pub last_update: DateTime<Utc>,
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            readings: HashMap::new(),
            signal_map: HashMap::new(),
            last_update: Utc::now(),
        }
    }
    
    pub fn add_device(&mut self, device_address: String, capabilities: ProbeCapabilities) {
        self.devices.insert(device_address, capabilities);
        self.last_update = Utc::now();
    }
    
    pub fn update_reading(&mut self, reading: ProbeReading) {
        // Update signal strength history
        let rssi_history = self.signal_map
            .entry(reading.device_address.clone())
            .or_default();
        rssi_history.push((reading.timestamp, reading.signal_strength));
        
        // Keep only last 100 readings
        if rssi_history.len() > 100 {
            rssi_history.remove(0);
        }
        
        self.readings.insert(reading.probe_id.clone(), reading);
        self.last_update = Utc::now();
    }
    
    pub fn get_active_probes(&self) -> Vec<&ProbeReading> {
        self.readings.values()
            .filter(|reading| reading.confidence > 0.3)
            .collect()
    }
    
    pub fn get_safety_alerts(&self) -> Vec<&ProbeReading> {
        self.readings.values()
            .filter(|reading| matches!(reading.safety_status, SafetyStatus::DangerousAmbient | SafetyStatus::DangerousInternal))
            .collect()
    }
}