use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoClient};
use aws_sdk_iotdataplane::Client as IoTDataClient;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::database::{Database, ReadingRecord};

/// Configuration for AWS IoT and DynamoDB
#[derive(Debug, Clone)]
pub struct AwsConfig {
    pub region: String,
    pub thing_name: String,
    pub table_name: String,
    pub sync_interval_secs: u64,
}

/// Temperature reading for cloud sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudReading {
    pub device_address: String,
    pub device_name: String,
    pub temperature: f64,
    pub ambient_temp: Option<f64>,
    pub battery_level: Option<u8>,
    pub signal_strength: i16,
    pub timestamp: DateTime<Utc>,
    pub source: String, // "local" or "cloud"
}

/// AWS client for IoT and DynamoDB operations
pub struct AwsClient {
    iot_data: IoTDataClient,
    dynamo: DynamoClient,
    config: AwsConfig,
    database: Arc<Database>,
}

impl AwsClient {
    /// Create a new AWS client
    pub async fn new(config: AwsConfig, database: Arc<Database>) -> Result<Self> {
        info!("Initializing AWS client for region: {}", config.region);
        
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(config.region.clone()))
            .load()
            .await;

        let iot_data = IoTDataClient::new(&sdk_config);
        let dynamo = DynamoClient::new(&sdk_config);

        info!("AWS client initialized successfully");
        
