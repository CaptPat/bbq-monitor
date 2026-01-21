// src/web_server.rs
use anyhow::Result;
use axum::{
    extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};

use crate::{Database, License};

/// Web server state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub tx: broadcast::Sender<TemperatureUpdate>,
    pub license: Arc<License>,
}

/// Real-time temperature update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureUpdate {
    pub device_address: String,
    pub device_name: String,
    pub timestamp: DateTime<Utc>,
    pub sensor_index: usize,
    pub temperature: f32,
    pub ambient_temp: Option<f32>,
    pub battery_level: Option<u8>,
    pub signal_strength: i16,
}

/// Device summary for API
#[derive(Debug, Serialize)]
pub struct DeviceSummary {
    pub device_address: String,
    pub device_name: String,
    pub brand: String,
    pub model: String,
    pub sensor_count: i64,
    pub last_seen: DateTime<Utc>,
    pub latest_reading: Option<ReadingSummary>,
}

/// Reading summary for API
#[derive(Debug, Serialize)]
pub struct ReadingSummary {
    pub timestamp: DateTime<Utc>,
    pub temperature: f32,
    pub ambient_temp: Option<f32>,
    pub battery_level: Option<u8>,
    pub signal_strength: i16,
}

/// Historical data query parameters
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

/// Start the web server
pub async fn start_server(
    db: Arc<Database>,
    license: Arc<License>,
    host: &str,
    port: u16,
) -> Result<(broadcast::Sender<TemperatureUpdate>, tokio::task::JoinHandle<()>)> {
    let (tx, _rx) = broadcast::channel(100);
    
    let state = AppState {
        db: db.clone(),
        tx: tx.clone(),
        license: license.clone(),
    };
    
    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/devices", get(list_devices))
        .route("/api/devices/:address", get(device_details))
        .route("/api/devices/:address/history", get(device_history))
        .route("/api/premium/status", get(premium_status))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .with_state(state);
    
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🌐 Web dashboard starting at http://{}", addr);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Web server error: {}", e);
        }
    });
    
    Ok((tx, handle))
}

/// Serve the main dashboard HTML
async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// List all devices
async fn list_devices(State(state): State<AppState>) -> Result<Json<Vec<DeviceSummary>>, AppError> {
    let devices = state.db.get_all_devices().await?;
    
    let mut summaries = Vec::new();
    for device in devices {
        let latest = state.db.get_latest_reading(&device.device_address).await.ok();
        
        summaries.push(DeviceSummary {
            device_address: device.device_address.clone(),
            device_name: device.device_name,
            brand: device.brand,
            model: device.model,
            sensor_count: device.sensor_count,
            last_seen: device.last_seen,
            latest_reading: latest.map(|r| ReadingSummary {
                timestamp: r.timestamp,
                temperature: r.temperature,
                ambient_temp: r.ambient_temp,
                battery_level: r.battery_level,
                signal_strength: r.signal_strength,
            }),
        });
    }
    
    Ok(Json(summaries))
}

