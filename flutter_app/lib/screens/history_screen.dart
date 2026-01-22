import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:fl_chart/fl_chart.dart';
import '../services/rust_ble_service.dart';
import '../services/license_service.dart';
import '../models/bbq_device.dart';

class HistoryScreen extends StatelessWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Temperature History'),
        actions: [
          Consumer<LicenseService>(
            builder: (context, licenseService, child) {
              if (licenseService.isPremium) {
                return const Padding(
                  padding: EdgeInsets.only(right: 16),
                  child: Center(
                    child: Text(
                      'Unlimited History',
                      style: TextStyle(
                        color: Colors.amber,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
                );
              }
              return const Padding(
                padding: EdgeInsets.only(right: 16),
                child: Center(
                  child: Text(
                    '7-Day Limit',
                    style: TextStyle(
                      color: Color(0xFFBF360C),
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
              );
            },
          ),
        ],
      ),
      body: Consumer2<LicenseService, RustBLEService>(
        builder: (context, licenseService, bleService, child) {
          return Column(
            children: [
              if (!licenseService.isPremium)
                _buildLimitedBanner(context),
              Expanded(
                child: _buildHistoryList(context, bleService, licenseService),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildLimitedBanner(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
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
            Icons.history,
            color: Colors.orange.shade700,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Free tier: 7-day history limit',
                  style: TextStyle(
                    color: Colors.orange.shade900,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                Text(
                  'Upgrade to Premium for unlimited history',
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.orange.shade700,
                  ),
                ),
              ],
            ),
          ),
          ElevatedButton(
            onPressed: () {
              // Navigate to premium screen
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.orange,
              foregroundColor: Colors.white,
            ),
            child: const Text('Upgrade'),
          ),
        ],
      ),
    );
  }

  Widget _buildHistoryList(
    BuildContext context,
    RustBLEService bleService,
    LicenseService licenseService,
  ) {
    if (bleService.devices.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.history,
              size: 64,
              color: Colors.grey[400],
            ),
            const SizedBox(height: 16),
            Text(
              'No devices with history',
              style: TextStyle(
                fontSize: 18,
                color: Colors.grey[600],
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: bleService.devices.length,
      itemBuilder: (context, index) {
        final device = bleService.devices[index];
        return _buildDeviceHistoryCard(device, licenseService.isPremium);
      },
    );
  }

  Widget _buildDeviceHistoryCard(BBQDevice device, bool isPremium) {
    final history = device.history;
    
    // Free users see limited history
    final displayHistory = isPremium 
        ? history 
        : history.take(20).toList(); // Show last 20 readings for free

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.bluetooth_connected,
                  color: device.isConnected ? Colors.green : Colors.grey,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    device.name,
                    style: const TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                if (!isPremium && history.length > 20)
                  Chip(
                    label: Text(
                      '${history.length - 20}+ older',
                      style: const TextStyle(fontSize: 12),
                    ),
                    backgroundColor: Colors.orange.shade100,
                  ),
              ],
            ),
            const SizedBox(height: 16),
            SizedBox(
              height: 200,
              child: _buildHistoryChart(displayHistory),
            ),
            const SizedBox(height: 12),
            Text(
              '${displayHistory.length} readings shown',
              style: TextStyle(
                fontSize: 12,
                color: Colors.grey[600],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHistoryChart(List<dynamic> history) {
    if (history.isEmpty) {
      return Center(
        child: Text(
          'No history data',
          style: TextStyle(color: Colors.grey[600]),
        ),
      );
    }

    final spots = history.asMap().entries.map((entry) {
      return FlSpot(
        entry.key.toDouble(),
        entry.value.temperature,
      );
    }).toList();

    return LineChart(
      LineChartData(
        gridData: FlGridData(
          show: true,
          drawVerticalLine: false,
          getDrawingHorizontalLine: (value) {
            return FlLine(
              color: Colors.grey[300]!,
              strokeWidth: 1,
            );
          },
        ),
        titlesData: const FlTitlesData(
          leftTitles: AxisTitles(
            sideTitles: SideTitles(
              showTitles: true,
              reservedSize: 40,
            ),
          ),
          bottomTitles: AxisTitles(
            sideTitles: SideTitles(showTitles: false),
          ),
          rightTitles: AxisTitles(
            sideTitles: SideTitles(showTitles: false),
          ),
          topTitles: AxisTitles(
            sideTitles: SideTitles(showTitles: false),
          ),
        ),
        borderData: FlBorderData(
          show: true,
          border: Border(
            bottom: BorderSide(color: Colors.grey[300]!),
            left: BorderSide(color: Colors.grey[300]!),
          ),
        ),
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
        ],
      ),
    );
  }
}
