# BBQ Monitor

A Rust-based Bluetooth Low Energy (BLE) temperature monitoring system for BBQ probes, with support for MeatStick, MEATER, and Weber iGrill devices.

## Features

### Phase 1 (Completed) ✅

- **Professional Logging** - Structured logging with `tracing` framework
  - Console and file output
  - Configurable log levels (trace, debug, info, warn, error)
  - Timestamped entries with proper formatting

- **Configuration Management** - TOML-based configuration
  - Device scan settings
  - Temperature thresholds and alerts
  - Database configuration
  - BLE filtering options (RSSI, MAC address, device names)

- **SQLite Database Storage** - Persistent data storage
  - Device registry (name, model, capabilities)
  - Temperature readings (multi-sensor support)
  - Automatic data retention cleanup
  - Indexed queries for performance

- **Improved BLE Parsing** - Protocol-aware temperature parsing
  - MeatStick protocol (6-sensor support)
  - MEATER protocol (2-sensor support)
  - Automatic Celsius to Fahrenheit conversion
  - Internal vs. ambient temperature detection
  - Sanity checking for invalid readings

## Quick Start

1. **Configure** - Edit `config.toml` to set your preferences:

   ```toml
   [device]
   scan_duration = 5
   monitor_duration = 300
   
   [temperature]
   unit = "fahrenheit"
   max_internal_temp = 200.0
   max_ambient_temp = 1000.0
   ```

2. **Run**:

   ```bash
   cargo run
   ```

3. **View Data** - Readings are stored in `bbq_monitor.db` (SQLite)

## Configuration

See [`config.toml`](config.toml) for all available options:

- **Device Settings**: Scan duration, reconnection attempts
- **Filters**: RSSI threshold, MAC filters, device name prefixes
- **Temperature**: Units, safety thresholds
- **Database**: Path, retention, batch size
- **Logging**: Level, file output

## Database Schema

### `devices` Table

- `device_address` - MAC address (primary key)
- `device_name` - Bluetooth name
- `brand` - Detected probe brand
- `model` - Model identifier
- `sensor_count` - Number of temperature sensors
- `first_seen` / `last_seen` - Connection timestamps

### `readings` Table

- `device_address` - Foreign key to devices
- `timestamp` - Reading timestamp
- `sensor_index` - Sensor position (0-5 for MeatStick V)
- `temperature` - Temperature in °F
- `ambient_temp` - Ambient temperature (if available)
- `battery_level` - Battery percentage (if available)
- `signal_strength` - RSSI value

## Architecture

```text
src/
├── main.rs              # BLE scanning, connection, monitoring
├── lib.rs               # Module exports
├── config.rs            # Configuration management
├── database.rs          # SQLite operations
├── protocol.rs          # Temperature parsing protocols
└── device_capabilities.rs  # Device detection & capabilities
```

## Supported Devices

- **MeatStick** (cA00*, cA02*, Y0C*)
  - MeatStick V (6 sensors, 1000°F ambient)
  - MeatStick V2 (2 sensors)
  - MeatStick Base Stations

- **MEATER**
  - MEATER Original
  - MEATER Plus
  - MEATER Block

- **Weber iGrill** (partial support)

## Logging

Logs are written to both console and file (`bbq_monitor.log` by default):

```text
INFO  🔥 BBQ Monitor v0.1.0 - Starting
INFO  Using adapter: Intel(R) Wireless Bluetooth(R)
INFO  🍖 Found: cA0012345678 (AA:BB:CC:DD:EE:FF) - RSSI: -65dBm
INFO     ✅ Connected to cA0012345678
INFO     📋 Detected: MeatStickV with 6 sensors
INFO  🌡️  cA0012345678 - Internal: 72.0°F, Ambient: 85.0°F, Sensors: 6
```

## Next Steps (Phase 2+)

- [ ] AWS IoT integration (device shadows, DynamoDB)
- [ ] Web dashboard (real-time monitoring)
- [ ] Temperature alerts & notifications
- [ ] Cooking profiles & presets
- [ ] Automatic reconnection & error recovery
- [ ] Unit tests & integration tests

## Requirements

- Rust 1.70+
- Bluetooth 4.0+ adapter
- Windows/Linux/macOS

## License

See LICENSE file for details.
