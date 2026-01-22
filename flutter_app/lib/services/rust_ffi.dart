// Flutter FFI bindings for Rust backend
import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:convert';
import 'package:ffi/ffi.dart';

// Type definitions for C functions
typedef ValidateLicenseC = ffi.Int8 Function(ffi.Pointer<Utf8>);
typedef ValidateLicenseDart = int Function(ffi.Pointer<Utf8>);

typedef GetLicenseInfoC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>);
typedef GetLicenseInfoDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>);

typedef FreeLicenseJsonC = ffi.Void Function(ffi.Pointer<Utf8>);
typedef FreeLicenseJsonDart = void Function(ffi.Pointer<Utf8>);

typedef BleInitializeC = ffi.Int8 Function();
typedef BleInitializeDart = int Function();

typedef BleStartScanC = ffi.Int8 Function();
typedef BleStartScanDart = int Function();

typedef BleStopScanC = ffi.Int8 Function();
typedef BleStopScanDart = int Function();

typedef BleGetDevicesC = ffi.Pointer<Utf8> Function();
typedef BleGetDevicesDart = ffi.Pointer<Utf8> Function();

typedef BleFreeDevicesJsonC = ffi.Void Function(ffi.Pointer<Utf8>);
typedef BleFreeDevicesJsonDart = void Function(ffi.Pointer<Utf8>);

typedef DbGetDevicesC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>);
typedef DbGetDevicesDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>);

typedef DbGetReadingsC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Int32);
typedef DbGetReadingsDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, int);

typedef DbGetLatestReadingC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);
typedef DbGetLatestReadingDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);

typedef DbGetHistoryC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);
typedef DbGetHistoryDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);

typedef DbFreeJsonC = ffi.Void Function(ffi.Pointer<Utf8>);
typedef DbFreeJsonDart = void Function(ffi.Pointer<Utf8>);

typedef StartBackgroundMonitorC = ffi.Int8 Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);
typedef StartBackgroundMonitorDart = int Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>);

class RustFFI {
  late ffi.DynamicLibrary _lib;
  
  // License functions
  late ValidateLicenseDart validateLicense;
  late GetLicenseInfoDart getLicenseInfo;
  late FreeLicenseJsonDart freeLicenseJson;
  
  // BLE functions
  late BleInitializeDart bleInitialize;
  late BleStartScanDart bleStartScan;
  late BleStopScanDart bleStopScan;
  late BleGetDevicesDart bleGetDevices;
  late BleFreeDevicesJsonDart bleFreeDevicesJson;
  
  // Database functions
  late DbGetDevicesDart dbGetDevices;
  late DbGetReadingsDart dbGetReadings;
  late DbGetLatestReadingDart dbGetLatestReading;
  late DbGetHistoryDart dbGetHistory;
  late DbFreeJsonDart dbFreeJson;
  
  // Background monitor
  late StartBackgroundMonitorDart startBackgroundMonitor;
  
  static RustFFI? _instance;
  
  RustFFI._() {
    _loadLibrary();
    _bindFunctions();
  }
  
  static RustFFI get instance {
    _instance ??= RustFFI._();
    return _instance!;
  }
  
  void _loadLibrary() {
    if (Platform.isWindows) {
      _lib = ffi.DynamicLibrary.open('bbq_monitor.dll');
    } else if (Platform.isLinux) {
      _lib = ffi.DynamicLibrary.open('libbbq_monitor.so');
    } else if (Platform.isMacOS) {
      _lib = ffi.DynamicLibrary.open('libbbq_monitor.dylib');
    } else if (Platform.isAndroid) {
      _lib = ffi.DynamicLibrary.open('libbbq_monitor.so');
    } else if (Platform.isIOS) {
      _lib = ffi.DynamicLibrary.process();
    } else {
      throw UnsupportedError('Unsupported platform');
    }
  }
  
  void _bindFunctions() {
    // License functions
    validateLicense = _lib.lookupFunction<ValidateLicenseC, ValidateLicenseDart>('validate_license');
    getLicenseInfo = _lib.lookupFunction<GetLicenseInfoC, GetLicenseInfoDart>('get_license_info');
    freeLicenseJson = _lib.lookupFunction<FreeLicenseJsonC, FreeLicenseJsonDart>('free_license_json');
    
    // BLE functions
    bleInitialize = _lib.lookupFunction<BleInitializeC, BleInitializeDart>('ble_initialize');
    bleStartScan = _lib.lookupFunction<BleStartScanC, BleStartScanDart>('ble_start_scan');
    bleStopScan = _lib.lookupFunction<BleStopScanC, BleStopScanDart>('ble_stop_scan');
    bleGetDevices = _lib.lookupFunction<BleGetDevicesC, BleGetDevicesDart>('ble_get_devices');
    bleFreeDevicesJson = _lib.lookupFunction<BleFreeDevicesJsonC, BleFreeDevicesJsonDart>('ble_free_devices_json');
    
    // Database functions
    dbGetDevices = _lib.lookupFunction<DbGetDevicesC, DbGetDevicesDart>('db_get_devices');
    dbGetReadings = _lib.lookupFunction<DbGetReadingsC, DbGetReadingsDart>('db_get_readings');
    dbGetLatestReading = _lib.lookupFunction<DbGetLatestReadingC, DbGetLatestReadingDart>('db_get_latest_reading');
    dbGetHistory = _lib.lookupFunction<DbGetHistoryC, DbGetHistoryDart>('db_get_history');
    dbFreeJson = _lib.lookupFunction<DbFreeJsonC, DbFreeJsonDart>('db_free_json');
    
    // Background monitor
    startBackgroundMonitor = _lib.lookupFunction<StartBackgroundMonitorC, StartBackgroundMonitorDart>('start_background_monitor');
  }
  
