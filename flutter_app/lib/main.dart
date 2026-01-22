import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'screens/home_screen.dart';
import 'screens/premium_screen.dart';
import 'services/license_service.dart';
import 'services/payment_service.dart';
import 'services/rust_ble_service.dart';
import 'services/settings_service.dart';
import 'theme/app_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize services
  try {
    await LicenseService.instance.initialize();
  } catch (e) {
    debugPrint('License service init failed: $e');
  }
  
  try {
    await PaymentService.instance.initialize();
  } catch (e) {
    debugPrint('Payment service init failed: $e');
  }
  
  try {
    await RustBLEService.instance.initialize();
  } catch (e) {
    debugPrint('BLE service init failed: $e');
  }
  
  runApp(const BbqMonitorApp());
}

class BbqMonitorApp extends StatelessWidget {
  const BbqMonitorApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: LicenseService.instance),
        ChangeNotifierProvider.value(value: PaymentService.instance),
        ChangeNotifierProvider.value(value: RustBLEService.instance),
        ChangeNotifierProvider(create: (_) => SettingsService()),
      ],
      child: MaterialApp(
        title: 'BBQ Monitor',
        theme: AppTheme.lightTheme,
        darkTheme: AppTheme.darkTheme,
        themeMode: ThemeMode.system,
        home: const HomeScreen(),
        routes: {
          '/premium': (context) => const PremiumScreen(),
        },
      ),
    );
  }
}
