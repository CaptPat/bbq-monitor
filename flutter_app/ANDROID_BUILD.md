# Android Release Build Configuration

## Building Release APK/AAB

### Option 1: Debug Signed (Current - for testing)
```bash
cd flutter_app
flutter build apk --release
# Output: build/app/outputs/flutter-apk/app-release.apk

# Or build App Bundle for Google Play
flutter build appbundle --release
# Output: build/app/outputs/bundle/release/app-release.aab
```

### Option 2: Production Signed (for Google Play)

1. **Create a keystore** (one-time setup):

   ```bash
   keytool -genkey -v -keystore ~/bbq-monitor-key.jks -keyalg RSA \
     -keysize 2048 -validity 10000 -alias bbq-monitor
   ```

2. **Create `android/key.properties`**:

   ```properties
   storePassword=<your-store-password>
   keyPassword=<your-key-password>
   keyAlias=bbq-monitor
   storeFile=<path-to>/bbq-monitor-key.jks
   ```

3. **Update `android/app/build.gradle.kts`** (add before `android` block):

   ```kotlin
   val keystoreProperties = Properties()
   val keystorePropertiesFile = rootProject.file("key.properties")
   if (keystorePropertiesFile.exists()) {
       keystoreProperties.load(FileInputStream(keystorePropertiesFile))
   }

   android {
       // ... existing config ...
       
       signingConfigs {
           create("release") {
               keyAlias = keystoreProperties["keyAlias"] as String
               keyPassword = keystoreProperties["keyPassword"] as String
               storeFile = file(keystoreProperties["storeFile"] as String)
               storePassword = keystoreProperties["storePassword"] as String
           }
       }
       
       buildTypes {
           release {
               signingConfig = signingConfigs.getByName("release")
               // ... rest of config
           }
       }
   }
   ```

4. **Build signed release**:

   ```bash
   flutter build appbundle --release
   ```

## Google Play Console Upload

1. Go to [Google Play Console](https://play.google.com/console)
2. Create new app or select existing
3. Upload `app-release.aab` from `build/app/outputs/bundle/release/`
4. Fill in store listing, content rating, pricing
5. Submit for review

## Current Configuration

- **App ID**: `com.bbqmonitor.app`
- **Min SDK**: 21 (Android 5.0)
- **Target SDK**: Latest from Flutter
- **Version**: 1.0.0+1
- **ProGuard**: Enabled for release builds
- **Current Signing**: Debug keys (for development)

## Testing Release Build

Install the debug-signed APK on your device:
```bash
flutter build apk --release
adb install build/app/outputs/flutter-apk/app-release.apk
```

## Important Files
- `android/app/build.gradle.kts` - Build configuration
- `android/app/src/main/AndroidManifest.xml` - Permissions and app config
- `android/app/proguard-rules.pro` - Code obfuscation rules
- `android/key.properties` - Signing keys (DO NOT commit to git!)
