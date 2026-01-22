// src/database.rs
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::info;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &str) -> Result<Self> {
        let connection_string = format!("sqlite:{}", database_path);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .context("Failed to connect to database")?;
        
        let db = Self { pool };
        db.initialize().await?;
        
        info!("Database initialized at {}", database_path);
        Ok(db)
    }
    
    async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                device_address TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                brand TEXT NOT NULL,
                model TEXT NOT NULL,
                sensor_count INTEGER NOT NULL,
                first_seen DATETIME NOT NULL,
                last_seen DATETIME NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create devices table")?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS readings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_address TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                sensor_index INTEGER NOT NULL,
                temperature REAL NOT NULL,
                ambient_temp REAL,
                battery_level INTEGER,
                signal_strength INTEGER NOT NULL,
                FOREIGN KEY (device_address) REFERENCES devices(device_address)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create readings table")?;
        
        // Create index for faster queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_readings_timestamp 
            ON readings(timestamp DESC)
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create timestamp index")?;
        
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_readings_device 
            ON readings(device_address, timestamp DESC)
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create device index")?;
        
        Ok(())
    }
    
    pub async fn upsert_device(
        &self,
        device_address: &str,
        device_name: &str,
        brand: &str,
        model: &str,
        sensor_count: usize,
    ) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO devices (device_address, device_name, brand, model, sensor_count, first_seen, last_seen)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(device_address) DO UPDATE SET
                device_name = excluded.device_name,
                model = excluded.model,
                sensor_count = excluded.sensor_count,
                last_seen = excluded.last_seen
            "#
        )
        .bind(device_address)
        .bind(device_name)
        .bind(brand)
        .bind(model)
        .bind(sensor_count as i64)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to upsert device")?;
        
        Ok(())
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_reading(
        &self,
        device_address: &str,
        timestamp: DateTime<Utc>,
        sensor_index: usize,
        temperature: f32,
        ambient_temp: Option<f32>,
        battery_level: Option<u8>,
        signal_strength: i16,
    ) -> Result<()> {
        self.insert_reading_impl(
            device_address,
            timestamp,
            sensor_index,
            temperature,
            ambient_temp,
            battery_level,
            signal_strength,
        ).await
    }
    
    #[allow(clippy::too_many_arguments)]
    async fn insert_reading_impl(
        &self,
        device_address: &str,
        timestamp: DateTime<Utc>,
        sensor_index: usize,
        temperature: f32,
        ambient_temp: Option<f32>,
        battery_level: Option<u8>,
        signal_strength: i16,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO readings (device_address, timestamp, sensor_index, temperature, 
                                ambient_temp, battery_level, signal_strength)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(device_address)
        .bind(timestamp)
        .bind(sensor_index as i64)
        .bind(temperature)
        .bind(ambient_temp)
        .bind(battery_level.map(|b| b as i64))
        .bind(signal_strength as i64)
        .execute(&self.pool)
        .await
        .context("Failed to insert reading")?;
        
        Ok(())
    }
    
    pub async fn cleanup_old_readings(&self, retention_days: u32) -> Result<u64> {
        if retention_days == 0 {
            return Ok(0);
        }
        
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let result = sqlx::query(
            r#"
            DELETE FROM readings WHERE timestamp < ?
            "#
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .context("Failed to cleanup old readings")?;
        
        let rows_deleted = result.rows_affected();
        if rows_deleted > 0 {
            info!("Cleaned up {} old readings", rows_deleted);
        }
        
        Ok(rows_deleted)
    }
    
    pub async fn get_latest_reading(&self, device_address: &str) -> Result<ReadingRecord> {
        let result = sqlx::query_as::<_, ReadingRecord>(
            r#"
            SELECT device_address, timestamp, sensor_index, temperature, 
                   ambient_temp, battery_level, signal_strength
            FROM readings
            WHERE device_address = ?
            ORDER BY timestamp DESC
            LIMIT 1
            "#
        )
        .bind(device_address)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch latest reading")?;
        
        Ok(result)
    }
    
    /// Get all devices
    pub async fn get_all_devices(&self) -> Result<Vec<DeviceRecord>> {
        let devices = sqlx::query_as::<_, DeviceRecord>(
            r#"
            SELECT device_address, device_name, brand, model, sensor_count, 
                   first_seen, last_seen
            FROM devices
            ORDER BY last_seen DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch devices")?;
        
        Ok(devices)
    }
    
    /// Get a specific device
    pub async fn get_device(&self, device_address: &str) -> Result<DeviceRecord> {
        let device = sqlx::query_as::<_, DeviceRecord>(
            r#"
            SELECT device_address, device_name, brand, model, sensor_count,
                   first_seen, last_seen
            FROM devices
            WHERE device_address = ?
            "#
        )
        .bind(device_address)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch device")?;
        
        Ok(device)
    }
    
    /// Get readings since a specific time
    pub async fn get_readings_since(
        &self,
        device_address: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReadingRecord>> {
        let readings = sqlx::query_as::<_, ReadingRecord>(
            r#"
            SELECT device_address, timestamp, sensor_index, temperature,
                   ambient_temp, battery_level, signal_strength
            FROM readings
            WHERE device_address = ? AND timestamp >= ?
            ORDER BY timestamp ASC
            "#
        )
        .bind(device_address)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch readings")?;
        
        Ok(readings)
    }
    
    /// Get recent readings for a device
    pub async fn get_device_readings(
        &self,
        device_address: &str,
        limit: usize,
    ) -> Result<Vec<ReadingRecord>> {
        let query = if limit == 0 {
            sqlx::query_as::<_, ReadingRecord>(
                r#"
                SELECT device_address, timestamp, sensor_index, temperature,
                       ambient_temp, battery_level, signal_strength
                FROM readings
                WHERE device_address = ?
                ORDER BY timestamp DESC
                "#
            )
            .bind(device_address)
        } else {
            sqlx::query_as::<_, ReadingRecord>(
                r#"
                SELECT device_address, timestamp, sensor_index, temperature,
                       ambient_temp, battery_level, signal_strength
                FROM readings
                WHERE device_address = ?
                ORDER BY timestamp DESC
                LIMIT ?
                "#
            )
            .bind(device_address)
            .bind(limit as i64)
        };
        
        let readings = query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch device readings")?;
        
        Ok(readings)
    }
    
    /// Get readings within a time range
    pub async fn get_readings_in_range(
        &self,
        device_address: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ReadingRecord>> {
        let readings = sqlx::query_as::<_, ReadingRecord>(
            r#"
            SELECT device_address, timestamp, sensor_index, temperature,
                   ambient_temp, battery_level, signal_strength
            FROM readings
            WHERE device_address = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp ASC
            "#
        )
        .bind(device_address)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch readings in range")?;
        
        Ok(readings)
    }
}

/// Device record from database
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct DeviceRecord {
    pub device_address: String,
    pub device_name: String,
    pub brand: String,
    pub model: String,
    pub sensor_count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Reading record from database
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ReadingRecord {
    pub device_address: String,
    pub timestamp: DateTime<Utc>,
    pub sensor_index: i64,
    pub temperature: f32,
    pub ambient_temp: Option<f32>,
    pub battery_level: Option<u8>,
    pub signal_strength: i16,
}