/// Get details for a specific device
async fn device_details(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<DeviceSummary>, AppError> {
    let device = state.db.get_device(&address).await?;
    let latest = state.db.get_latest_reading(&address).await.ok();
    
    Ok(Json(DeviceSummary {
        device_address: device.device_address.clone(),
        device_name: device.device_name,
        brand: device.brand,
        model: device.model,
        sensor_count: device.sensor_count,
        last_seen: device.last_seen,
        latest_reading: latest.map(|r| ReadingSummary {
            timestamp: r.timestamp,
            temperature: r.temperature,
            ambient_temp: r.ambient_temp,
            battery_level: r.battery_level,
            signal_strength: r.signal_strength,
        }),
    }))
}

/// Get historical readings for a device
async fn device_history(
    State(state): State<AppState>,
    Path(address): Path<String>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> Result<Json<Vec<ReadingSummary>>, AppError> {
    let cutoff = Utc::now() - chrono::Duration::hours(query.hours as i64);
    let readings = state.db.get_readings_since(&address, cutoff).await?;
    
    let summaries: Vec<ReadingSummary> = readings
        .into_iter()
        .map(|r| ReadingSummary {
            timestamp: r.timestamp,
            temperature: r.temperature,
            ambient_temp: r.ambient_temp,
            battery_level: r.battery_level,
            signal_strength: r.signal_strength,
        })
        .collect();
    
    Ok(Json(summaries))
}

/// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    
    debug!("WebSocket client connected");
    
    // Send initial device list
    if let Ok(devices) = state.db.get_all_devices().await {
        for device in devices {
            if let Ok(latest) = state.db.get_latest_reading(&device.device_address).await {
                let update = TemperatureUpdate {
                    device_address: device.device_address.clone(),
                    device_name: device.device_name,
                    timestamp: latest.timestamp,
                    sensor_index: latest.sensor_index as usize,
                    temperature: latest.temperature,
                    ambient_temp: latest.ambient_temp,
                    battery_level: latest.battery_level,
                    signal_strength: latest.signal_strength,
                };
                
                if let Ok(json) = serde_json::to_string(&update) {
                    let _ = socket.send(Message::Text(json)).await;
                }
            }
        }
    }
    
    // Stream real-time updates
    while let Ok(update) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&update) {
            if socket.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }
    
    debug!("WebSocket client disconnected");
}

/// Premium status endpoint
async fn premium_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let license = &state.license;
    
    let response = serde_json::json!({
        "tier": license.tier,
        "features": {
            "cloud_sync": license.features.cloud_sync,
            "unlimited_history": license.features.unlimited_history,
            "cook_profiles": license.features.cook_profiles,
            "remote_access": license.features.remote_access,
            "advanced_analytics": license.features.advanced_analytics,
            "alerts": license.features.alerts,
        },
        "is_valid": license.is_valid(),
        "expires_at": license.expires_at,
        "days_until_expiry": license.days_until_expiry(),
    });
    
    Ok(Json(response))
}

