# BBQ Monitor - Build & Distribution Guide

## Quick Start

### Android APK (for testing)
```bash
cd flutter_app
flutter build apk --release
```
📦 Output: `build/app/outputs/flutter-apk/app-release.apk`

### Android App Bundle (for Google Play)
```bash
flutter build appbundle --release
```
📦 Output: `build/app/outputs/bundle/release/app-release.aab`

### Windows Executable
```bash
flutter build windows --release
```
📦 Output: `build\windows\x64\runner\Release\BBQMonitor.exe`

### Windows MSIX (for Microsoft Store)
```bash
flutter pub get
flutter pub run msix:create
```
📦 Output: `build\windows\x64\runner\Release\bbq_monitor.msix`

## Platform Status

| Platform | Status | Build Command | Output | Distribution |
|----------|--------|---------------|--------|--------------|
| Android | ✅ Ready | `flutter build apk` | APK | Direct download |
| Android | ✅ Ready | `flutter build appbundle` | AAB | Google Play |
| Windows | ✅ Ready | `flutter build windows` | .exe | Portable |
| Windows | ✅ Ready | `flutter pub run msix:create` | MSIX | Microsoft Store |
| iOS | ✅ Ready | `flutter build ios` | IPA | App Store |
| Web | ✅ Ready | `flutter build web` | HTML/JS | Web hosting |

## Build Types

### Debug (Development)
```bash
flutter run  # Hot reload enabled
```

### Release (Production)
```bash
flutter build <platform> --release  # Optimized, no debug info
```

### Profile (Performance Testing)
```bash
flutter build <platform> --profile
```

## File Sizes (Estimated)

- **Android APK**: ~50-80 MB
- **Android AAB**: ~40-60 MB (Play Store optimized)
- **Windows Portable**: ~80-120 MB
- **Windows MSIX**: ~70-100 MB

## Distribution Channels

### Android
1. **Google Play Store** (Recommended)
   - Professional appearance
   - Automatic updates
   - Larger audience reach
   - Requires $25 one-time developer fee

2. **Direct APK Download**
   - No store fees
   - Manual updates
   - Users must enable "Install from unknown sources"

### Windows
1. **Microsoft Store** (Recommended)
   - Professional appearance
   - Automatic updates
   - Requires $19 one-time developer fee
   - Users trust Store apps

2. **Direct .exe Download**
   - No store fees
   - Manual updates
   - Code signing recommended ($100-400/year)

3. **Portable Zip**
   - No installation needed
   - Good for tech users

## Production Checklist

### Before First Release
- [ ] Set unique app IDs (currently: `com.bbqmonitor.app`)
- [ ] Create app icons (Android: multiple sizes, Windows: .ico)
- [ ] Set up code signing for Android (keystore)
- [ ] Consider code signing for Windows (certificate)
- [ ] Test on clean Windows 11 and Android devices
- [ ] Prepare store listings (descriptions, screenshots)
- [ ] Set up backend APIs (payment, license validation)

### For Each Update
- [ ] Update version in pubspec.yaml
- [ ] Test on real devices
- [ ] Build release versions
- [ ] Test release builds
- [ ] Upload to stores or distribution site
- [ ] Update documentation

## Current Configuration

**App Information:**
- Name: BBQ Monitor
- Package: com.bbqmonitor.app
- Version: 1.0.0+1

**Android:**
- Min SDK: 21 (Android 5.0)
- Target SDK: Latest
- Permissions: Bluetooth, Location (for BLE), Internet

**Windows:**
- Min OS: Windows 10 1809+
- Architecture: x64
- Requirements: Bluetooth adapter

## Detailed Documentation

- [ANDROID_BUILD.md](ANDROID_BUILD.md) - Complete Android build & signing guide
- [WINDOWS_BUILD.md](WINDOWS_BUILD.md) - Complete Windows build & packaging guide
- [BLE_INTEGRATION.md](BLE_INTEGRATION.md) - Bluetooth setup & troubleshooting

## Support

For build issues:
1. Check Flutter doctor: `flutter doctor -v`
2. Clean build: `flutter clean && flutter pub get`
3. Review platform-specific docs above