  // Helper methods to handle string conversion and memory management
  
  bool validateLicenseKey(String key) {
    final keyPtr = key.toNativeUtf8();
    try {
      final result = validateLicense(keyPtr);
      return result == 1;
    } finally {
      malloc.free(keyPtr);
    }
  }
  
  Map<String, dynamic>? getLicenseInformation(String key) {
    final keyPtr = key.toNativeUtf8();
    try {
      final jsonPtr = getLicenseInfo(keyPtr);
      if (jsonPtr.address == 0) return null;
      
      try {
        final jsonStr = jsonPtr.toDartString();
        return jsonDecode(jsonStr) as Map<String, dynamic>;
      } finally {
        freeLicenseJson(jsonPtr);
      }
    } finally {
      malloc.free(keyPtr);
    }
  }
  
  bool initializeBle() {
    final result = bleInitialize();
    return result == 1;
  }
  
  bool startBleScan() {
    final result = bleStartScan();
    return result == 1;
  }
  
  bool stopBleScan() {
    final result = bleStopScan();
    return result == 1;
  }
  
  List<Map<String, dynamic>> getBleDevices() {
    final jsonPtr = bleGetDevices();
    if (jsonPtr.address == 0) return [];
    
    try {
      final jsonStr = jsonPtr.toDartString();
      final decoded = jsonDecode(jsonStr);
      return List<Map<String, dynamic>>.from(decoded);
    } finally {
      bleFreeDevicesJson(jsonPtr);
    }
  }
  
  List<Map<String, dynamic>> getDatabaseDevices(String dbPath) {
    final pathPtr = dbPath.toNativeUtf8();
    try {
      final jsonPtr = dbGetDevices(pathPtr);
      if (jsonPtr.address == 0) return [];
      
      try {
        final jsonStr = jsonPtr.toDartString();
        final decoded = jsonDecode(jsonStr);
        return List<Map<String, dynamic>>.from(decoded);
      } finally {
        dbFreeJson(jsonPtr);
      }
    } finally {
      malloc.free(pathPtr);
    }
  }
  
  List<Map<String, dynamic>> getDatabaseReadings(String dbPath, String deviceId, int limit) {
    final pathPtr = dbPath.toNativeUtf8();
    final idPtr = deviceId.toNativeUtf8();
    try {
      final jsonPtr = dbGetReadings(pathPtr, idPtr, limit);
      if (jsonPtr.address == 0) return [];
      
      try {
        final jsonStr = jsonPtr.toDartString();
        final decoded = jsonDecode(jsonStr);
        return List<Map<String, dynamic>>.from(decoded);
      } finally {
        dbFreeJson(jsonPtr);
      }
    } finally {
      malloc.free(pathPtr);
      malloc.free(idPtr);
    }
  }
  
  Map<String, dynamic>? getLatestDatabaseReading(String dbPath, String deviceId) {
    final pathPtr = dbPath.toNativeUtf8();
    final idPtr = deviceId.toNativeUtf8();
    try {
      final jsonPtr = dbGetLatestReading(pathPtr, idPtr);
      if (jsonPtr.address == 0) return null;
      
      try {
        final jsonStr = jsonPtr.toDartString();
        return jsonDecode(jsonStr) as Map<String, dynamic>;
      } finally {
        dbFreeJson(jsonPtr);
      }
    } finally {
      malloc.free(pathPtr);
      malloc.free(idPtr);
    }
  }
  
  List<Map<String, dynamic>> getDatabaseHistory(
    String dbPath,
    String deviceId,
    DateTime startTime,
    DateTime endTime,
  ) {
    final pathPtr = dbPath.toNativeUtf8();
    final idPtr = deviceId.toNativeUtf8();
    final startPtr = startTime.toIso8601String().toNativeUtf8();
    final endPtr = endTime.toIso8601String().toNativeUtf8();
    
    try {
      final jsonPtr = dbGetHistory(pathPtr, idPtr, startPtr, endPtr);
      if (jsonPtr.address == 0) return [];
      
      try {
        final jsonStr = jsonPtr.toDartString();
        final decoded = jsonDecode(jsonStr);
        return List<Map<String, dynamic>>.from(decoded);
      } finally {
        dbFreeJson(jsonPtr);
      }
    } finally {
      malloc.free(pathPtr);
      malloc.free(idPtr);
      malloc.free(startPtr);
      malloc.free(endPtr);
    }
  }
  
  bool startBackgroundMonitoring(String dbPath, String configPath) {
    final dbPathPtr = dbPath.toNativeUtf8();
    final configPathPtr = configPath.toNativeUtf8();
    
    try {
      final result = startBackgroundMonitor(dbPathPtr, configPathPtr);
      return result == 1;
    } finally {
      malloc.free(dbPathPtr);
      malloc.free(configPathPtr);
    }
  }
}
