import 'package:flutter_blue_plus/flutter_blue_plus.dart';

class BBQDevice {
  final String id;
  final String name;
  final BluetoothDevice? device; // Nullable for Rust backend support
  double? currentTemp;
  double? targetTemp;
  double? ambientTemp;
  DateTime? lastUpdate;
  bool isConnected;
  List<TemperatureReading> history;

  BBQDevice({
    required this.id,
    required this.name,
    this.device, // Optional for Rust backend
    this.currentTemp,
    this.targetTemp,
    this.ambientTemp,
    this.lastUpdate,
    this.isConnected = false,
    List<TemperatureReading>? history,
  }) : history = history ?? [];

  String get timeRemaining {
    if (currentTemp == null || targetTemp == null || currentTemp! >= targetTemp!) {
      return '--';
    }

    // Calculate based on temperature rise rate from history
    if (history.length < 2) return 'Calculating...';

    final recentReadings = history.length > 10 
        ? history.sublist(history.length - 10) 
        : history;
    
    if (recentReadings.isEmpty) return 'Calculating...';

    final firstReading = recentReadings.first;
    final lastReading = recentReadings.last;
    final timeDiff = lastReading.timestamp.difference(firstReading.timestamp).inSeconds;
    
    if (timeDiff == 0) return 'Calculating...';

    final tempDiff = lastReading.temperature - firstReading.temperature;
    final ratePerSecond = tempDiff / timeDiff;

    if (ratePerSecond <= 0) return 'N/A';

    final remainingTemp = targetTemp! - currentTemp!;
    final secondsRemaining = (remainingTemp / ratePerSecond).round();

    final hours = secondsRemaining ~/ 3600;
    final minutes = (secondsRemaining % 3600) ~/ 60;

    if (hours > 0) {
      return '${hours}h ${minutes}min';
    } else {
      return '${minutes}min';
    }
  }

  double get progress {
    if (currentTemp == null || targetTemp == null || targetTemp == 0) {
      return 0.0;
    }
    return (currentTemp! / targetTemp!).clamp(0.0, 1.0);
  }

  bool get isStale {
    if (lastUpdate == null) return true;
    final age = DateTime.now().difference(lastUpdate!);
    return age.inSeconds > 30; // Consider stale if no update in 30 seconds
  }

  void updateTemperature(double temp, double? ambient) {
    currentTemp = temp;
    if (ambient != null) ambientTemp = ambient;
    lastUpdate = DateTime.now();
    history.add(TemperatureReading(
      temperature: temp,
      ambientTemperature: ambient,
      timestamp: lastUpdate!,
    ));

    // Keep only last 100 readings to prevent memory issues
    if (history.length > 100) {
      history = history.sublist(history.length - 100);
    }
  }
}

class TemperatureReading {
  final double temperature;
  final double? ambientTemperature;
  final DateTime timestamp;

  TemperatureReading({
    required this.temperature,
    this.ambientTemperature,
    required this.timestamp,
  });
}
