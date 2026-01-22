# BBQ Monitor Flutter App

Cross-platform mobile and desktop client for BBQ Monitor, supporting Android
and Windows with native payment integration.

## Features

- **Real-time Temperature Monitoring**: Live updates from MEATER/MeatStick devices
- **Cross-Platform**: Single codebase for Android, Windows, iOS, and Web
- **Premium Upgrade**: In-app purchases with platform-specific payment providers
  - Google Play billing on Android (30% fee)
  - Stripe checkout on Windows (2.9% + $0.30 fee)
- **Rust FFI Integration**: Core business logic from Rust crate for performance
- **Beautiful UI**: Material Design 3 with light/dark themes

## Premium Features

- ☁️ Cloud sync across devices
- 📊 Unlimited history retention
- 🍖 Save cook profiles & recipes
- 📈 Advanced analytics & trends
- 🔔 SMS & email alerts
- ⚡ Real-time multi-device monitoring

## Architecture

```
flutter_app/
├── lib/
│   ├── main.dart                    # App entry point
│   ├── screens/
│   │   ├── home_screen.dart         # Device dashboard
│   │   └── premium_screen.dart      # Upgrade UI
│   ├── services/
│   │   ├── license_service.dart     # License validation via FFI
│   │   └── payment_service.dart     # Platform-abstracted payments
│   └── theme/
│       └── app_theme.dart           # BBQ-themed colors
└── pubspec.yaml                     # Dependencies
```

## Setup Instructions

### Prerequisites

- Flutter SDK 3.0+
- Rust toolchain (for building native library)
- Android SDK (for Android builds)
- Visual Studio 2019+ with C++ tools (for Windows builds)

### 1. Install Flutter Dependencies

```bash
cd flutter_app
flutter pub get
```

### 2. Build Rust Native Library

#### For Android (ARM64)

```bash
# Install Android NDK target
rustup target add aarch64-linux-android

# Build
cargo build --release --target aarch64-linux-android --lib

# Copy to Android jniLibs
mkdir -p flutter_app/android/app/src/main/jniLibs/arm64-v8a
cp target/aarch64-linux-android/release/libbbq_monitor.so \
   flutter_app/android/app/src/main/jniLibs/arm64-v8a/
```

#### For Android (x86_64 emulator)

```bash
rustup target add x86_64-linux-android
cargo build --release --target x86_64-linux-android --lib
mkdir -p flutter_app/android/app/src/main/jniLibs/x86_64
cp target/x86_64-linux-android/release/libbbq_monitor.so \
   flutter_app/android/app/src/main/jniLibs/x86_64/
```

#### For Windows

```bash
# Build
cargo build --release --lib

# Copy to Windows runner
mkdir -p flutter_app/windows/runner
cp target/release/bbq_monitor.dll flutter_app/windows/runner/
```

### 3. Configure Payment Providers

#### Google Play (Android)

1. Create app in Google Play Console
2. Set up in-app products:
   - `bbq_premium_monthly`: $4.99/month
   - `bbq_premium_yearly`: $49.99/year
   - `bbq_premium_lifetime`: $79.99 one-time
3. Add service account for purchase verification
4. Update `android/app/build.gradle` with billing library version

#### Stripe (Windows)

1. Create Stripe account and get API keys
2. Set up products and prices in Stripe Dashboard
3. Update `lib/services/payment_service.dart` with your Stripe publishable key
4. Configure webhook endpoint for purchase completion

### 4. Build and Run

#### Android

```bash
flutter run -d android
```

Or build APK:

```bash
flutter build apk --release
```

#### Windows

```bash
flutter run -d windows
```

Or build installer:

```bash
flutter build windows --release
```

## FFI Integration

The app uses `flutter_rust_bridge` to call Rust functions for license validation:

```dart
// Dart side
final isValid = _validateLicenseWithRust(licenseKey);
```

```rust
// Rust side (src/lib.rs)
#[no_mangle]
pub extern "C" fn validate_license(key: *const c_char) -> i8 {
    // Validate license and return 1 for valid, 0 for invalid
}
```

## Payment Flow

### Android (Google Play)

1. User taps "Choose Plan" on Premium screen
2. `GooglePlayPaymentService.purchasePremium()` initiates purchase
3. Google Play billing dialog shown to user
4. On success, `_verifyPurchase()` calls backend API
5. Backend verifies with Google, generates license key
6. License key returned to app and activated via FFI

### Windows (Stripe)

1. User taps "Choose Plan" on Premium screen
2. `StripePaymentService.purchasePremium()` calls backend for checkout URL
3. Backend creates Stripe checkout session
4. Browser launched with Stripe-hosted checkout
5. User completes payment on Stripe
6. Stripe webhook notifies backend, generates license key
7. License key emailed to user
8. User enters key in app to activate

## Development

### Running Tests

```bash
flutter test
```

### Debugging FFI

Enable verbose logging in `license_service.dart`:

```dart
print('FFI Library path: $libraryPath');
print('Validation result: $result');
```

### Hot Reload

Flutter hot reload works for UI changes. For Rust changes:

1. Rebuild Rust library: `cargo build --release --lib`
2. Copy new library to platform folder
3. Restart app (hot restart not sufficient for native changes)

## Troubleshooting

### "License validation failed"

- Ensure Rust library is built and in correct location
- Check library loading path in `license_service.dart`
- Verify license key format (base64 encoded)

### "Purchase failed"

- **Android**: Check Google Play Console configuration, test with sandbox account
- **Windows**: Verify Stripe API keys, check backend logs for webhook delivery

### "Library not found" errors

- Rebuild Rust library for target platform
- Verify library copied to correct folder (jniLibs for Android, runner for Windows)
- Check `flutter doctor` for missing toolchains

## License

See [LICENSE](../LICENSE) for details.
