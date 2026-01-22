# Windows Build Configuration

## Building Windows Application

### Development Build
```powershell
cd flutter_app
flutter build windows --release
# Output: build\windows\x64\runner\Release\BBQMonitor.exe
```

The executable and all dependencies will be in:
`build\windows\x64\runner\Release\`

### Testing the Build

```powershell
.\build\windows\x64\runner\Release\BBQMonitor.exe
```

## Creating MSIX Installer for Microsoft Store

### Prerequisites

1. Install `msix` package:

   ```powershell
   flutter pub add msix --dev
   ```

2. Update `pubspec.yaml` with MSIX configuration:

   ```yaml
msix_config:
  display_name: BBQ Monitor
  publisher_display_name: Your Company
  identity_name: com.bbqmonitor.app
  msix_version: 1.0.0.0
  logo_path: assets\icons\app_icon.png
  capabilities: 'internetClient,bluetooth'
  publisher: CN=YourPublisher
```

### Build MSIX
```powershell
flutter pub run msix:create
# Output: build\windows\x64\runner\Release\bbq_monitor.msix
```

### For Microsoft Store Submission
1. Get a Microsoft Store developer account
2. Create signing certificate or use Store signing
3. Build with proper certificate:
```powershell
flutter pub run msix:create --store
```

## Creating Traditional Installer (Inno Setup)

### Install Inno Setup
Download from: https://jrsoftware.org/isdl.php

### Create Installer Script
Save as `installer.iss`:
```innosetup
#define MyAppName "BBQ Monitor"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Your Company"
#define MyAppExeName "BBQMonitor.exe"
#define SourcePath "build\windows\x64\runner\Release"

[Setup]
AppId={{YOUR-GUID-HERE}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=build\windows\installer
OutputBaseFilename=BBQMonitor-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; \
  Description: "Create a desktop icon"; \
  GroupDescription: "Additional icons:"

[Files]
Source: "{#SourcePath}\{#MyAppExeName}"; \
  DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourcePath}\*"; DestDir: "{app}"; \
  Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; \
  Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; \
  Description: "Launch {#MyAppName}"; \
  Flags: nowait postinstall skipifsilent
```

### Build Installer
```powershell
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer.iss
# Output: build\windows\installer\BBQMonitor-Setup.exe
```

## Portable Distribution (No Installer)

Simply zip the Release folder:
```powershell
Compress-Archive -Path build\windows\x64\runner\Release\* -DestinationPath BBQMonitor-Portable.zip
```

Users extract and run `BBQMonitor.exe` directly.

## Current Configuration

- **Executable Name**: BBQMonitor.exe
- **App ID**: com.bbqmonitor.app
- **Architecture**: x64
- **Build Type**: Release (optimized)

## Distribution Options

### 1. Microsoft Store (Recommended)
- Automatic updates
- Trusted installation
- Requires MSIX package
- $19 one-time developer fee

### 2. Direct Download
- .exe installer via Inno Setup
- Manual updates
- Sign with code signing certificate (recommended)

### 3. Portable
- Zip file distribution
- No installation needed
- Good for tech-savvy users

## Code Signing (Recommended for Production)

To avoid "Unknown Publisher" warnings:

1. Get a code signing certificate from:
   - DigiCert
   - Sectigo
   - GlobalSign

2. Sign the executable:

   ```powershell
   signtool sign /f certificate.pfx /p password \
     /t http://timestamp.digicert.com \
     build\windows\x64\runner\Release\BBQMonitor.exe
```

## Testing on Clean System

Test the installer/portable on a Windows 11 VM without Flutter installed
to ensure all dependencies are bundled.

## Requirements for End Users

- Windows 10 (1809 or later) / Windows 11
- x64 processor
- Bluetooth adapter (for BLE connectivity)
- ~100MB disk space
