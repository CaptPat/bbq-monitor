import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/license_service.dart';

class CookProfilesScreen extends StatefulWidget {
  const CookProfilesScreen({super.key});

  @override
  State<CookProfilesScreen> createState() => _CookProfilesScreenState();
}

class _CookProfilesScreenState extends State<CookProfilesScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Cook Profiles & Recipes'),
      ),
      body: Consumer<LicenseService>(
        builder: (context, licenseService, child) {
          if (!licenseService.isPremium) {
            return _buildPremiumRequired(context);
          }
          return _buildProfilesContent();
        },
      ),
      floatingActionButton: Consumer<LicenseService>(
        builder: (context, licenseService, child) {
          if (!licenseService.isPremium) {
            return const SizedBox.shrink();
          }
          return FloatingActionButton(
            onPressed: _createNewProfile,
            child: const Icon(Icons.add),
          );
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
              Icons.restaurant_menu,
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
              'Save and reuse cook profiles with Premium.',
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
                  _buildFeature(Icons.save, 'Save unlimited cook profiles'),
                  _buildFeature(Icons.restaurant, 'Pre-set recipes for common meats'),
                  _buildFeature(Icons.share, 'Share profiles with friends'),
                  _buildFeature(Icons.history, 'Track cook history'),
                  _buildFeature(Icons.note, 'Add notes and photos'),
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
          Expanded(
            child: Text(
              text,
              style: const TextStyle(fontSize: 16),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildProfilesContent() {
    final profiles = [
      {'name': 'Brisket Low & Slow', 'temp': 225, 'time': '12-16 hrs', 'icon': Icons.smoking_rooms},
      {'name': 'Pulled Pork', 'temp': 225, 'time': '10-12 hrs', 'icon': Icons.restaurant},
      {'name': 'Baby Back Ribs', 'temp': 250, 'time': '4-5 hrs', 'icon': Icons.set_meal},
      {'name': 'Chicken Breast', 'temp': 375, 'time': '45-60 min', 'icon': Icons.egg},
    ];

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Padding(
          padding: EdgeInsets.only(bottom: 16),
          child: Text(
            'My Saved Profiles',
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        ...profiles.map((profile) => _buildProfileCard(profile)),
        const SizedBox(height: 80), // Space for FAB
      ],
    );
  }

  Widget _buildProfileCard(Map<String, dynamic> profile) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: ListTile(
        contentPadding: const EdgeInsets.all(16),
        leading: CircleAvatar(
          backgroundColor: Colors.deepOrange.shade100,
          child: Icon(
            profile['icon'] as IconData,
            color: Colors.deepOrange,
          ),
        ),
        title: Text(
          profile['name'] as String,
          style: const TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
          ),
        ),
        subtitle: Padding(
          padding: const EdgeInsets.only(top: 8),
          child: Row(
            children: [
              Icon(Icons.thermostat, size: 16, color: Colors.grey[600]),
              const SizedBox(width: 4),
              Text('${profile['temp']}°F'),
              const SizedBox(width: 16),
              Icon(Icons.access_time, size: 16, color: Colors.grey[600]),
              const SizedBox(width: 4),
              Text(profile['time'] as String),
            ],
          ),
        ),
        trailing: PopupMenuButton(
          itemBuilder: (context) => [
            const PopupMenuItem(
              value: 'use',
              child: Row(
                children: [
                  Icon(Icons.play_arrow),
                  SizedBox(width: 8),
                  Text('Use Profile'),
                ],
              ),
            ),
            const PopupMenuItem(
              value: 'edit',
              child: Row(
                children: [
                  Icon(Icons.edit),
                  SizedBox(width: 8),
                  Text('Edit'),
                ],
              ),
            ),
            const PopupMenuItem(
              value: 'delete',
              child: Row(
                children: [
                  Icon(Icons.delete),
                  SizedBox(width: 8),
                  Text('Delete'),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _createNewProfile() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create Cook Profile'),
        content: const Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              decoration: InputDecoration(
                labelText: 'Profile Name',
                hintText: 'e.g., Brisket Low & Slow',
              ),
            ),
            SizedBox(height: 16),
            TextField(
              decoration: InputDecoration(
                labelText: 'Target Temperature (°F)',
                hintText: '225',
              ),
              keyboardType: TextInputType.number,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              // Save profile logic
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }
}
