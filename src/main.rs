// src/main.rs
use anyhow::{Context, Result};
use bbq_monitor::{
    Config, Database, LicenseValidator, MeatStickProtocol, ProbeCapabilities, TemperatureUpdate,
    COMBUSTION_UART_SERVICE, COMBUSTION_UART_RX_CHAR, COMBUSTION_UART_TX_CHAR,
    MEATSTICK_SERVICE, MEATSTICK_CHAR,
};
#[cfg(feature = "aws")]
use bbq_monitor::AwsClient;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Manager;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;
    
    // Initialize logging
    init_logging(&config)?;
    
    info!("🔥 BBQ Monitor v0.1.0 - Starting");
    info!("Configuration loaded from config.toml");
    
    // Validate premium license
    let validator = LicenseValidator::new();
    let license = validator.validate(&config.premium.license_key)?;
    info!("📋 License: {} tier", license.tier);
    
    if !license.features.cloud_sync && config.aws.enabled {
        warn!("⚠️  Cloud sync requires Premium license. Upgrade at https://bbqmonitor.example.com/premium");
    }
    
    // Initialize database
    let db = Arc::new(
        Database::new(&config.database.path)
            .await
            .context("Failed to initialize database")?
    );
    
    // Cleanup old readings (respect license tier for retention)
    let retention_days = if license.features.unlimited_history {
        0 // Keep forever for premium
    } else {
        7 // 7 days for free tier
    };
    db.cleanup_old_readings(retention_days).await?;
    
    // Initialize AWS client if enabled AND licensed
    #[cfg(feature = "aws")]
    let aws_client = if config.aws.enabled && license.features.cloud_sync {
        info!("Initializing AWS cloud sync...");
        let aws_config = bbq_monitor::aws_client::AwsConfig {
            region: config.aws.region.clone(),
            thing_name: config.aws.thing_name.clone(),
            table_name: config.aws.table_name.clone(),
            sync_interval_secs: config.aws.sync_interval_secs,
        };
        
        match AwsClient::new(aws_config, db.clone()).await {
            Ok(client) => {
                info!("✅ AWS cloud sync initialized");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!("⚠️  Failed to initialize AWS client: {}. Continuing without cloud sync.", e);
                None
            }
        }
    } else {
        if config.aws.enabled {
            info!("AWS cloud sync disabled (Premium license required)");
        } else {
            info!("AWS cloud sync disabled in configuration");
        }
        None
    };
    
    #[cfg(not(feature = "aws"))]
    let aws_client: Option<Arc<()>> = {
        if config.aws.enabled {
            warn!("⚠️  AWS cloud sync requested but not compiled in. Rebuild with '--features aws' and Rust 1.88+");
        }
        None
    };
    
    // Create shutdown channel for cleanup
    let (_shutdown_tx, _shutdown_rx) = broadcast::channel::<()>(1);
    
    // Start AWS sync background task if available
    #[cfg(feature = "aws")]
    if let Some(aws) = aws_client.clone() {
        let aws_shutdown = _shutdown_tx.subscribe();
        tokio::spawn(async move {
            aws.start_sync_task(aws_shutdown).await;
        });
    }
    
    // Suppress unused variable warning when aws feature is disabled
    #[cfg(not(feature = "aws"))]
    let _ = aws_client;
    
    // Start web server
    let web_host = config.web.as_ref().map(|w| w.host.as_str()).unwrap_or("127.0.0.1");
    let web_port = config.web.as_ref().map(|w| w.port).unwrap_or(8080);
    
    let (tx, _web_handle) = bbq_monitor::start_server(db.clone(), Arc::new(license), web_host, web_port).await?;
    
    // Initialize BLE manager
    info!("Initializing Bluetooth adapter...");
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    
    if adapters.is_empty() {
        error!("No Bluetooth adapters found");
        return Ok(());
    }
    
    let adapter = &adapters[0];
    info!("Using adapter: {}", adapter.adapter_info().await?);
    
    // Start scanning for devices
    info!("Scanning for BBQ devices for {} seconds...", config.device.scan_duration);
    adapter.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(config.device.scan_duration)).await;
    
    let peripherals = adapter.peripherals().await?;
    let mut connected_devices = Vec::new();
    
    // Find and connect to BBQ devices
    for peripheral in peripherals {
        let properties = match peripheral.properties().await? {
            Some(props) => props,
            None => continue,
        };
        
        let device_address = properties.address.to_string();
        let device_name = properties.local_name.unwrap_or_else(|| "Unknown".to_string());
        let rssi = properties.rssi.unwrap_or(0);
        
        // Apply filters
        if !should_connect(&device_name, &device_address, rssi, &config) {
            continue;
        }
        
        info!("🍖 Found: {} ({}) - RSSI: {}dBm", device_name, device_address, rssi);
        
        match peripheral.connect().await {
            Ok(_) => {
                info!("   ✅ Connected to {}", device_name);
                
                // Discover services
                peripheral.discover_services().await?;
                let services = peripheral.services();
                
                // Detect device capabilities
                let service_uuids: Vec<String> = services.iter()
                    .map(|s| s.uuid.to_string())
                    .collect();
                
                let capabilities = ProbeCapabilities::detect_from_device(
                    &device_name,
                    &device_address,
                    &service_uuids,
                );
                
                info!("   📋 Detected: {:?} with {} sensors", 
                    capabilities.brand, capabilities.sensor_count);
                
                // Save device to database
                db.upsert_device(
                    &device_address,
                    &device_name,
                    &format!("{:?}", capabilities.brand),
                    &capabilities.model,
                    capabilities.sensor_count,
                ).await?;
                
                // Subscribe to notifications
                if setup_notifications(&peripheral, &device_name).await? {
                    connected_devices.push((
                        peripheral.clone(),
                        device_name.clone(),
                        device_address.clone(),
                        capabilities,
                    ));
                }
            }
            Err(e) => {
                warn!("   ❌ Connection failed to {}: {}", device_name, e);
            }
        }
    }
    
    adapter.stop_scan().await?;
    
    if connected_devices.is_empty() {
        warn!("No devices connected for monitoring");
        return Ok(());
    }
    
    info!("🔔 Monitoring {} devices for {} seconds...", 
        connected_devices.len(), config.device.monitor_duration);
    
    // Monitor devices
    let notification_count = monitor_devices(
        adapter,
        &connected_devices,
        &db,
        &config,
        &tx,
    ).await?;
    
    info!("📊 Monitoring complete. Processed {} readings", notification_count);
    
    // Print device summary
    print_device_summary(&connected_devices).await?;
    
    // Disconnect all devices
    for (peripheral, name, _, _) in &connected_devices {
        let _ = peripheral.disconnect().await;
        info!("🔌 Disconnected {}", name);
    }
    
    Ok(())
}

