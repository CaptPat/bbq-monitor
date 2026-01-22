import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:fl_chart/fl_chart.dart';
import '../services/license_service.dart';
import '../services/ble_service.dart';
import '../models/bbq_device.dart';
import 'premium_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('BBQ Monitor'),
        actions: [
          Consumer<LicenseService>(
            builder: (context, licenseService, child) {
              if (licenseService.isPremium) {
                return _buildPremiumBadge();
              }
              return _buildUpgradeButton();
            },
          ),
        ],
      ),
      body: Consumer2<LicenseService, BLEService>(
        builder: (context, licenseService, bleService, child) {
          return Column(
            children: [
              if (!licenseService.isPremium) _buildUpgradeBanner(),
              Expanded(
                child: _buildDeviceList(bleService),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildPremiumBadge() {
    return Container(
      margin: const EdgeInsets.only(right: 16),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: Colors.amber,
        borderRadius: BorderRadius.circular(16),
      ),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.workspace_premium,
            size: 16,
            color: Colors.white,
          ),
          SizedBox(width: 4),
          Text(
            'PREMIUM',
            style: TextStyle(
              color: Colors.white,
              fontSize: 12,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUpgradeButton() {
    return TextButton.icon(
      onPressed: () {
        Navigator.push(
          context,
          MaterialPageRoute(builder: (context) => const PremiumScreen()),
        );
      },
      icon: const Icon(Icons.arrow_upward, size: 18),
      label: const Text('Upgrade'),
      style: TextButton.styleFrom(
        foregroundColor: Colors.amber,
      ),
    );
  }

  Widget _buildUpgradeBanner() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            Colors.orange.shade400,
            Colors.deepOrange.shade400,
          ],
        ),
      ),
      child: Row(
        children: [
          const Icon(
            Icons.star,
            color: Colors.white,
            size: 24,
          ),
          const SizedBox(width: 12),
          const Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Unlock Cloud Sync & More',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  'Unlimited history, alerts, and multi-device access',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => const PremiumScreen()),
              );
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.white,
              foregroundColor: Colors.deepOrange,
            ),
            child: const Text('Upgrade'),
          ),
        ],
      ),
    );
  }

  Widget _buildDeviceList(BLEService bleService) {
    if (bleService.devices.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.bluetooth_searching,
              size: 64,
              color: Colors.grey[400],
            ),
            const SizedBox(height: 16),
            Text(
              bleService.isScanning
                  ? 'Scanning for devices...'
                  : 'No devices found',
              style: TextStyle(
                fontSize: 18,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: bleService.isScanning
                  ? null
                  : () => bleService.startScanning(),
              icon: const Icon(Icons.refresh),
              label: const Text('Scan for Devices'),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: () async {
        await bleService.startScanning();
      },
      child: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: bleService.devices.length,
        itemBuilder: (context, index) {
          final device = bleService.devices[index];
          return Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: _buildDeviceCard(device, bleService),
          );
        },
      ),
    );
  }

  Widget _buildDeviceCard(BBQDevice device, BLEService bleService) {
    final currentTemp = device.currentTemp ?? 0.0;
    final targetTemp = device.targetTemp ?? 165.0;
    final ambientTemp = device.ambientTemp;
    final isConnected = device.isConnected;
    final isStale = device.isStale;
    final progress = device.progress;
    
    return Card(
      elevation: 4,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Row(
              children: [
                Icon(
                  isConnected
                      ? Icons.bluetooth_connected
                      : Icons.bluetooth,
                  color: isConnected
                      ? (isStale ? Colors.orange : Colors.green)
                      : Colors.grey,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        device.name,
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      if (isStale && isConnected)
                        Text(
                          'No recent data',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.orange[700],
                          ),
                        ),
                    ],
                  ),
                ),
                IconButton(
                  icon: Icon(
                    isConnected ? Icons.link_off : Icons.link,
                    color: isConnected ? Colors.red : Colors.blue,
                  ),
                  onPressed: () {
                    if (isConnected) {
                      bleService.disconnectFromDevice(device);
                    } else {
                      bleService.connectToDevice(device);
                    }
                  },
                  tooltip: isConnected ? 'Disconnect' : 'Connect',
                ),
              ],
            ),
            
            const SizedBox(height: 16),

            // Current Temperature
            Row(
              children: [
                Text(
                  device.currentTemp != null
                      ? '${currentTemp.toStringAsFixed(1)}°F'
                      : '--°F',
                  style: const TextStyle(
                    fontSize: 48,
                    fontWeight: FontWeight.bold,
                    color: Colors.deepOrange,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      GestureDetector(
                        onTap: () => _showTargetTempDialog(device, bleService),
                        child: Row(
                          children: [
                            Text(
                              'Target: ${targetTemp.toStringAsFixed(0)}°F',
                              style: TextStyle(
                                fontSize: 14,
                                color: Colors.grey[600],
                              ),
                            ),
                            const SizedBox(width: 4),
                            Icon(
                              Icons.edit,
                              size: 14,
                              color: Colors.grey[600],
                            ),
                          ],
                        ),
                      ),
                      if (ambientTemp != null)
                        Text(
                          'Ambient: ${ambientTemp.toStringAsFixed(0)}°F',
                          style: TextStyle(
                            fontSize: 14,
                            color: Colors.grey[600],
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),

            const SizedBox(height: 16),

            // Progress Bar
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                LinearProgressIndicator(
                  value: progress,
                  minHeight: 8,
                  backgroundColor: Colors.grey[300],
                  valueColor: AlwaysStoppedAnimation<Color>(
                    progress < 0.5
                        ? Colors.blue
                        : progress < 0.8
                            ? Colors.orange
                            : Colors.green,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  '${(progress * 100).toStringAsFixed(0)}% complete',
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.grey[600],
                  ),
                ),
              ],
            ),

            const SizedBox(height: 16),

            // Time Remaining
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.blue.shade50,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(
                    Icons.access_time,
                    size: 16,
                    color: Colors.blue,
                  ),
                  const SizedBox(width: 4),
                  Text(
                    device.timeRemaining,
                    style: const TextStyle(
                      fontSize: 14,
                      color: Colors.blue,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ],
              ),
            ),

            const SizedBox(height: 16),

            // Mini Chart (placeholder)
            SizedBox(
              height: 100,
              child: _buildMiniChart(device, targetTemp),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMiniChart(BBQDevice device, double target) {
    final history = device.history;
    
    // Use real history if available, otherwise show placeholder
    final spots = history.isNotEmpty
        ? history.asMap().entries.map((entry) {
            return FlSpot(
              entry.key.toDouble(),
              entry.value.temperature,
            );
          }).toList()
        : [FlSpot(0, device.currentTemp ?? 0)];

    return LineChart(
      LineChartData(
        gridData: const FlGridData(show: false),
        titlesData: const FlTitlesData(show: false),
        borderData: FlBorderData(show: false),
        minX: 0,
        maxX: spots.length > 1 ? spots.length.toDouble() - 1 : 19,
        minY: 0,
        maxY: target + 20,
        lineBarsData: [
          LineChartBarData(
            spots: spots,
            isCurved: true,
            color: Colors.deepOrange,
            barWidth: 3,
            dotData: const FlDotData(show: false),
            belowBarData: BarAreaData(
              show: true,
              color: Colors.deepOrange.withValues(alpha: 0.2),
            ),
          ),
          // Target line
          LineChartBarData(
            spots: [
              FlSpot(0, target),
              FlSpot(spots.length > 1 ? spots.length.toDouble() - 1 : 19, target),
            ],
            isCurved: false,
            color: Colors.green.withValues(alpha: 0.5),
            barWidth: 2,
            dotData: const FlDotData(show: false),
            dashArray: [5, 5],
          ),
        ],
      ),
    );
  }

  Future<void> _showTargetTempDialog(BBQDevice device, BLEService bleService) async {
    final controller = TextEditingController(
      text: (device.targetTemp ?? 165.0).toStringAsFixed(0),
    );

    final result = await showDialog<double>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Set Target Temperature'),
        content: TextField(
          controller: controller,
          keyboardType: TextInputType.number,
          decoration: const InputDecoration(
            labelText: 'Target Temperature (°F)',
            hintText: '165',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              final temp = double.tryParse(controller.text);
              if (temp != null && temp > 0 && temp <= 500) {
                Navigator.pop(context, temp);
              }
            },
            child: const Text('Set'),
          ),
        ],
      ),
    );

    if (result != null) {
      await bleService.setTargetTemperature(device, result);
    }
  }
}
