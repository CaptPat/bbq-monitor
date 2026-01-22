import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/license_service.dart';

class AlertsScreen extends StatefulWidget {
  const AlertsScreen({super.key});

  @override
  State<AlertsScreen> createState() => _AlertsScreenState();
}

class _AlertsScreenState extends State<AlertsScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Alerts & Notifications'),
      ),
      body: Consumer<LicenseService>(
        builder: (context, licenseService, child) {
          if (!licenseService.isPremium) {
            return _buildPremiumRequired(context);
          }
          return _buildAlertsContent();
        },
      ),
    );
  }

  Widget _buildPremiumRequired(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.notifications_off,
              size: 80,
              color: Colors.grey[400],
            ),
            const SizedBox(height: 24),
            const Text(
              'Premium Feature',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Custom alerts and notifications require a Premium subscription.',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 16,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 32),
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                color: Colors.blue.shade50,
                borderRadius: BorderRadius.circular(16),
              ),
              child: Column(
                children: [
                  _buildFeature(Icons.notifications_active, 'Custom temperature alerts'),
                  _buildFeature(Icons.email, 'Email notifications'),
                  _buildFeature(Icons.sms, 'SMS alerts'),
                  _buildFeature(Icons.timer, 'Cook time notifications'),
                  _buildFeature(Icons.warning, 'Stall detection alerts'),
                ],
              ),
            ),
            const SizedBox(height: 32),
            ElevatedButton.icon(
              onPressed: () {
                // Navigate to premium screen
              },
              icon: const Icon(Icons.arrow_upward),
              label: const Text('Upgrade to Premium'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.amber,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(
                  horizontal: 32,
                  vertical: 16,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildFeature(IconData icon, String text) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Icon(icon, color: Colors.blue.shade700),
          const SizedBox(width: 12),
          Text(
            text,
            style: const TextStyle(fontSize: 16),
          ),
        ],
      ),
    );
  }

  Widget _buildAlertsContent() {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Temperature Alerts',
                  style: TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Target temperature reached'),
                  subtitle: const Text('Alert when food reaches target temp'),
                  value: true,
                  onChanged: (value) {},
                  contentPadding: EdgeInsets.zero,
                ),
                SwitchListTile(
                  title: const Text('High temperature warning'),
                  subtitle: const Text('Alert if temperature exceeds safe limit'),
                  value: true,
                  onChanged: (value) {},
                  contentPadding: EdgeInsets.zero,
                ),
                SwitchListTile(
                  title: const Text('Low temperature warning'),
                  subtitle: const Text('Alert if temperature drops too low'),
                  value: false,
                  onChanged: (value) {},
                  contentPadding: EdgeInsets.zero,
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Notification Methods',
                  style: TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Push notifications'),
                  subtitle: const Text('In-app and system notifications'),
                  value: true,
                  onChanged: (value) {},
                  contentPadding: EdgeInsets.zero,
                ),
                SwitchListTile(
                  title: const Text('Email alerts'),
                  subtitle: const Text('Send alerts to your email'),
                  value: false,
                  onChanged: (value) {},
                  contentPadding: EdgeInsets.zero,
                ),
                ListTile(
                  title: const Text('Email address'),
                  subtitle: const Text('user@example.com'),
                  trailing: const Icon(Icons.edit),
                  contentPadding: EdgeInsets.zero,
                  onTap: () {},
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
