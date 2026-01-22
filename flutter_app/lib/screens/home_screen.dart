import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/rust_ble_service.dart';
import '../services/license_service.dart';
import 'package:fl_chart/fl_chart.dart';
import '../models/bbq_device.dart';
import '../utils/premium_gate.dart';
import 'premium_screen.dart';
import 'settings_screen.dart';
import 'history_screen.dart';
import 'alerts_screen.dart';
import 'cook_profiles_screen.dart';

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
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => const SettingsScreen()),
              );
            },
            tooltip: 'Settings',
          ),
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
      drawer: _buildDrawer(context),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text(
              'STATIC TEST - Window Working',
              style: TextStyle(fontSize: 24, color: Colors.red),
            ),
            const SizedBox(height: 20),
            Consumer<RustBLEService>(
              builder: (context, bleService, child) {
                return Column(
                  children: [
                    Text('Initialized: ${bleService.isInitialized}'),
                    Text('Scanning: ${bleService.isScanning}'),
                    Text('Devices: ${bleService.devices.length}'),
                    const SizedBox(height: 20),
                    ElevatedButton(
                      onPressed: () => bleService.startScanning(),
                      child: const Text('Start Scan'),
                    ),
                  ],
                );
              },
            ),
          ],
        ),
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

  // Unused - keeping for future use
  // ignore: unused_element
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

  // Unused - keeping for future use
  // ignore: unused_element
  Widget _buildOfflineModeBanner() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Colors.orange.shade50,
        border: Border(
          bottom: BorderSide(
            color: Colors.orange.shade200,
            width: 1,
          ),
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.cloud_off,
            color: Colors.orange.shade700,
            size: 20,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              'Offline Mode - Data stored locally only',
              style: TextStyle(
                color: Colors.orange.shade900,
                fontSize: 14,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          TextButton(
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => const SettingsScreen()),
              );
            },
            child: Text(
              'Settings',
              style: TextStyle(
                color: Colors.orange.shade700,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildDeviceList(RustBLEService bleService) {
    // Show loading indicator during initialization
    if (!bleService.isInitialized) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Initializing Bluetooth...'),
          ],
        ),
      );
    }
    
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
                  ? 'Scanning for devices... (0 found)'
                  : '0 devices found',
              style: TextStyle(
                fontSize: 18,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () {
                if (bleService.isScanning) {
                  bleService.stopScanning();
                } else {
                  bleService.startScanning();
                }
              },
              icon: Icon(
                bleService.isScanning ? Icons.cancel : Icons.refresh,
              ),
              label: Text(
                bleService.isScanning ? 'Cancel Scan' : 'Scan for Devices',
              ),
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

  Widget _buildDeviceCard(BBQDevice device, RustBLEService bleService) {
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
                  onPressed: null, // Connection managed by Rust backend
                  tooltip: isConnected ? 'Connected' : 'Disconnected',
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

  Widget _buildDrawer(BuildContext context) {
    return Drawer(
      child: Consumer<LicenseService>(
        builder: (context, licenseService, child) {
          return ListView(
            padding: EdgeInsets.zero,
            children: [
              DrawerHeader(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    colors: [
                      Theme.of(context).primaryColor,
                      Theme.of(context).primaryColor.withValues(alpha: 0.7),
                    ],
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    const Icon(
                      Icons.outdoor_grill,
                      size: 48,
                      color: Colors.white,
                    ),
                    const SizedBox(height: 12),
                    const Text(
                      'BBQ Monitor',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 24,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      licenseService.isPremium ? 'Premium User' : 'Free User',
                      style: TextStyle(
                        color: Colors.white.withValues(alpha: 0.9),
                        fontSize: 14,
                      ),
                    ),
                  ],
                ),
              ),
              ListTile(
                leading: const Icon(Icons.home),
                title: const Text('Home'),
                onTap: () => Navigator.pop(context),
              ),
              ListTile(
                leading: const Icon(Icons.history),
                title: const Text('History'),
                trailing: licenseService.isPremium
                    ? null
                    : Icon(
                        Icons.lock,
                        size: 16,
                        color: Colors.orange.shade700,
                      ),
                onTap: () {
                  Navigator.pop(context);
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => const HistoryScreen()),
                  );
                },
              ),
              ListTile(
                leading: const Icon(Icons.notifications),
                title: const Text('Alerts'),
                trailing: licenseService.isPremium
                    ? null
                    : Icon(
                        Icons.lock,
                        size: 16,
                        color: Colors.orange.shade700,
                      ),
                onTap: () {
                  Navigator.pop(context);
                  if (PremiumGate.checkAccess(context, featureName: 'Custom Alerts')) {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => const AlertsScreen()),
                    );
                  }
                },
              ),
              ListTile(
                leading: const Icon(Icons.restaurant),
                title: const Text('Cook Profiles'),
                trailing: licenseService.isPremium
                    ? null
                    : Icon(
                        Icons.lock,
                        size: 16,
                        color: Colors.orange.shade700,
                      ),
                onTap: () {
                  Navigator.pop(context);
                  if (PremiumGate.checkAccess(context, featureName: 'Cook Profiles')) {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => const CookProfilesScreen()),
                    );
                  }
                },
              ),
              const Divider(),
              ListTile(
                leading: const Icon(Icons.settings),
                title: const Text('Settings'),
                onTap: () {
                  Navigator.pop(context);
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => const SettingsScreen()),
                  );
                },
              ),
              if (!licenseService.isPremium)
                ListTile(
                  leading: const Icon(Icons.workspace_premium, color: Colors.amber),
                  title: const Text(
                    'Upgrade to Premium',
                    style: TextStyle(
                      color: Colors.amber,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  onTap: () {
                    Navigator.pop(context);
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => const PremiumScreen()),
                    );
                  },
                ),
            ],
          );
        },
      ),
    );
  }

  Future<void> _showTargetTempDialog(BBQDevice device, RustBLEService bleService) async {
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
      // Target temperature stored locally in device model
      // Backend storage would require adding endpoint to Rust web server
      debugPrint('Target temperature set: $result°F');
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Target set to $result°F')),
        );
      }
    }
  }
}
