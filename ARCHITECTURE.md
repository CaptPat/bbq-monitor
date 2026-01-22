# BBQ Monitor Architecture

## Overview

The BBQ Monitor uses a clean separation between the **Rust backend** (data engine) and **Flutter UI** (view layer) via FFI (Foreign Function Interface).

## Architecture Layers

### Rust Backend (Data Engine)
**Location:** `src/*.rs` compiled to `bbq_monitor.dll` / `libbbq_monitor.so`

**Responsibilities:**
- **BLE Device Management:** Scan, connect, and communicate with BBQ thermometers using `btleplug`
- **Data Storage:** Write all device readings to SQLite database via `sqlx`
- **Cloud Sync:** Background AWS IoT and DynamoDB sync (when enabled)
- **License Validation:** Premium feature enforcement
- **FFI Interface:** Expose C-compatible functions for Flutter to call

**Key Features:**
- Cross-platform BLE support (Windows, macOS, Linux, iOS, Android)
- Async background tasks using Tokio runtime
- Automatic device discovery and reconnection
- Efficient batch writes to SQLite

### Flutter UI (View Layer)
**Location:** `flutter_app/lib/**/*.dart`

**Responsibilities:**
- **Display Data:** Query SQLite via Rust FFI and render UI
- **User Input:** Settings, navigation, feature selection
- **Premium Gates:** Show upgrade prompts for locked features
- **Offline Mode:** Toggle cloud sync behavior

**Key Features:**
- Material Design 3 UI
- Real-time temperature monitoring
- Charts and history visualization
- Settings and preferences management

## FFI Interface

### License Functions
```c
// Validate license key (returns 1 if valid, 0 if invalid)
int validate_license(const char* key);

// Get license info as JSON string
char* get_license_info(const char* key);

// Free license JSON memory
void free_license_json(char* ptr);
```

### BLE Functions
```c
// Initialize BLE manager
int ble_initialize();

// Start scanning for BBQ devices
int ble_start_scan();

// Stop scanning
int ble_stop_scan();

// Get discovered devices as JSON array
char* ble_get_devices();

// Free devices JSON memory
void ble_free_devices_json(char* ptr);
```

### Database Query Functions
```c
// Get all devices from database
char* db_get_devices(const char* db_path);

// Get recent readings for a device (limit=0 for all)
char* db_get_readings(const char* db_path, const char* device_id, int limit);

// Get latest reading for a device
char* db_get_latest_reading(const char* db_path, const char* device_id);

// Get readings within time range (ISO 8601 timestamps)
char* db_get_history(const char* db_path, const char* device_id, 
                     const char* start_time, const char* end_time);

// Free database JSON memory
void db_free_json(char* ptr);
```

### Background Task Function
```c
// Start background BLE monitoring and cloud sync
// Runs continuously in separate thread
int start_background_monitor(const char* db_path, const char* config_path);
```

## Data Flow

### Device Discovery & Monitoring
```
┌──────────────┐
│ BBQ Thermometer │
│  (BLE Device)   │
└────────┬─────────┘
         │ BLE GATT
         │ (btleplug)
         ▼
┌─────────────────┐
│  Rust Backend   │
│  - Scan devices │
│  - Read temps   │
│  - Parse data   │
└────────┬────────┘
         │ sqlx
         │ INSERT readings
         ▼
┌─────────────────┐
│  SQLite DB      │
│  - devices      │
│  - readings     │
└────────┬────────┘
         │ FFI Query
         │ SELECT * FROM readings
         ▼
┌─────────────────┐
│  Flutter UI     │
│  - Display chart│
│  - Show alerts  │
└─────────────────┘
```

### Cloud Sync (Premium Feature)
```
┌─────────────────┐
│  SQLite DB      │
│  readings table │
└────────┬────────┘
         │ Background Task
         │ (Tokio spawn)
         ▼
┌─────────────────┐
│  AWS Client     │
│  - IoT Core     │
│  - DynamoDB     │
└────────┬────────┘
         │ HTTPS
         ▼
┌─────────────────┐
│  AWS Cloud      │
│  - Thing Shadow │
│  - Data Table   │
└─────────────────┘
```

