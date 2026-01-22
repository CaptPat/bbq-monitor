import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'package:ffi/ffi.dart';

enum LicenseTier {
  free,
  premium,
}

class License {
  final LicenseTier tier;
  final bool cloudSync;
  final bool unlimitedHistory;
  final bool cookProfiles;
  final bool advancedAnalytics;
  final bool alerts;
  final DateTime? expiresAt;
  
  License({
    required this.tier,
    required this.cloudSync,
    required this.unlimitedHistory,
    required this.cookProfiles,
    required this.advancedAnalytics,
    required this.alerts,
    this.expiresAt,
  });
  
  factory License.free() {
    return License(
      tier: LicenseTier.free,
      cloudSync: false,
      unlimitedHistory: false,
      cookProfiles: false,
      advancedAnalytics: false,
      alerts: false,
    );
  }
  
  factory License.premium({DateTime? expiresAt}) {
    return License(
      tier: LicenseTier.premium,
      cloudSync: true,
      unlimitedHistory: true,
      cookProfiles: true,
      advancedAnalytics: true,
      alerts: true,
      expiresAt: expiresAt,
    );
  }
  
  bool get isValid {
    if (expiresAt == null) return true;
    return DateTime.now().isBefore(expiresAt!);
  }
  
  int? get daysUntilExpiry {
    if (expiresAt == null) return null;
    return expiresAt!.difference(DateTime.now()).inDays;
  }
}

class LicenseService with ChangeNotifier {
  static final LicenseService instance = LicenseService._();
  LicenseService._();
  
  License _license = License.free();
  String? _licenseKey;
  
  License get license => _license;
  String? get licenseKey => _licenseKey;
  bool get isPremium => _license.tier == LicenseTier.premium && _license.isValid;
  
  Future<void> initialize() async {
    // Load saved license key
    final prefs = await SharedPreferences.getInstance();
    _licenseKey = prefs.getString('license_key');
    
    if (_licenseKey != null && _licenseKey!.isNotEmpty) {
      await _validateLicense(_licenseKey!);
    }
  }
  
  Future<bool> activateLicense(String key) async {
    try {
      // Validate with Rust core via FFI
      final isValid = await _validateLicenseWithRust(key);
      
      if (isValid) {
        _licenseKey = key;
        
        // Save to local storage
        final prefs = await SharedPreferences.getInstance();
        await prefs.setString('license_key', key);
        
        await _validateLicense(key);
        notifyListeners();
        return true;
      }
      
      return false;
    } catch (e) {
      debugPrint('License activation failed: $e');
      return false;
    }
  }
  
  Future<void> _validateLicense(String key) async {
    // Call Rust FFI to validate license
    final isValid = await _validateLicenseWithRust(key);
    
    if (isValid) {
      // Parse license details from Rust validation
      // Rust FFI validates and returns premium license status
      _license = License.premium();
      debugPrint('License validated successfully via Rust FFI');
    } else {
      _license = License.free();
    }
    
    notifyListeners();
  }
  
  Future<bool> _validateLicenseWithRust(String key) async {
    try {
      // Load Rust library
      final dylib = _loadLibrary();
      
      // Get function pointer
      final validateLicense = dylib.lookupFunction<
          ffi.Int8 Function(ffi.Pointer<Utf8>),
          int Function(ffi.Pointer<Utf8>)
      >('validate_license');
      
      // Convert string to C string
      final keyPtr = key.toNativeUtf8(allocator: malloc);
      
      // Call Rust function
      final result = validateLicense(keyPtr);
      
      // Free memory
      malloc.free(keyPtr);
      
      return result == 1;
    } catch (e) {
      debugPrint('Rust FFI error: $e');
      return false;
    }
  }
  
  ffi.DynamicLibrary _loadLibrary() {
    if (Platform.isAndroid) {
      return ffi.DynamicLibrary.open('libbbq_monitor.so');
    } else if (Platform.isWindows) {
      return ffi.DynamicLibrary.open('bbq_monitor.dll');
    } else if (Platform.isLinux) {
      return ffi.DynamicLibrary.open('libbbq_monitor.so');
    } else if (Platform.isMacOS) {
      return ffi.DynamicLibrary.open('libbbq_monitor.dylib');
    }
    throw UnsupportedError('Platform not supported');
  }
  
  Future<void> clearLicense() async {
    _license = License.free();
    _licenseKey = null;
    
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('license_key');
    
    notifyListeners();
  }
}