        Ok(Self {
            iot_data,
            dynamo,
            config,
            database,
        })
    }

    /// Publish a reading to IoT Core
    pub async fn publish_reading(&self, reading: &CloudReading) -> Result<()> {
        let topic = format!("bbq-monitor/{}/readings", self.config.thing_name);
        let payload = serde_json::to_vec(reading)
            .context("Failed to serialize reading")?;

        debug!("Publishing reading to topic: {}", topic);
        
        self.iot_data
            .publish()
            .topic(&topic)
            .payload(aws_sdk_iotdataplane::primitives::Blob::new(payload))
            .qos(1)
            .send()
            .await
            .context("Failed to publish to IoT Core")?;

        debug!("Successfully published reading to IoT Core");
        Ok(())
    }

    /// Store a reading in DynamoDB
    pub async fn store_reading(&self, reading: &CloudReading) -> Result<()> {
        let mut item = HashMap::new();
        
        // Composite key: device_address#timestamp
        let sort_key = format!("{}#{}", 
            reading.device_address, 
            reading.timestamp.timestamp_millis()
        );
        
        item.insert(
            "device_address".to_string(),
            AttributeValue::S(reading.device_address.clone()),
        );
        item.insert(
            "timestamp_key".to_string(),
            AttributeValue::S(sort_key),
        );
        item.insert(
            "device_name".to_string(),
            AttributeValue::S(reading.device_name.clone()),
        );
        item.insert(
            "temperature".to_string(),
            AttributeValue::N(reading.temperature.to_string()),
        );
        item.insert(
            "signal_strength".to_string(),
            AttributeValue::N(reading.signal_strength.to_string()),
        );
        item.insert(
            "timestamp".to_string(),
            AttributeValue::S(reading.timestamp.to_rfc3339()),
        );
        item.insert(
            "source".to_string(),
            AttributeValue::S(reading.source.clone()),
        );

        if let Some(ambient) = reading.ambient_temp {
            item.insert(
                "ambient_temp".to_string(),
                AttributeValue::N(ambient.to_string()),
            );
        }

        if let Some(battery) = reading.battery_level {
            item.insert(
                "battery_level".to_string(),
                AttributeValue::N(battery.to_string()),
            );
        }

        debug!("Storing reading in DynamoDB table: {}", self.config.table_name);
        
        self.dynamo
            .put_item()
            .table_name(&self.config.table_name)
            .set_item(Some(item))
            .send()
            .await
            .context("Failed to store reading in DynamoDB")?;

        debug!("Successfully stored reading in DynamoDB");
        Ok(())
    }

    /// Query recent readings from DynamoDB for a device
    pub async fn query_device_readings(
        &self,
        device_address: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<CloudReading>> {
        let since_key = format!("{}#{}", device_address, since.timestamp_millis());
        
        debug!(
            "Querying DynamoDB for device {} since {}", 
            device_address, 
            since.to_rfc3339()
        );

        let result = self.dynamo
            .query()
            .table_name(&self.config.table_name)
            .key_condition_expression(
                "device_address = :addr AND timestamp_key >= :since"
            )
            .expression_attribute_values(
                ":addr",
                AttributeValue::S(device_address.to_string()),
            )
            .expression_attribute_values(
                ":since",
                AttributeValue::S(since_key),
            )
            .send()
            .await
            .context("Failed to query DynamoDB")?;

        let mut readings = Vec::new();
        
        if let Some(items) = result.items {
            for item in items {
                if let Ok(reading) = self.parse_dynamo_item(item) {
                    readings.push(reading);
                }
            }
        }

        debug!("Retrieved {} readings from DynamoDB", readings.len());
        Ok(readings)
    }

    /// Parse a DynamoDB item into a CloudReading
    fn parse_dynamo_item(&self, item: HashMap<String, AttributeValue>) -> Result<CloudReading> {
        let device_address = item
            .get("device_address")
            .and_then(|v| v.as_s().ok())
            .context("Missing device_address")?
            .to_string();

        let device_name = item
            .get("device_name")
            .and_then(|v| v.as_s().ok())
            .context("Missing device_name")?
            .to_string();

        let temperature = item
            .get("temperature")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .context("Missing or invalid temperature")?;

        let signal_strength = item
            .get("signal_strength")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<i16>().ok())
            .context("Missing or invalid signal_strength")?;

        let timestamp_str = item
            .get("timestamp")
            .and_then(|v| v.as_s().ok())
            .context("Missing timestamp")?;

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .context("Invalid timestamp format")?
            .with_timezone(&Utc);

        let source = item
            .get("source")
            .and_then(|v| v.as_s().ok())
            .unwrap_or("cloud")
            .to_string();

        let ambient_temp = item
            .get("ambient_temp")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<f64>().ok());

        let battery_level = item
            .get("battery_level")
            .and_then(|v| v.as_n().ok())
            .and_then(|s| s.parse::<u8>().ok());

        Ok(CloudReading {
            device_address,
            device_name,
            temperature,
            ambient_temp,
            battery_level,
            signal_strength,
            timestamp,
            source,
        })
    }

    /// Sync local readings to cloud
    pub async fn sync_to_cloud(&self, since: DateTime<Utc>) -> Result<usize> {
        info!("Starting sync to cloud since {}", since.to_rfc3339());
        
        let devices = self.database.get_all_devices().await?;
        let mut synced_count = 0;

        for device in devices {
            let readings = self.database
                .get_readings_since(&device.address, since)
                .await?;

            debug!(
                "Syncing {} readings for device {}", 
                readings.len(), 
                device.address
            );

            for reading in readings {
                let cloud_reading = CloudReading {
                    device_address: reading.device_address.clone(),
                    device_name: device.name.clone(),
                    temperature: reading.temperature,
                    ambient_temp: reading.ambient_temp,
                    battery_level: reading.battery_level,
                    signal_strength: reading.signal_strength,
                    timestamp: reading.timestamp,
                    source: "local".to_string(),
                };

                // Store in DynamoDB
                if let Err(e) = self.store_reading(&cloud_reading).await {
                    error!("Failed to store reading in DynamoDB: {}", e);
                    continue;
                }

                // Publish to IoT Core
                if let Err(e) = self.publish_reading(&cloud_reading).await {
                    error!("Failed to publish reading to IoT Core: {}", e);
                    continue;
                }

                synced_count += 1;
            }
        }

        info!("Synced {} readings to cloud", synced_count);
        Ok(synced_count)
    }

    /// Sync cloud readings to local database
    pub async fn sync_from_cloud(&self, since: DateTime<Utc>) -> Result<usize> {
        info!("Starting sync from cloud since {}", since.to_rfc3339());
        
        let devices = self.database.get_all_devices().await?;
        let mut synced_count = 0;

        for device in devices {
            let cloud_readings = self
                .query_device_readings(&device.address, since)
                .await?;

            debug!(
                "Retrieved {} cloud readings for device {}", 
                cloud_readings.len(), 
                device.address
            );

            for reading in cloud_readings {
                // Skip if this reading originated from this instance
                if reading.source == "local" {
                    continue;
                }

                // Check if we already have this reading
                let existing = self.database
                    .get_readings_since(&reading.device_address, reading.timestamp)
                    .await?;

                let has_reading = existing.iter().any(|r| {
                    (r.timestamp - reading.timestamp).num_seconds().abs() < 5
                });

                if has_reading {
                    continue;
                }

                // Insert cloud reading into local database
                self.database
                    .insert_reading(
                        &reading.device_address,
                        reading.temperature,
                        reading.ambient_temp,
                        reading.battery_level,
                        reading.signal_strength,
                        reading.timestamp,
                    )
                    .await?;

                synced_count += 1;
            }
        }

        info!("Synced {} readings from cloud", synced_count);
        Ok(synced_count)
    }

    /// Start background sync task
    pub async fn start_sync_task(
        self: Arc<Self>,
        mut shutdown: broadcast::Receiver<()>,
    ) {
        info!(
            "Starting background sync task with interval: {}s",
            self.config.sync_interval_secs
        );

        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.sync_interval_secs)
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let since = Utc::now() - chrono::Duration::hours(1);
                    
                    // Sync to cloud
                    match self.sync_to_cloud(since).await {
                        Ok(count) => debug!("Synced {} readings to cloud", count),
                        Err(e) => error!("Cloud sync to failed: {}", e),
                    }

                    // Sync from cloud
                    match self.sync_from_cloud(since).await {
                        Ok(count) => debug!("Synced {} readings from cloud", count),
                        Err(e) => error!("Cloud sync from failed: {}", e),
                    }
                }
                _ = shutdown.recv() => {
                    info!("Shutting down background sync task");
                    break;
                }
            }
        }
    }
}
