# 🚀 Quick Build Reference

## ✅ Ready to Build For

### 📱 Android
```bash
# APK (sideload/test)
flutter build apk --release

# App Bundle (Google Play)
flutter build appbundle --release
```

### 🪟 Windows
```bash
# Portable executable
flutter build windows --release

# MSIX installer (Store)
flutter pub run msix:create
```

### 🍎 iOS
```bash
flutter build ios --release
```

### 🌐 Web
```bash
flutter build web --release
```

---

## 📦 Build Outputs

| Platform | File Location | Size | Use |
|----------|--------------|------|-----|
| Android APK | `build/app/outputs/flutter-apk/` | ~70MB | Direct |
|  | `app-release.apk` |  | install |
| Android AAB | `build/app/outputs/bundle/release/` | ~50MB | Play |
|  | `app-release.aab` |  | Store |
| Windows EXE | `build/windows/x64/runner/Release/` | ~90MB | Portable |
|  | `BBQMonitor.exe` |  |  |
| Windows MSIX | `build/windows/x64/runner/Release/` | ~80MB | MS |
|  | `bbq_monitor.msix` |  | Store |

---

## 🎯 Current Status

✅ Android build configured (minSdk 21, BLE permissions)  
✅ Windows build configured (x64, Bluetooth capable)  
✅ iOS build ready  
✅ ProGuard enabled for Android optimization  
✅ MSIX packaging configured  
⚠️ Debug signing (change for production)

---

## 📝 Before Production Release

1. **Android**: Create keystore for Google Play signing
2. **Windows**: Get code signing certificate (optional)
3. **Both**: Update app icons
4. **Both**: Test on clean devices without Flutter

---

## 📖 Full Documentation

- `BUILD_GUIDE.md` - Complete overview
- `ANDROID_BUILD.md` - Android specifics & Play Store
- `WINDOWS_BUILD.md` - Windows specifics & MS Store
- `BLE_INTEGRATION.md` - Bluetooth setup

---

## 🆘 Troubleshooting

**Build fails?**
```bash
flutter clean
flutter pub get
flutter doctor -v
```

**Android issues?**  
Check `ANDROID_BUILD.md` → Signing section

**Windows issues?**  
Check `WINDOWS_BUILD.md` → Requirements
