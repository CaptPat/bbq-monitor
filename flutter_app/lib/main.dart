import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'screens/home_screen.dart';
import 'screens/premium_screen.dart';
import 'services/license_service.dart';
import 'services/payment_service.dart';
import 'services/ble_service.dart';
import 'theme/app_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize services
  await LicenseService.instance.initialize();
  await PaymentService.instance.initialize();
  
  runApp(const BbqMonitorApp());
}

class BbqMonitorApp extends StatelessWidget {
  const BbqMonitorApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => LicenseService.instance),
        ChangeNotifierProvider(create: (_) => PaymentService.instance),
        ChangeNotifierProvider(create: (_) => BLEService()),
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