fn init_logging(config: &Config) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            format!("bbq_monitor={},info", config.logging.level).into()
        });
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false);
    
    if config.logging.file_enabled {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.logging.file_path)
            .context("Failed to open log file")?;
        
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Arc::new(file))
            .with_ansi(false);
        
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }
    
    Ok(())
}

fn should_connect(name: &str, address: &str, rssi: i16, config: &Config) -> bool {
    // Check RSSI threshold
    if rssi < config.filters.min_rssi {
        debug!("Skipping {} - RSSI {} below threshold {}", name, rssi, config.filters.min_rssi);
        return false;
    }
    
    // Check MAC address filter
    if !config.filters.mac_filters.is_empty() && !config.filters.mac_filters.iter().any(|filter| address.contains(filter)) {
        return false;
    }
    
    // Check device name prefixes
    if !config.filters.device_prefixes.is_empty() && !config.filters.device_prefixes.iter().any(|prefix| name.starts_with(prefix)) {
        return false;
    }
    
    // General BBQ device detection
    is_bbq_device(name, address)
}

fn is_bbq_device(name: &str, address: &str) -> bool {
    let name_lower = name.to_lowercase();
    
    // MeatStick devices
    if name.starts_with("cA00") || name.starts_with("cA02") || name.starts_with("Y0C") {
        return true;
    }
    
    // Meater devices
    if name_lower.contains("meater") {
        return true;
    }
    
    // Weber devices
    if name_lower.contains("igrill") || name_lower.contains("weber") {
        return true;
    }
    
    // Check MAC address patterns
    if address.starts_with("40:51:6C") {
        return true;
    }
    
    false
}

