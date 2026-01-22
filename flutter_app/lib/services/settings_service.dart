import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SettingsService extends ChangeNotifier {
  bool _offlineMode = false;
  bool _isInitialized = false;

  bool get offlineMode => _offlineMode;
  bool get isInitialized => _isInitialized;
  
  // Cloud sync is enabled when user has premium AND offline mode is OFF
  bool get cloudSyncEnabled => !_offlineMode;

  SettingsService() {
    _init();
  }

  Future<void> _init() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      _offlineMode = prefs.getBool('offline_mode') ?? false;
      _isInitialized = true;
      notifyListeners();
    } catch (e) {
      debugPrint('Error loading settings: $e');
      _isInitialized = true;
      notifyListeners();
    }
  }

  Future<void> setOfflineMode(bool enabled) async {
    _offlineMode = enabled;
    notifyListeners();
    
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setBool('offline_mode', enabled);
    } catch (e) {
      debugPrint('Error saving offline mode: $e');
    }
  }

  Future<void> toggleOfflineMode() async {
    await setOfflineMode(!_offlineMode);
  }
}
