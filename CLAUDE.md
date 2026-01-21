# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BBQ Monitor is a Rust-based BLE temperature monitoring system for BBQ thermometer probes (MeatStick, MEATER, Weber iGrill). It features local SQLite persistence, an optional AWS cloud sync layer (IoT Core + DynamoDB), and an Axum-based web dashboard with WebSocket real-time updates. A Flutter mobile app integrates via FFI.

## Build Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Run main application
cargo run --bin license-tool -- generate premium 365   # Generate 1-year license
cargo run --bin license-tool -- validate "KEY"         # Validate a license key

# AWS features (requires Rust 1.88+, uncomment deps in Cargo.toml first)
cargo build --features aws --release
```

## Architecture

**Data Flow:**
```
BLE Devices → main.rs (scan/connect) → protocol.rs (parse temps)
   → database.rs (store) → web_server.rs (API/WebSocket)
                        ↘ aws_client.rs (cloud sync, optional)
```

**Key Modules:**

| Module | Purpose |
|--------|---------|
| main.rs | Entry point: config loading, BLE scanning, device monitoring loop, AWS sync task |
| protocol.rs | Temperature parsing for MeatStick (13-bit packed bit-fields) and MEATER (u16 little-endian) |
| database.rs | SQLite schema (devices + readings tables), indexed queries, data retention |
| device_capabilities.rs | Device detection by name prefix/service UUIDs, brand/model capabilities |
| web_server.rs | Axum routes: `/api/devices`, `/api/devices/:address/history`, `/ws` for real-time updates |
| premium.rs | License validation, feature gating (free: 7-day local, premium: cloud + unlimited) |
| aws_client.rs | IoT Core publishing, DynamoDB storage, periodic sync |
| config.rs | TOML configuration loading |
| lib.rs | FFI exports for Flutter integration |

**Configuration:** All runtime settings in `config.toml` (device filters, scan duration, database path, web server, AWS credentials, premium license).

## Protocol Details

- **MeatStick**: 8 sensors in 13-byte payload, formula: `temp_celsius = (raw_value * 0.05) - 20`
- **MEATER**: 2 sensors (tip + ambient) in 8-byte payload, u16 little-endian with ambient calculation offset
- See `PROTOCOL_DOCUMENTATION.md` for full BLE specs

## Licensing Model

- **Free tier**: Local monitoring, 7-day history retention
- **Premium tier**: Cloud sync, unlimited history, cook profiles, analytics, alerts, remote access
- License keys are Base64-encoded with expiry and feature flags (production should add RSA signatures)

## Key Patterns

- Async throughout using tokio runtime with broadcast channels for temperature updates
- Arc-wrapped database pool for shared access across tasks
- Error handling via `anyhow::Result<T>` with `.context()` additions
- Structured logging via `tracing` macros (info!, warn!, debug!, error!)

## Extension Points

- Add device support: Extend `protocol.rs` parsing + `device_capabilities.rs` detection
- Add API endpoints: Modify `web_server.rs` route handlers
- Adjust retention: Edit `retention_days` in config.toml (gated by license tier)
