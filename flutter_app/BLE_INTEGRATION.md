# BLE Integration Guide

## Overview
The BBQ Monitor app now includes full Bluetooth Low Energy (BLE)
support for connecting to wireless BBQ thermometers.

## Supported Devices
The app will automatically detect and connect to BBQ thermometers from these brands:
- MEATER
- MeatStick
- Inkbird
- ThermoWorks
- Weber
- Traeger

## Features

### Device Management
- **Auto-scanning**: Automatically scans for nearby devices on app launch
- **Manual refresh**: Pull down to refresh the device list
- **Connect/Disconnect**: Tap the link icon to connect or disconnect
  from a device
- **Multiple devices**: Connect to multiple thermometers simultaneously
  (Premium feature)

### Temperature Monitoring
- **Real-time updates**: Temperature data updates automatically when connected
- **Historical data**: View temperature history in the mini chart
- **Target temperature**: Tap the target temperature to set your desired
  doneness
- **Time estimation**: Automatic calculation of time remaining based on
  cooking rate
- **Stale data detection**: Warning indicator if no data received in
  30 seconds

### Data Display
- **Current temperature**: Large, easy-to-read display
- **Ambient temperature**: Chamber/grill temperature (if supported by device)
- **Progress indicator**: Visual progress bar toward target temperature
- **Temperature chart**: Historical temperature trend visualization

## Platform-Specific Setup

### Android
Bluetooth permissions are automatically requested when needed. The app requires:
- Bluetooth and Bluetooth Admin (Android < 12)
- Bluetooth Scan and Connect (Android 12+)
- Location (Android 10-11 only, for BLE scanning)

### iOS
Bluetooth permissions are requested on first use. Background Bluetooth
is enabled for continuous monitoring.

### Windows/macOS/Linux
Desktop platforms may require manual Bluetooth adapter configuration.
Check device settings if scanning doesn't work.

## Data Format
The BLE service expects temperature data in the following format:
- **2-byte format**: Temperature only (little-endian, 0.1°C resolution)
- **4-byte format**: Probe temp (bytes 0-1) + Ambient temp (bytes 2-3)

Temperatures are automatically converted from Celsius to Fahrenheit for display.

## Customization

### Adding Custom Device Support
To support additional device brands, update the `knownBrands` list in `lib/services/ble_service.dart`:

```dart
final knownBrands = ['meater', 'meatstick', 'inkbird', 'yourdevice'];
```

### Custom Data Parsing
If your device uses a different data format, modify the
`_parseTemperatureData` method in `lib/services/ble_service.dart`.

### Service UUIDs
Default UUIDs are configured for common devices. Update these constants if needed:
```dart
static const String temperatureServiceUuid = '0000fff0-0000-1000-8000-00805f9b34fb';
static const String temperatureCharUuid = '0000fff1-0000-1000-8000-00805f9b34fb';
```

## Troubleshooting

### Device Not Found
1. Ensure device is powered on and in pairing mode
2. Check Bluetooth is enabled on your phone/tablet
3. Try manual refresh (pull down on device list)
4. Verify device is within Bluetooth range (typically 30-100 feet)

### Connection Issues
1. Try disconnecting and reconnecting
2. Restart the app
3. Restart Bluetooth on your device
4. Check device battery level

### No Temperature Updates
1. Verify device is actually measuring (probe inserted)
2. Check for "No recent data" warning
3. Try disconnecting and reconnecting
4. Verify device is still in range

### Permission Errors
1. Check app permissions in device settings
2. Enable Bluetooth permission
3. On Android 10-11: Enable location permission (required for BLE)
4. Restart app after granting permissions

## Architecture

### Key Components
- **`BBQDevice` model**: Represents a connected thermometer with temperature history
- **`BLEService`**: Manages device scanning, connection, and data updates
- **`HomeScreen`**: Displays connected devices and temperature data

### State Management
The BLE service uses Provider for state management, automatically
notifying the UI of:
- Device discovery
- Connection status changes
- Temperature updates
- Scan state changes

### Data Flow
1. `BLEService` scans for devices
2. User taps connect on a device
3. Service establishes connection and discovers characteristics
4. Service subscribes to temperature notifications
5. Temperature data is parsed and stored in device history
6. UI automatically updates via Provider notifications

## Future Enhancements
- [ ] Device-specific profiles for better compatibility
- [ ] Configurable temperature units (°F/°C)
- [ ] Alert notifications for target temperatures
- [ ] Export temperature history
- [ ] Multi-probe support for advanced thermometers
- [ ] Cloud sync of temperature history (Premium)