## Database Schema

### devices table
```sql
CREATE TABLE devices (
    device_address TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    brand TEXT NOT NULL,
    model TEXT NOT NULL,
    sensor_count INTEGER NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL
);
```

### readings table
```sql
CREATE TABLE readings (
    device_address TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    sensor_index INTEGER NOT NULL,
    temperature REAL NOT NULL,
    ambient_temp REAL,
    battery_level INTEGER,
    signal_strength INTEGER NOT NULL,
    PRIMARY KEY (device_address, timestamp, sensor_index),
    FOREIGN KEY (device_address) REFERENCES devices(device_address)
);
```

## Configuration

### config.toml
```toml
[device]
scan_duration = 15        # BLE scan duration (seconds)
monitor_duration = 300    # How long to monitor (seconds)
reconnect_attempts = 3

[filters]
device_prefixes = ["cA00", "cA02", "Y0C"]  # MeatStick prefixes
mac_filters = []          # Optional MAC address filters
min_rssi = -80           # Minimum signal strength

[database]
path = "bbq_monitor.db"
retention_days = 7       # Free tier retention

[premium]
license_key = ""         # Empty for free tier

[aws]
enabled = false          # Requires premium license
region = "us-east-1"
thing_name = ""
table_name = "bbq-monitor-readings"
sync_interval_secs = 300
```

## Platform Support

### Rust Backend (btleplug)
- ✅ Windows 10/11 (native Bluetooth LE APIs)
- ✅ macOS (CoreBluetooth)
- ✅ Linux (BlueZ)
- ✅ iOS (CoreBluetooth)
- ✅ Android (via JNI)

### Flutter UI
- ✅ Android (Material Design)
- ✅ iOS (Cupertino widgets)
- ✅ Windows (Win32)
- ✅ macOS
- ✅ Linux
- ✅ Web (limited - no BLE)

## Premium Features

Enforced by `LicenseValidator` in Rust backend:

### Free Tier
- BLE device scanning (1 device)
- Real-time temperature monitoring
- 7-day history retention
- Local-only storage

### Premium Tier
- Unlimited devices
- Unlimited history retention
- AWS cloud sync
- Custom alerts
- Cook profiles
- Advanced analytics

## Development Workflow

### Building Rust Backend
```bash
cd bbq-monitor
cargo build --release --lib
# Produces: target/release/bbq_monitor.dll (Windows)
#          target/release/libbbq_monitor.so (Linux)
#          target/release/libbbq_monitor.dylib (macOS)
```

### Building Flutter App
```bash
cd flutter_app

# Copy Rust DLL to Flutter build directory
# Windows Debug:
cp ../target/release/bbq_monitor.dll build/windows/x64/runner/Debug/
# Windows Release:
cp ../target/release/bbq_monitor.dll build/windows/x64/runner/Release/

# Run app
flutter run -d windows
flutter run -d android
flutter run -d ios
```

### Testing
```bash
# Rust tests
cargo test

# Flutter tests
cd flutter_app
flutter test

# Integration tests
flutter integration_test
```

## Deployment

### Desktop (Windows/macOS/Linux)
1. Build Rust library: `cargo build --release`
2. Build Flutter app: `flutter build windows/macos/linux`
3. Copy Rust DLL to app bundle
4. Create installer with Inno Setup / pkg / AppImage

### Mobile (Android/iOS)
1. Rust library automatically built via `flutter build apk/ipa`
2. Native code included in app bundle
3. Submit to Play Store / App Store

## Future Enhancements

- [ ] Remove `flutter_blue_plus` dependency entirely
- [ ] Implement full BLE FFI bridge in Flutter
- [ ] Add web server UI for desktop monitoring
- [ ] Implement MQTT support for real-time alerts
- [ ] Add Bluetooth mesh networking for multiple monitors
- [ ] Support additional thermometer brands (Thermoworks, Inkbird)
