import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:in_app_purchase/in_app_purchase.dart';
import 'package:flutter_stripe/flutter_stripe.dart';
import 'package:url_launcher/url_launcher.dart';
import 'license_service.dart';

enum PaymentPlatform {
  googlePlay,
  stripe,
  paddle,
}

enum PremiumTier {
  free,
  monthly,
  yearly,
  lifetime,
}

abstract class PaymentService with ChangeNotifier {
  static PaymentService? _instance;
  
  static PaymentService get instance {
    _instance ??= _createPlatformService();
    return _instance!;
  }
  
  static PaymentService _createPlatformService() {
    if (Platform.isAndroid) {
      return GooglePlayPaymentService();
    } else if (Platform.isWindows) {
      return StripePaymentService();
    }
    throw UnsupportedError('Platform not supported');
  }
  
  Future<void> initialize();
  Future<void> purchasePremium(PremiumTier tier);
  Future<bool> restorePurchases();
  Future<List<PremiumProduct>> getProducts();
  
  bool get isProcessing;
  String? get error;
}

class PremiumProduct {
  final String id;
  final String title;
  final String description;
  final String price;
  final PremiumTier tier;
  
  PremiumProduct({
    required this.id,
    required this.title,
    required this.description,
    required this.price,
    required this.tier,
  });
}

// ============================================================================
// Google Play Implementation (Android)
// ============================================================================

class GooglePlayPaymentService extends PaymentService {
  final InAppPurchase _iap = InAppPurchase.instance;
  bool _isProcessing = false;
  String? _error;
  
  static const String productMonthly = 'bbq_premium_monthly';
  static const String productYearly = 'bbq_premium_yearly';
  static const String productLifetime = 'bbq_premium_lifetime';
  
  @override
  bool get isProcessing => _isProcessing;
  
  @override
  String? get error => _error;
  
  @override
  Future<void> initialize() async {
    final available = await _iap.isAvailable();
    if (!available) {
      _error = 'In-app purchases not available';
      return;
    }
    
    // Listen to purchase updates
    _iap.purchaseStream.listen(_onPurchaseUpdate, onError: (error) {
      _error = error.toString();
      notifyListeners();
    });
  }
  
  @override
  Future<List<PremiumProduct>> getProducts() async {
    final response = await _iap.queryProductDetails({
      productMonthly,
      productYearly,
      productLifetime,
    });
    
    if (response.error != null) {
      _error = response.error!.message;
      notifyListeners();
      return [];
    }
    
    return response.productDetails.map((product) {
      PremiumTier tier;
      if (product.id == productMonthly) {
        tier = PremiumTier.monthly;
      } else if (product.id == productYearly) {
        tier = PremiumTier.yearly;
      } else {
        tier = PremiumTier.lifetime;
      }
      
      return PremiumProduct(
        id: product.id,
        title: product.title,
        description: product.description,
        price: product.price,
        tier: tier,
      );
    }).toList();
  }
  
  @override
  Future<void> purchasePremium(PremiumTier tier) async {
    _isProcessing = true;
    _error = null;
    notifyListeners();
    
    final productId = _getProductId(tier);
    
    final productDetails = await _iap.queryProductDetails({productId});
    final purchaseParam = PurchaseParam(
      productDetails: productDetails.productDetails.first,
    );
    
    try {
      if (tier == PremiumTier.lifetime) {
        await _iap.buyNonConsumable(purchaseParam: purchaseParam);
      } else {
        await _iap.buyNonConsumable(purchaseParam: purchaseParam);
      }
    } catch (e) {
      _error = e.toString();
      _isProcessing = false;
      notifyListeners();
    }
  }
  
  @override
  Future<bool> restorePurchases() async {
    try {
      await _iap.restorePurchases();
      return true;
    } catch (e) {
      _error = e.toString();
      notifyListeners();
      return false;
    }
  }
  