async fn setup_notifications(
    peripheral: &btleplug::platform::Peripheral,
    _device_name: &str,
) -> Result<bool> {
    let services = peripheral.services();
    let mut subscribed = false;
    
    // MeatStick temperature service
    for service in &services {
        if service.uuid == MEATSTICK_SERVICE {
            debug!("   🌡️  Found MeatStick service");
            
            for characteristic in &service.characteristics {
                if characteristic.uuid == MEATSTICK_CHAR {
                    match peripheral.subscribe(characteristic).await {
                        Ok(_) => {
                            info!("   ✅ Subscribed to temperature notifications");
                            subscribed = true;
                        }
                        Err(e) => {
                            warn!("   ❌ Failed to subscribe: {}", e);
                        }
                    }
                }
            }
        }
        
        // Nordic UART service (for commands)
        if service.uuid == COMBUSTION_UART_SERVICE {
            debug!("   📡 Found Nordic UART service");
            
            for characteristic in &service.characteristics {
                let char_uuid = characteristic.uuid;
                
                // TX characteristic (device sends to us)
                if char_uuid == COMBUSTION_UART_RX_CHAR && peripheral.subscribe(characteristic).await.is_ok() {
                    info!("   📡 Subscribed to Nordic UART notifications");
                    subscribed = true;
                }
                
                // RX characteristic (we send to device)
                if char_uuid == COMBUSTION_UART_TX_CHAR {
                    debug!("   📤 Sending wake-up commands...");
                    
                    let commands: Vec<&[u8]> = vec![
                        b"temp\r\n",
                        b"status\r\n",
                    ];
                    
                    for cmd in &commands {
                        let _ = peripheral.write(characteristic, cmd, WriteType::WithoutResponse).await;
                        time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }
    
    Ok(subscribed)
}

async fn monitor_devices(
    adapter: &btleplug::platform::Adapter,
    connected_devices: &[(btleplug::platform::Peripheral, String, String, ProbeCapabilities)],
    db: &Database,
    config: &Config,
    tx: &tokio::sync::broadcast::Sender<TemperatureUpdate>,
) -> Result<u32> {
    let mut events = adapter.events().await?;
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(config.device.monitor_duration);
    let mut notification_count = 0;
    
    while start_time.elapsed() < timeout {
        tokio::select! {
            Some(event) = events.next() => {
                match event {
                    CentralEvent::DeviceUpdated(id) => {
                        for (peripheral, name, address, capabilities) in connected_devices {
                            if peripheral.id() == id {
                                if let Ok(reading_count) = process_device_update(
                                    peripheral, name, address, capabilities, db, tx
                                ).await {
                                    notification_count += reading_count;
                                }
                            }
                        }
                    }
                    
                    CentralEvent::DeviceDisconnected(id) => {
                        for (peripheral, name, _, _) in connected_devices {
                            if peripheral.id() == id {
                                warn!("🔌 Device {} disconnected", name);
                            }
                        }
                    }
                    
                    _ => {
                        debug!("BLE Event: {:?}", event);
                    }
                }
            }
            
            _ = time::sleep(Duration::from_secs(5)) => {
                // Periodic polling for devices that don't send notifications
                for (peripheral, name, address, capabilities) in connected_devices {
                    if peripheral.is_connected().await.unwrap_or(false) {
                        if let Ok(count) = poll_device_readings(
                            peripheral, name, address, capabilities, db, tx
                        ).await {
                            notification_count += count;
                        }
                    }
                }
            }
        }
    }
    
    Ok(notification_count)
}

async fn process_device_update(
    peripheral: &btleplug::platform::Peripheral,
    name: &str,
    address: &str,
    capabilities: &ProbeCapabilities,
    db: &Database,
    tx: &tokio::sync::broadcast::Sender<TemperatureUpdate>,
) -> Result<u32> {
    let mut count = 0;
    
    peripheral.discover_services().await?;
    let services = peripheral.services();
    
    for service in &services {
        if service.uuid == MEATSTICK_SERVICE {
            for characteristic in &service.characteristics {
                if characteristic.uuid == MEATSTICK_CHAR {
                    if let Ok(data) = peripheral.read(characteristic).await {
                        if !data.is_empty() {
                            count += process_temperature_data(&data, name, address, capabilities, db, tx).await?;
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}

async fn poll_device_readings(
    peripheral: &btleplug::platform::Peripheral,
    name: &str,
    address: &str,
    capabilities: &ProbeCapabilities,
    db: &Database,
    tx: &tokio::sync::broadcast::Sender<TemperatureUpdate>,
) -> Result<u32> {
    let services = peripheral.services();
    let mut count = 0;
    
    for service in &services {
        if service.uuid == MEATSTICK_SERVICE {
            for characteristic in &service.characteristics {
                if characteristic.uuid == MEATSTICK_CHAR {
                    if let Ok(data) = peripheral.read(characteristic).await {
                        if !data.is_empty() {
                            count += process_temperature_data(&data, name, address, capabilities, db, tx).await?;
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}

async fn process_temperature_data(
    data: &[u8],
    name: &str,
    address: &str,
    _capabilities: &ProbeCapabilities,
    db: &Database,
    tx: &tokio::sync::broadcast::Sender<TemperatureUpdate>,
) -> Result<u32> {
    match MeatStickProtocol::parse_temperature_data(data) {
        Ok(temperatures) => {
            let timestamp = Utc::now();
            let ambient_temp = MeatStickProtocol::get_ambient_temp(&temperatures);
            let internal_temp = MeatStickProtocol::get_internal_temp(&temperatures);
            
            info!("🌡️  {} - Internal: {:.1}°F, Ambient: {:.1}°F, Sensors: {}", 
                name,
                internal_temp.unwrap_or(0.0),
                ambient_temp.unwrap_or(0.0),
                temperatures.len()
            );
            
            // Store each sensor reading
            let mut count = 0;
            for (i, &temp) in temperatures.iter().enumerate() {
                db.insert_reading(
                    address,
                    timestamp,
                    i,
                    temp,
                    ambient_temp,
                    None, // battery level not available yet
                    0,    // signal strength from properties
                ).await?;
                
                // Broadcast update to web clients
                let update = TemperatureUpdate {
                    device_address: address.to_string(),
                    device_name: name.to_string(),
                    timestamp,
                    sensor_index: i,
                    temperature: temp,
                    ambient_temp,
                    battery_level: None,
                    signal_strength: 0,
                };
                let _ = tx.send(update);
                
                count += 1;
            }
            
            Ok(count)
        }
        Err(e) => {
            debug!("Failed to parse temperature data from {}: {}", name, e);
            debug!("Raw data: {:02X?}", data);
            Ok(0)
        }
    }
}

async fn print_device_summary(
    devices: &[(btleplug::platform::Peripheral, String, String, ProbeCapabilities)],
) -> Result<()> {
    info!("🔍 DEVICE SUMMARY:");
    
    for (peripheral, name, address, capabilities) in devices {
        let services = peripheral.services();
        let mut info_str = format!("  {} ({}) - {:?}", name, address, capabilities.brand);
        
        // Try to read serial number
        for service in &services {
            if service.uuid.to_string() == "0000180a-0000-1000-8000-00805f9b34fb" {
                for characteristic in &service.characteristics {
                    if characteristic.uuid.to_string() == "00002a25-0000-1000-8000-00805f9b34fb" {
                        if let Ok(data) = peripheral.read(characteristic).await {
                            let serial = String::from_utf8_lossy(&data);
                            info_str.push_str(&format!(" S/N: {}", serial));
                        }
                    }
                }
            }
        }
        
        info!("{}", info_str);
    }
    
    Ok(())
}
