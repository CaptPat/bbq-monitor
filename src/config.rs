// src/config.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub filters: FilterConfig,
    pub temperature: TemperatureConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub web: Option<WebConfig>,
    pub premium: PremiumConfig,
    pub aws: AwsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub scan_duration: u64,
    pub monitor_duration: u64,
    pub reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub device_prefixes: Vec<String>,
    pub mac_filters: Vec<String>,
    pub min_rssi: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureConfig {
    pub unit: String,
    pub max_internal_temp: f32,
    pub max_ambient_temp: f32,
    pub warning_threshold_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub retention_days: u32,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumConfig {
    pub license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub enabled: bool,
    pub region: String,
    pub thing_name: String,
    pub table_name: String,
    pub sync_interval_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config.toml";
        
        if !Path::new(config_path).exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: DeviceConfig {
                scan_duration: 5,
                monitor_duration: 300,
                reconnect_attempts: 3,
            },
            filters: FilterConfig {
                device_prefixes: vec![
                    "cA00".to_string(),
                    "cA02".to_string(),
                    "Y0C".to_string(),
                ],
                mac_filters: vec![],
                min_rssi: -80,
            },
            temperature: TemperatureConfig {
                unit: "fahrenheit".to_string(),
                max_internal_temp: 200.0,
                max_ambient_temp: 1000.0,
                warning_threshold_percent: 90.0,
            },
            database: DatabaseConfig {
                path: "bbq_monitor.db".to_string(),
                retention_days: 30,
                batch_size: 100,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: true,
                file_path: "bbq_monitor.log".to_string(),
            },
            web: Some(WebConfig {
                enabled: true,
                host: "127.0.0.1".to_string(),
                port: 8080,
            }),
            premium: PremiumConfig {
                license_key: String::new(),
            },
            aws: AwsConfig {
                enabled: false,
                region: "us-east-1".to_string(),
                thing_name: String::new(),
                table_name: "bbq-monitor-readings".to_string(),
                sync_interval_secs: 300,
            },
        }
    }
}