  void _onPurchaseUpdate(List<PurchaseDetails> purchases) async {
    for (final purchase in purchases) {
      if (purchase.status == PurchaseStatus.purchased) {
        // Verify purchase with backend and get license key
        final licenseKey = await _verifyPurchase(purchase);
        
        if (licenseKey != null) {
          await LicenseService.instance.activateLicense(licenseKey);
        }
        
        // Complete the purchase
        if (purchase.pendingCompletePurchase) {
          await _iap.completePurchase(purchase);
        }
      } else if (purchase.status == PurchaseStatus.error) {
        _error = purchase.error?.message ?? 'Purchase failed';
      }
      
      _isProcessing = false;
      notifyListeners();
    }
  }
  
  Future<String?> _verifyPurchase(PurchaseDetails purchase) async {
    // Call your backend to verify and get license key
    // This would call your Lambda function
    try {
      // TODO: Implement API call
      // final response = await http.post(
      //   Uri.parse('https://api.bbqmonitor.com/purchases/verify'),
      //   body: {
      //     'platform': 'android',
      //     'purchase_token': purchase.verificationData.serverVerificationData,
      //   },
      // );
      // return json.decode(response.body)['license_key'];
      
      return null; // Placeholder
    } catch (e) {
      debugPrint('Purchase verification failed: $e');
      return null;
    }
  }
  
  String _getProductId(PremiumTier tier) {
    switch (tier) {
      case PremiumTier.monthly:
        return productMonthly;
      case PremiumTier.yearly:
        return productYearly;
      case PremiumTier.lifetime:
        return productLifetime;
      default:
        throw ArgumentError('Invalid tier');
    }
  }
}

// ============================================================================
// Stripe Implementation (Windows/Web)
// ============================================================================

class StripePaymentService extends PaymentService {
  bool _isProcessing = false;
  String? _error;
  
  @override
  bool get isProcessing => _isProcessing;
  
  @override
  String? get error => _error;
  
  @override
  Future<void> initialize() async {
    Stripe.publishableKey = 'pk_test_YOUR_KEY_HERE';
    await Stripe.instance.applySettings();
  }
  
  @override
  Future<List<PremiumProduct>> getProducts() async {
    // Return static products for Windows
    return [
      PremiumProduct(
        id: 'monthly',
        title: 'Premium Monthly',
        description: 'All premium features, billed monthly',
        price: '\$4.99/month',
        tier: PremiumTier.monthly,
      ),
      PremiumProduct(
        id: 'yearly',
        title: 'Premium Annual',
        description: 'All premium features, save 17%',
        price: '\$49.99/year',
        tier: PremiumTier.yearly,
      ),
      PremiumProduct(
        id: 'lifetime',
        title: 'Premium Lifetime',
        description: 'One-time payment, lifetime access',
        price: '\$79.99',
        tier: PremiumTier.lifetime,
      ),
    ];
  }
  
  @override
  Future<void> purchasePremium(PremiumTier tier) async {
    _isProcessing = true;
    _error = null;
    notifyListeners();
    
    try {
      // Get checkout URL from your backend
      final checkoutUrl = await _getCheckoutUrl(tier);
      
      // Open Stripe Checkout in browser
      final uri = Uri.parse(checkoutUrl);
      if (await canLaunchUrl(uri)) {
        await launchUrl(uri, mode: LaunchMode.externalApplication);
      } else {
        _error = 'Could not open checkout page';
      }
    } catch (e) {
      _error = e.toString();
    } finally {
      _isProcessing = false;
      notifyListeners();
    }
  }
  
  @override
  Future<bool> restorePurchases() async {
    // For Windows, users re-enter license key manually
    return false;
  }
  
  Future<String> _getCheckoutUrl(PremiumTier tier) async {
    // TODO: Call your backend to create Stripe Checkout session
    // final response = await http.post(
    //   Uri.parse('https://api.bbqmonitor.com/checkout/create'),
    //   body: {'tier': tier.toString()},
    // );
    // return json.decode(response.body)['url'];
    
    // Placeholder - return hosted checkout page
    return 'https://premium.bbqmonitor.com/checkout?tier=$tier';
  }
}
