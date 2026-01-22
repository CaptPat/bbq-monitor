# Bluetooth Protocol Documentation

This document describes the Bluetooth Low Energy (BLE) protocols used by
supported BBQ thermometer devices.

## Table of Contents

- [Combustion Inc (MeatStick)](#combustion-inc-meatstick)
- [MEATER Protocol](#meater-protocol)
- [References](#references)

---

## Combustion Inc (MeatStick)

### Primary Service UUIDs

**Probe Status Service:**

- UUID: `00000100-CAAB-3792-3D44-97AE51C1407A`
- Primary service for temperature data and device status

**UART Service (Nordic-style):**

- Service UUID: `6E400001-B5A3-F393-E0A9-E50E24DCCA9E`
- RX Characteristic (write to device): `6E400002-B5A3-F393-E0A9-E50E24DCCA9E`
- TX Characteristic (notifications from device): `6E400003-B5A3-F393-E0A9-E50E24DCCA9E`

**Legacy MeatStick Service (older firmware):**

- Service UUID: `8D53DC1D-1DB7-4CD3-868B-8A527460AA84`
- Characteristic UUID: `DA2E7828-FBCE-4E01-AE9E-261174997C48`

### MeatStick Temperature Format

**Probe models** (Predictive Thermometer):

- **Data Length:** 13 bytes (104 bits)
- **Sensor Count:** 8 thermistors
- **Encoding:** 13-bit values per sensor, packed bit-fields, little-endian
- **Temperature Conversion:** `Temperature (°C) = (raw_value * 0.05) - 20`
- **Temperature Range:** -20°C to 369°C per sensor

#### Sensor Layout

For Combustion Predictive Probe:

- **T1-T4:** Core temperatures (deepest internal readings)
  - T4 is typically the deepest sensor
- **T5-T7:** Mid-section temperatures
- **T8:** Ambient/surface temperature (closest to handle)

#### Data Structure

```text
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│   T1   │   T2   │   T3   │   T4   │   T5   │   T6   │   T7   │   T8   │
│13 bits │13 bits │13 bits │13 bits │13 bits │13 bits │13 bits │13 bits │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
Total: 104 bits = 13 bytes
```

#### Example Parsing

For a raw 13-bit value of `844`:

- Temperature (°C) = (844 * 0.05) - 20 = 22.2°C
- Temperature (°F) = 22.2 * 9/5 + 32 = 72°F

### MeatStick Device Detection

**Device Name Pattern:**

- MeatStick devices: Names starting with `cA00`
- Base stations: Names starting with `cA02`

**Capabilities:**

- MeatStick V: 8 sensors, 1000°F ambient max, 650ft range, 24hr battery
- Base stations: Act as repeaters to extend range

---

## MEATER Protocol

### MEATER Service UUIDs

**Primary Service:**

- Service UUID: `A75CC7FC-C956-488F-AC2A-2DBC08B63A04`
- Uses standard BLE GATT characteristics

**Characteristic Handles:**

- Handle 22: Firmware version
- Handle 31: Temperature data (8 bytes)
- Handle 35: Battery level

### MEATER Temperature Format

**Data Length:** 8 bytes

**Structure:**

```text
Byte 0-1: Tip Temperature (little-endian u16)
Byte 2-3: RA Ambient Reading (little-endian u16)
Byte 4-5: OA Ambient Reading (little-endian u16)
Byte 6-7: Reserved/Unknown
```

### Temperature Conversion

**Tip Temperature:**

```rust
temp_celsius = raw_value / 10.0
temp_fahrenheit = temp_celsius * 9/5 + 32
```

**Ambient Temperature:**

```rust
ambient_raw = tip_raw + max(0, ((ra_raw - min(48, oa_raw)) * 16 * 589) / 1487)
ambient_celsius = ambient_raw / 10.0
ambient_fahrenheit = ambient_celsius * 9/5 + 32
```

Where:

- `tip_raw`: Raw tip temperature value
- `ra_raw`: RA ambient reading
- `oa_raw`: OA ambient reading

### MEATER Device Detection

**Device Name Pattern:**

- Contains "MEATER" (case-insensitive)
- Variants: "MEATER", "MEATER PLUS", "MEATER BLOCK"

**Capabilities:**

- MEATER Original: 2 sensors, 527°F ambient max, 33ft range, 24hr battery
- MEATER Plus: 2 sensors, 527°F ambient max, 165ft range, 24hr battery
- MEATER Block: Base station for up to 4 probes

---

## Implementation Notes

### Sensor Priority

When reading multiple sensors:

**MeatStick (Combustion):**

1. For internal temperature: Use T4 (deepest core), fallback to T3, T2, T1
2. For ambient temperature: Use T8 (surface), fallback to T7 or T6

**MEATER:**

1. Index 0: Tip temperature (internal)
2. Index 1: Ambient temperature

### Error Handling

Validate temperature readings:

- Internal: -40°F to 600°F
- Ambient: -40°F to 1100°F
- Discard readings of 0°F or out-of-range values
- Handle insufficient data gracefully

### Testing

Tests are provided in `src/protocol.rs` to verify:

- Correct parsing of packed bit-field data (MeatStick)
- Proper temperature conversion formulas (both protocols)
- Edge cases and boundary conditions

---

## References

### Official Documentation

1. **Combustion Inc (MeatStick):**
   - Repository: <https://github.com/combustion-inc/combustion-documentation>
   - Files:
     - `Probe BLE Specification.md`
     - `MeatNet Node BLE Specification.md`
     - `UART Service.md`

2. **MEATER:**
   - No official public documentation available
   - Reverse engineering by Nathan Faber: <https://github.com/nathanfaber/meaterble>

### Additional Resources

- Bluetooth SIG GATT Specifications: <https://www.bluetooth.com/specifications/gatt/>
- Nordic UART Service: <https://developer.nordicsemi.com/nRF_Connect_SDK/doc/latest/nrf/libraries/bluetooth_services/services/nus.html>

---

## License

This documentation is provided for interoperability purposes. All trademarks
and protocols are property of their respective owners:

- MeatStick®, Combustion Inc.™
- MEATER®, Apption Labs™
