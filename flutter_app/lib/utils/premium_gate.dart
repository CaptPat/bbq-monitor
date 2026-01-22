import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/license_service.dart';
import '../screens/premium_screen.dart';

/// Helper class for gating premium features
class PremiumGate {
  /// Shows premium upgrade dialog if user is not premium
  /// Returns true if user has premium access, false otherwise
  static bool checkAccess(BuildContext context, {String? featureName}) {
    final licenseService = Provider.of<LicenseService>(context, listen: false);
    
    if (licenseService.isPremium) {
      return true;
    }
    
    _showPremiumDialog(context, featureName: featureName);
    return false;
  }
  
  /// Navigate to a premium feature screen (shows upgrade dialog if not premium)
  /// Returns true if navigation happened, false if blocked
  static Future<bool> navigateIfPremium(
    BuildContext context,
    Widget destination, {
    String? featureName,
  }) async {
    if (checkAccess(context, featureName: featureName)) {
      await Navigator.push(
        context,
        MaterialPageRoute(builder: (context) => destination),
      );
      return true;
    }
    return false;
  }
  
  static void _showPremiumDialog(BuildContext context, {String? featureName}) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Row(
          children: [
            Icon(
              Icons.workspace_premium,
              color: Colors.amber,
              size: 28,
            ),
            SizedBox(width: 12),
            Text('Premium Feature'),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (featureName != null) ...[
              Text(
                featureName,
                style: const TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 8),
            ],
            const Text(
              'This feature requires a Premium subscription.',
              style: TextStyle(fontSize: 16),
            ),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.blue.shade50,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _buildFeatureItem(Icons.cloud_sync, 'Cloud sync across devices'),
                  _buildFeatureItem(Icons.history, 'Unlimited history'),
                  _buildFeatureItem(Icons.notifications_active, 'Custom alerts'),
                  _buildFeatureItem(Icons.restaurant, 'Cook profiles & recipes'),
                ],
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Not Now'),
          ),
          ElevatedButton.icon(
            onPressed: () {
              Navigator.pop(context);
              Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => const PremiumScreen()),
              );
            },
            icon: const Icon(Icons.arrow_upward, size: 18),
            label: const Text('Upgrade'),
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.amber,
              foregroundColor: Colors.white,
            ),
          ),
        ],
      ),
    );
  }
  
  static Widget _buildFeatureItem(IconData icon, String text) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Icon(icon, size: 16, color: Colors.blue.shade700),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              text,
              style: TextStyle(
                fontSize: 14,
                color: Colors.blue.shade900,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
