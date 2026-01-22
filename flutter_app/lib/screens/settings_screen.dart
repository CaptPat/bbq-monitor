import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/license_service.dart';
import '../services/settings_service.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: Consumer2<LicenseService, SettingsService>(
        builder: (context, licenseService, settingsService, child) {
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              // Cloud Sync Section
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(
                            Icons.cloud,
                            color: Theme.of(context).primaryColor,
                          ),
                          const SizedBox(width: 8),
                          const Text(
                            'Cloud Sync',
                            style: TextStyle(
                              fontSize: 18,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      
                      // Offline Mode Toggle
                      SwitchListTile(
                        title: const Text('Offline Mode'),
                        subtitle: Text(
                          licenseService.isPremium
                              ? 'Disable cloud sync while keeping premium features'
                              : 'Premium required for cloud sync',
                        ),
                        value: settingsService.offlineMode,
                        onChanged: licenseService.isPremium
                            ? (value) => settingsService.setOfflineMode(value)
                            : null,
                        contentPadding: EdgeInsets.zero,
                      ),
                      
                      if (licenseService.isPremium) ...[
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: settingsService.offlineMode
                                ? Colors.orange.shade50
                                : Colors.green.shade50,
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Row(
                            children: [
                              Icon(
                                settingsService.offlineMode
                                    ? Icons.cloud_off
                                    : Icons.cloud_done,
                                size: 16,
                                color: settingsService.offlineMode
                                    ? Colors.orange
                                    : Colors.green,
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  settingsService.offlineMode
                                      ? 'Cloud sync disabled. Data stored locally only.'
                                      : 'Cloud sync active. Data syncs across devices.',
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: settingsService.offlineMode
                                        ? Colors.orange.shade800
                                        : Colors.green.shade800,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                      
                      if (!licenseService.isPremium) ...[
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(12),
                          decoration: BoxDecoration(
                            color: Colors.blue.shade50,
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Row(
                            children: [
                              Icon(
                                Icons.info_outline,
                                size: 16,
                                color: Colors.blue.shade700,
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  'Upgrade to Premium to sync data across devices and access unlimited history.',
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Colors.blue.shade800,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
              
              const SizedBox(height: 16),
              
              // About Section
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(
                            Icons.info_outline,
                            color: Theme.of(context).primaryColor,
                          ),
                          const SizedBox(width: 8),
                          const Text(
                            'About',
                            style: TextStyle(
                              fontSize: 18,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      
                      const ListTile(
                        title: Text('Version'),
                        subtitle: Text('1.0.0'),
                        contentPadding: EdgeInsets.zero,
                      ),
                      
                      ListTile(
                        title: const Text('License'),
                        subtitle: Text(
                          licenseService.isPremium
                              ? 'Premium (Active)'
                              : 'Free',
                        ),
                        trailing: licenseService.isPremium
                            ? const Icon(
                                Icons.workspace_premium,
                                color: Colors.amber,
                              )
                            : null,
                        contentPadding: EdgeInsets.zero,
                      ),
                    ],
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}