/// Error type for API handlers
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("API error: {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Embedded HTML for the dashboard
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BBQ Monitor Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
            color: #fff;
            padding: 20px;
        }
        .container { max-width: 1400px; margin: 0 auto; }
        h1 {
            text-align: center;
            margin-bottom: 30px;
            font-size: 2.5em;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        .premium-badge {
            display: inline-block;
            background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
            color: white;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.7em;
            font-weight: bold;
            margin-left: 10px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }
        .premium-banner {
            background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
            padding: 15px 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            text-align: center;
            box-shadow: 0 4px 8px rgba(0,0,0,0.2);
        }
        .premium-banner h3 {
            margin-bottom: 8px;
            font-size: 1.2em;
        }
        .premium-banner p {
            opacity: 0.9;
            font-size: 0.9em;
            margin-bottom: 12px;
        }
        .premium-banner a {
            display: inline-block;
            background: white;
            color: #1e40af;
            padding: 10px 24px;
            border-radius: 6px;
            text-decoration: none;
            font-weight: bold;
            transition: transform 0.2s;
        }
        .premium-banner a:hover {
            transform: translateY(-2px);
        }
        .status {
            text-align: center;
            margin-bottom: 20px;
            padding: 10px;
            background: rgba(255,255,255,0.1);
            border-radius: 8px;
            font-size: 0.9em;
        }
        .status.connected { color: #4ade80; }
        .status.disconnected { color: #f87171; }
        .devices-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .device-card {
            background: rgba(255, 255, 255, 0.95);
            color: #1e293b;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 8px 16px rgba(0,0,0,0.2);
        }
        .device-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
            padding-bottom: 15px;
            border-bottom: 2px solid #e2e8f0;
        }
        .device-name {
            font-size: 1.3em;
            font-weight: bold;
            color: #1e40af;
        }
        .device-brand {
            font-size: 0.85em;
            color: #64748b;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .temperature-display {
            text-align: center;
            margin: 20px 0;
        }
        .temp-value {
            font-size: 3em;
            font-weight: bold;
            color: #dc2626;
        }
        .temp-label {
            font-size: 0.9em;
            color: #64748b;
            margin-top: 5px;
        }
        .timestamp {
            text-align: center;
            font-size: 0.75em;
            color: #94a3b8;
            margin-top: 8px;
        }
        .timestamp.aged {
            font-style: italic;
            color: #f59e0b;
        }
        .timestamp.stale {
            font-style: italic;
            color: #ef4444;
        }
        .metrics {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 10px;
            margin-top: 15px;
        }
        .metric {
            text-align: center;
            padding: 10px;
            background: #f1f5f9;
            border-radius: 6px;
        }
        .metric-value {
            font-size: 1.2em;
            font-weight: bold;
            color: #1e40af;
        }
        .metric-label {
            font-size: 0.75em;
            color: #64748b;
            text-transform: uppercase;
            margin-top: 3px;
        }
        .chart-container {
            margin-top: 20px;
            height: 200px;
        }
        @media (max-width: 768px) {
            .devices-grid {
                grid-template-columns: 1fr;
            }
            h1 { font-size: 1.8em; }
            .temp-value { font-size: 2.5em; }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔥 BBQ Monitor Dashboard<span class="premium-badge" id="tier-badge" style="display: none;">FREE</span></h1>
        <div id="premium-banner" style="display: none;"></div>
        <div id="status" class="status disconnected">
            ⚠️ Connecting to server...
        </div>
        <div id="devices" class="devices-grid"></div>
    </div>

    <script>
        let ws = null;
        let charts = {};
        let deviceData = {};

        // Load premium status
        async function loadPremiumStatus() {
            try {
                const response = await fetch('/api/premium/status');
                const status = await response.json();
                
                const badge = document.getElementById('tier-badge');
                badge.style.display = 'inline-block';
                badge.textContent = status.tier.toUpperCase();
                
                if (status.tier === 'Free') {
                    badge.style.background = 'linear-gradient(135deg, #64748b 0%, #475569 100%)';
                    
                    // Show premium banner for free users
                    const banner = document.getElementById('premium-banner');
                    banner.style.display = 'block';
                    banner.className = 'premium-banner';
                    banner.innerHTML = `
                        <h3>🌟 Upgrade to Premium</h3>
                        <p>Unlock cloud sync, unlimited history, cook profiles, and more!</p>
                        <a href="https://bbqmonitor.example.com/premium" target="_blank">View Premium Features →</a>
                    `;
                } else if (status.tier === 'Premium') {
                    badge.style.background = 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)';
                    
                    // Show expiry warning if needed
                    if (status.days_until_expiry !== null && status.days_until_expiry < 30) {
                        const banner = document.getElementById('premium-banner');
                        banner.style.display = 'block';
                        banner.className = 'premium-banner';
                        banner.style.background = 'linear-gradient(135deg, #dc2626 0%, #b91c1c 100%)';
                        banner.innerHTML = `
                            <h3>⚠️ License Expiring Soon</h3>
                            <p>Your Premium license expires in ${status.days_until_expiry} days</p>
                            <a href="https://bbqmonitor.example.com/renew" target="_blank">Renew License →</a>
                        `;
                    }
                }
            } catch (error) {
                console.error('Failed to load premium status:', error);
            }
        }

        function connect() {
            const wsUrl = `ws://${window.location.host}/ws`;
            ws = new WebSocket(wsUrl);
            
            ws.onopen = () => {
                console.log('WebSocket connected');
                updateStatus(true);
            };
            
            ws.onmessage = (event) => {
                const update = JSON.parse(event.data);
                handleUpdate(update);
            };
            
            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                updateStatus(false);
            };
            
            ws.onclose = () => {
                console.log('WebSocket disconnected');
                updateStatus(false);
                setTimeout(connect, 3000);
            };
        }

        function updateStatus(connected) {
            const status = document.getElementById('status');
            if (connected) {
                status.className = 'status connected';
                status.textContent = '✅ Connected - Live Updates Active';
            } else {
                status.className = 'status disconnected';
                status.textContent = '⚠️ Disconnected - Reconnecting...';
            }
        }

        function handleUpdate(update) {
            const addr = update.device_address;
            
            if (!deviceData[addr]) {
                deviceData[addr] = {
                    name: update.device_name,
                    address: addr,
                    readings: [],
                    timestamps: []
                };
                createDeviceCard(addr);
            }
            
            const data = deviceData[addr];
            data.readings.push(update.temperature);
            data.timestamps.push(new Date(update.timestamp));
            
            // Keep last 50 readings
            if (data.readings.length > 50) {
                data.readings.shift();
                data.timestamps.shift();
            }
            
            updateDeviceCard(addr, update);
            updateChart(addr);
        }

        function createDeviceCard(addr) {
            const data = deviceData[addr];
            const container = document.getElementById('devices');
            
            const card = document.createElement('div');
            card.className = 'device-card';
            card.id = `device-${addr}`;
            card.innerHTML = `
                <div class="device-header">
                    <div>
                        <div class="device-name">${data.name}</div>
                        <div class="device-brand">Thermometer</div>
                    </div>
                </div>
                <div class="temperature-display">
                    <div class="temp-value" id="temp-${addr}">--°F</div>
                    <div class="temp-label">Internal Temperature</div>
                    <div class="timestamp" id="timestamp-${addr}">No data</div>
                </div>
                <div class="metrics">
                    <div class="metric">
                        <div class="metric-value" id="ambient-${addr}">--</div>
                        <div class="metric-label">Ambient</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value" id="battery-${addr}">--</div>
                        <div class="metric-label">Battery</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value" id="rssi-${addr}">--</div>
                        <div class="metric-label">Signal</div>
                    </div>
                </div>
                <div class="chart-container">
                    <canvas id="chart-${addr}"></canvas>
                </div>
            `;
            
            container.appendChild(card);
            
            // Create chart
            const ctx = document.getElementById(`chart-${addr}`).getContext('2d');
            charts[addr] = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Temperature',
                        data: [],
                        borderColor: '#dc2626',
                        backgroundColor: 'rgba(220, 38, 38, 0.1)',
                        tension: 0.4,
                        fill: true
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { display: false }
                    },
                    scales: {
                        y: {
                            beginAtZero: false,
                            ticks: { color: '#64748b' }
                        },
                        x: {
                            ticks: { 
                                color: '#64748b',
                                maxTicksLimit: 8
                            }
                        }
                    }
                }
            });
        }

        function updateDeviceCard(addr, update) {
            document.getElementById(`temp-${addr}`).textContent = 
                `${update.temperature.toFixed(1)}°F`;
            
            document.getElementById(`ambient-${addr}`).textContent = 
                update.ambient_temp ? `${update.ambient_temp.toFixed(1)}°F` : '--';
            
            document.getElementById(`battery-${addr}`).textContent = 
                update.battery_level ? `${update.battery_level}%` : '--';
            
            document.getElementById(`rssi-${addr}`).textContent = 
                `${update.signal_strength} dBm`;
            
            // Update timestamp
            const timestampEl = document.getElementById(`timestamp-${addr}`);
            const now = new Date(update.timestamp);
            timestampEl.textContent = `Last: ${now.toLocaleTimeString()}`;
            timestampEl.dataset.timestamp = update.timestamp;
            updateTimestampAging(addr);
        }

        function updateChart(addr) {
            const chart = charts[addr];
            const data = deviceData[addr];
            
            chart.data.labels = data.timestamps.map(t => 
                t.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
            );
            chart.data.datasets[0].data = data.readings;
            chart.update('none');
        }

        function updateTimestampAging(addr) {
            const timestampEl = document.getElementById(`timestamp-${addr}`);
            if (!timestampEl || !timestampEl.dataset.timestamp) return;
            
            const lastUpdate = new Date(timestampEl.dataset.timestamp);
            const ageSeconds = (Date.now() - lastUpdate.getTime()) / 1000;
            
            // Remove all aging classes
            timestampEl.classList.remove('aged', 'stale');
            
            // Add appropriate class based on age
            if (ageSeconds > 60) {
                timestampEl.classList.add('stale');
            } else if (ageSeconds > 30) {
                timestampEl.classList.add('aged');
            }
        }

        function updateAllTimestamps() {
            for (const addr in deviceData) {
                updateTimestampAging(addr);
            }
        }

        // Update aging indicators every second
        setInterval(updateAllTimestamps, 1000);

        // Load premium status on page load
        loadPremiumStatus();

        // Start connection
        connect();
    </script>
</body>
</html>
"#;
