import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import '../services/rust_ffi.dart';
import '../models/bbq_device.dart';

class RustBLEService extends ChangeNotifier {
  static final RustBLEService _instance = RustBLEService._internal();
  static RustBLEService get instance => _instance;
  
  final RustFFI _ffi = RustFFI.instance;
  final List<BBQDevice> _devices = [];
  bool _isScanning = false;
  bool _isInitialized = false;
  Timer? _pollTimer;
  Timer? _scanTimer;
  String? _dbPath;
  bool _backgroundMonitorStarted = false;

  List<BBQDevice> get devices => List.unmodifiable(_devices);
  bool get isScanning => _isScanning;
  bool get isInitialized => _isInitialized;

  RustBLEService._internal();

  Future<void> initialize() async {
    debugPrint('🚀 Starting RustBLEService initialization...');
    try {
      // Initialize database path
      if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
        final appDir = await getApplicationDocumentsDirectory();
        _dbPath = '${appDir.path}/bbq_monitor.db';
        debugPrint('📁 Database path: $_dbPath');
      } else {
        final appDir = await getApplicationSupportDirectory();
        _dbPath = '${appDir.path}/bbq_monitor.db';
        debugPrint('📁 Database path: $_dbPath');
      }

      // Initialize BLE
      debugPrint('🔧 Calling Rust BLE initialize...');
      final success = _ffi.initializeBle();
      debugPrint('🔧 Rust BLE initialize result: $success');
      
      if (success) {
        _isInitialized = true;
        debugPrint('✅ Rust BLE initialized successfully');
        
        // Start background monitoring (BLE scanning + AWS sync in Rust)
        await _startBackgroundMonitor();
        
        // Start polling database for updates
        _startDatabasePolling();
      } else {
        debugPrint('❌ Failed to initialize Rust BLE');
        _isInitialized = true; // Still mark as initialized so UI shows
      }
      debugPrint('📢 Notifying listeners - isInitialized: $_isInitialized');
      notifyListeners();
    } catch (e, stackTrace) {
      debugPrint('❌❌❌ BLE initialization error: $e');
      debugPrint('Stack trace: $stackTrace');
      _isInitialized = true; // Mark as initialized even on error so UI shows
      notifyListeners();
    }
  }

  Future<void> _startBackgroundMonitor() async {
    if (_backgroundMonitorStarted) return;
    
    try {
      // Get config path
      String configPath;
      if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
        final appDir = await getApplicationDocumentsDirectory();
        configPath = '${appDir.path}/config.toml';
        
        // Create default config if doesn't exist
        final configFile = File(configPath);
        if (!await configFile.exists()) {
          await configFile.writeAsString(_getDefaultConfig());
        }
      } else {
        configPath = 'config.toml'; // Bundled asset for mobile
      }

      final success = _ffi.startBackgroundMonitoring(_dbPath!, configPath);
      if (success) {
        _backgroundMonitorStarted = true;
        debugPrint('✅ Background monitor started (Rust handles BLE + AWS sync)');
      } else {
        debugPrint('⚠️ Failed to start background monitor');
      }
    } catch (e) {
      debugPrint('Background monitor error: $e');
    }
  }

  String _getDefaultConfig() {
    return '''
[device]
scan_duration = 15
monitor_duration = 300
reconnect_attempts = 3

[filters]
device_prefixes = ["cA00", "cA02", "Y0C"]
mac_filters = []
min_rssi = -80

[temperature]
unit = "fahrenheit"
max_internal_temp = 200.0
max_ambient_temp = 1000.0
warning_threshold_percent = 90.0

[database]
path = "bbq_monitor.db"
retention_days = 7
batch_size = 100

[logging]
level = "info"
file_enabled = true
file_path = "bbq_monitor.log"

[web]
enabled = false
host = "127.0.0.1"
port = 8080

[premium]
license_key = ""

[aws]
enabled = false
region = "us-east-1"
thing_name = ""
table_name = "bbq-monitor-readings"
sync_interval_secs = 300
''';
  }

  void _startDatabasePolling() {
    // Poll database every 2 seconds for new devices and readings
    _pollTimer?.cancel();
    _pollTimer = Timer.periodic(const Duration(seconds: 2), (_) {
      _updateDevicesFromDatabase();
    });
  }

  Future<void> _updateDevicesFromDatabase() async {
    if (_dbPath == null) return;

    try {
      // Get devices from database
      final devicesList = _ffi.getDatabaseDevices(_dbPath!);
      
      // BBQDevice model adapted: uses Rust backend for BLE, no BluetoothDevice needed
      // All BLE communication handled by Rust layer via FFI
      debugPrint('Found ${devicesList.length} devices in database');
      
      // Update devices list
      _devices.clear();
      for (final deviceData in devicesList) {
        debugPrint('  - ${deviceData['device_name']} (${deviceData['device_address']})');
        
        // Create BBQDevice from database data
        final device = BBQDevice(
          device: null, // No BluetoothDevice with Rust backend
          name: deviceData['device_name'] ?? 'Unknown Device',
          id: deviceData['device_address'] ?? '',
        );
        _devices.add(device);
      }
      
      notifyListeners();
    } catch (e) {
      debugPrint('Error updating devices from database: $e');
    }
  }

  Future<void> startScanning() async {
    if (_isScanning || !_isInitialized) return;

    try {
      _isScanning = true;
      notifyListeners();

      // Rust backend handles actual scanning in background
      // We just need to poll the database for results
      debugPrint('📡 Scanning for BBQ devices (Rust backend)...');
      
      // Stop scanning after 30 seconds
      _scanTimer?.cancel();
      _scanTimer = Timer(const Duration(seconds: 30), stopScanning);
      
    } catch (e) {
      debugPrint('BLE scanning error: $e');
      _isScanning = false;
      notifyListeners();
    }
  }

  Future<void> stopScanning() async {
    try {
      _scanTimer?.cancel();
      _isScanning = false;
      notifyListeners();
      debugPrint('🛑 Stopped scanning');
    } catch (e) {
      debugPrint('Error stopping scan: $e');
    }
  }

  Future<List<Map<String, dynamic>>> getDeviceHistory(
    String deviceId,
    DateTime startTime,
    DateTime endTime,
  ) async {
    if (_dbPath == null) return [];
    
    try {
      return _ffi.getDatabaseHistory(_dbPath!, deviceId, startTime, endTime);
    } catch (e) {
      debugPrint('Error getting device history: $e');
      return [];
    }
  }

  Future<List<Map<String, dynamic>>> getRecentReadings(
    String deviceId,
    int limit,
  ) async {
    if (_dbPath == null) return [];
    
    try {
      return _ffi.getDatabaseReadings(_dbPath!, deviceId, limit);
    } catch (e) {
      debugPrint('Error getting recent readings: $e');
      return [];
    }
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    _scanTimer?.cancel();
    super.dispose();
  }
}
