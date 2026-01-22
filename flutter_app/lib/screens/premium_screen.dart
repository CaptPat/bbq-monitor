import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:io';
import '../services/license_service.dart';
import '../services/payment_service.dart';

class PremiumScreen extends StatefulWidget {
  const PremiumScreen({super.key});

  @override
  State<PremiumScreen> createState() => _PremiumScreenState();
}

class _PremiumScreenState extends State<PremiumScreen> {
  List<PremiumProduct> _products = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadProducts();
  }

  Future<void> _loadProducts() async {
    final products = await PaymentService.instance.getProducts();
    setState(() {
      _products = products;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Upgrade to Premium'),
        elevation: 0,
      ),
      body: Consumer<LicenseService>(
        builder: (context, licenseService, child) {
          if (licenseService.isPremium) {
            return _buildPremiumActive(licenseService);
          }
          return _buildUpgradeOptions();
        },
      ),
    );
  }

  Widget _buildPremiumActive(LicenseService licenseService) {
    final license = licenseService.license;
    
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.check_circle,
              size: 80,
              color: Colors.green,
            ),
            const SizedBox(height: 24),
            const Text(
              '🎉 Premium Active!',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 16),
            if (license.daysUntilExpiry != null)
              Text(
                'Expires in ${license.daysUntilExpiry} days',
                style: TextStyle(
                  fontSize: 16,
                  color: Colors.grey[600],
                ),
              )
            else
              Text(
                'Lifetime License',
                style: TextStyle(
                  fontSize: 16,
                  color: Colors.grey[600],
                ),
              ),
            const SizedBox(height: 32),
            _buildFeatureList(enabled: true),
          ],
        ),
      ),
    );
  }

  Widget _buildUpgradeOptions() {
    return SingleChildScrollView(
      child: Column(
        children: [
          // Header
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(32),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  Theme.of(context).primaryColor,
                  Theme.of(context).primaryColor.withValues(alpha: 0.7),
                ],
              ),
            ),
            child: Column(
              children: [
                const Icon(
                  Icons.workspace_premium,
                  size: 64,
                  color: Colors.white,
                ),
                const SizedBox(height: 16),
                const Text(
                  'Unlock Premium Features',
                  style: TextStyle(
                    fontSize: 28,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  'Get the most out of BBQ Monitor',
                  style: TextStyle(
                    fontSize: 16,
                    color: Colors.white.withValues(alpha: 0.9),
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 32),

          // Feature Comparison
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Premium Features',
                  style: TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 16),
                _buildFeatureList(enabled: true),
              ],
            ),
          ),

          const SizedBox(height: 32),

          // Pricing Cards
          if (_loading)
            const Center(child: CircularProgressIndicator())
          else
            ..._products.map((product) => _buildPricingCard(product)),

          const SizedBox(height: 24),

          // Restore Purchases (Android only)
          if (Platform.isAndroid)
            TextButton(
              onPressed: _restorePurchases,
              child: const Text('Restore Purchases'),
            ),

          const SizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildFeatureList({required bool enabled}) {
    final features = [
      {'icon': Icons.cloud_sync, 'text': 'Cloud sync across devices'},
      {'icon': Icons.history, 'text': 'Unlimited history retention'},
      {'icon': Icons.restaurant, 'text': 'Save cook profiles & recipes'},
      {'icon': Icons.analytics, 'text': 'Advanced analytics & trends'},
      {'icon': Icons.notifications_active, 'text': 'SMS & email alerts'},
      {'icon': Icons.speed, 'text': 'Real-time multi-device monitoring'},
    ];

    return Column(
      children: features.map((feature) {
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: Row(
            children: [
              Icon(
                feature['icon'] as IconData,
                color: enabled ? Colors.green : Colors.grey,
                size: 28,
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Text(
                  feature['text'] as String,
                  style: const TextStyle(fontSize: 16),
                ),
              ),
              if (enabled)
                const Icon(Icons.check, color: Colors.green),
            ],
          ),
        );
      }).toList(),
    );
  }

  Widget _buildPricingCard(PremiumProduct product) {
    final isPopular = product.tier == PremiumTier.yearly;
    
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
      decoration: BoxDecoration(
        border: Border.all(
          color: isPopular ? Theme.of(context).primaryColor : Colors.grey[300]!,
          width: isPopular ? 2 : 1,
        ),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Stack(
        children: [
          if (isPopular)
            Positioned(
              top: 0,
              right: 0,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                decoration: BoxDecoration(
                  color: Theme.of(context).primaryColor,
                  borderRadius: const BorderRadius.only(
                    topRight: Radius.circular(14),
                    bottomLeft: Radius.circular(14),
                  ),
                ),
                child: const Text(
                  'BEST VALUE',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ),
          Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  product.title,
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  product.description,
                  style: TextStyle(
                    fontSize: 14,
                    color: Colors.grey[600],
                  ),
                ),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Text(
                      product.price,
                      style: const TextStyle(
                        fontSize: 28,
                        fontWeight: FontWeight.bold,
                        color: Colors.blue,
                      ),
                    ),
                    const Spacer(),
                    ElevatedButton(
                      onPressed: () => _purchase(product.tier),
                      style: ElevatedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 32,
                          vertical: 12,
                        ),
                      ),
                      child: const Text(
                        'Choose Plan',
                        style: TextStyle(fontSize: 16),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _purchase(PremiumTier tier) async {
    try {
      await PaymentService.instance.purchasePremium(tier);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Purchase failed: $e')),
        );
      }
    }
  }

  Future<void> _restorePurchases() async {
    final success = await PaymentService.instance.restorePurchases();
    
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            success
                ? 'Purchases restored successfully'
                : 'No purchases to restore',
          ),
        ),
      );
    }
  }
}
