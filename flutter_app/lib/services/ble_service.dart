import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter_blue_plus/flutter_blue_plus.dart';
import '../models/bbq_device.dart';

class BLEService extends ChangeNotifier {
  final List<BBQDevice> _devices = [];
  bool _isScanning = false;
  StreamSubscription<List<ScanResult>>? _scanSubscription;
  final Map<String, StreamSubscription> _deviceSubscriptions = {};

  // Common UUIDs for BBQ thermometers - these may need adjustment for specific devices
  static const String temperatureServiceUuid = '0000fff0-0000-1000-8000-00805f9b34fb';
  static const String temperatureCharUuid = '0000fff1-0000-1000-8000-00805f9b34fb';

  List<BBQDevice> get devices => List.unmodifiable(_devices);
  bool get isScanning => _isScanning;

  BLEService() {
    _init();
  }

  Future<void> _init() async {
    // Check if Bluetooth is available
    if (await FlutterBluePlus.isSupported == false) {
      debugPrint('Bluetooth not supported by this device');
      return;
    }

    // Start scanning automatically
    startScanning();
  }

  Future<void> startScanning() async {
    if (_isScanning) return;

    try {
      _isScanning = true;
      notifyListeners();

      // Start scanning with a filter for common BBQ thermometer services
      await FlutterBluePlus.startScan(
        timeout: const Duration(seconds: 15),
        androidUsesFineLocation: false,
      );

      _scanSubscription = FlutterBluePlus.scanResults.listen((results) {
        for (ScanResult result in results) {
          _addOrUpdateDevice(result);
        }
      });

      // Auto-stop after timeout
      Future.delayed(const Duration(seconds: 15), () {
        if (_isScanning) stopScanning();
      });
    } catch (e) {
      debugPrint('Error starting scan: $e');
      _isScanning = false;
      notifyListeners();
    }
  }

  Future<void> stopScanning() async {
    try {
      await FlutterBluePlus.stopScan();
    } catch (e) {
      debugPrint('Error stopping scan: $e');
    }
    
    _isScanning = false;
    notifyListeners();
  }

  void _addOrUpdateDevice(ScanResult result) {
    final deviceName = result.device.platformName;
    
    // Filter for known BBQ thermometer brands
    final knownBrands = ['meater', 'meatstick', 'inkbird', 'thermoworks', 'weber', 'traeger'];
    final isKnownDevice = knownBrands.any(
      (brand) => deviceName.toLowerCase().contains(brand)
    );

    if (!isKnownDevice && deviceName.isEmpty) return;

    final existingIndex = _devices.indexWhere(
      (d) => d.device.remoteId == result.device.remoteId
    );

    if (existingIndex == -1) {
      final device = BBQDevice(
        id: result.device.remoteId.toString(),
        name: deviceName.isNotEmpty ? deviceName : 'Unknown Device',
        device: result.device,
      );
      _devices.add(device);
      notifyListeners();
    }
  }

  Future<void> connectToDevice(BBQDevice device) async {
    try {
      await device.device.connect(
        timeout: const Duration(seconds: 15),
        autoConnect: false,
      );

      device.isConnected = true;
      notifyListeners();

      // Discover services
      List<BluetoothService> services = await device.device.discoverServices();
      
      // Find temperature characteristic and subscribe to notifications
      for (BluetoothService service in services) {
        for (BluetoothCharacteristic characteristic in service.characteristics) {
          if (characteristic.properties.notify) {
            await characteristic.setNotifyValue(true);
            
            _deviceSubscriptions[device.id] = characteristic.lastValueStream.listen((value) {
              _parseTemperatureData(device, value);
            });
          }
        }
      }
    } catch (e) {
      debugPrint('Error connecting to device: $e');
      device.isConnected = false;
      notifyListeners();
    }
  }

  Future<void> disconnectFromDevice(BBQDevice device) async {
    try {
      // Cancel subscription
      await _deviceSubscriptions[device.id]?.cancel();
      _deviceSubscriptions.remove(device.id);

      // Disconnect
      await device.device.disconnect();
      
      device.isConnected = false;
      notifyListeners();
    } catch (e) {
      debugPrint('Error disconnecting from device: $e');
    }
  }

  void _parseTemperatureData(BBQDevice device, List<int> value) {
    // This is a generic parser - specific devices may need custom parsing
    // MEATER format: Bytes 0-1: tip temp, Bytes 2-3: ambient temp (little endian)
    
    try {
      if (value.length >= 4) {
        // Parse as 16-bit little endian integers
        final tipRaw = value[0] | (value[1] << 8);
        final ambientRaw = value[2] | (value[3] << 8);
        
        // Convert to Fahrenheit (assuming raw values are in 0.1°C)
        final tipCelsius = tipRaw / 10.0;
        final ambientCelsius = ambientRaw / 10.0;
        
        final tipFahrenheit = (tipCelsius * 9 / 5) + 32;
        final ambientFahrenheit = (ambientCelsius * 9 / 5) + 32;
        
        device.updateTemperature(tipFahrenheit, ambientFahrenheit);
        notifyListeners();
      } else if (value.length >= 2) {
        // Simple 2-byte temperature only
        final tempRaw = value[0] | (value[1] << 8);
        final tempCelsius = tempRaw / 10.0;
        final tempFahrenheit = (tempCelsius * 9 / 5) + 32;
        
        device.updateTemperature(tempFahrenheit, null);
        notifyListeners();
      }
    } catch (e) {
      debugPrint('Error parsing temperature data: $e');
    }
  }

  Future<void> setTargetTemperature(BBQDevice device, double targetTemp) async {
    device.targetTemp = targetTemp;
    notifyListeners();
    
    // TODO: Send target temperature to device if supported
    // Some devices allow setting target temps via BLE
  }

  void removeDevice(BBQDevice device) {
    disconnectFromDevice(device);
    _devices.remove(device);
    notifyListeners();
  }

  @override
  void dispose() {
    stopScanning();
    _scanSubscription?.cancel();
    
    // Disconnect all devices
    for (var device in _devices) {
      if (device.isConnected) {
        disconnectFromDevice(device);
      }
    }
    
    super.dispose();
  }
}
