<p align="center">
  <img src="icon.png" width="128" alt="App Icon">
</p>

<h1 align="center">HA Win</h1>

<p align="center">
  <a href="https://github.com/derDeno/HA-Win/releases/latest">
    <img alt="Latest release" src="https://img.shields.io/github/v/release/derDeno/HA-Win?display_name=tag&sort=semver">
  </a>
  <a href="https://github.com/derDeno/HA-Win/actions/workflows/release-draft.yml">
    <img alt="Build" src="https://img.shields.io/github/actions/workflow/status/derDeno/HA-Win/release-draft.yml">
  </a>
</p>

Windows 10/11 MQTT bridge for Home Assistant with built-in discovery.

## Highlights
- MQTT discovery for restart/shutdown/standby buttons
- Home Assistant MQTT notify integration to deliver PC notifications
- Tray icon and background operation
- Simple settings UI with auto-start toggle

## Requirements
- Home Assistant with the MQTT integration configured
- Windows 10/11
- .NET 10 runtime (SDK only required for building)

## Installation

### 1. Installer
- Donwload the latest `HA-Win-Setup.exe` file and execute it.

### 2. Manual installation
- Download the zip and copy the whole folder to the program files directory and execute the `HAWin.exe`

### 3. Compile yourself
- Donwload the source code
- run 
```powershell
dotnet build .\src\HaWin\HaWin.csproj
```

---

## Quick start
1. Run HA Win and open settings from the tray icon.
2. Configure your MQTT broker and save.
3. Home Assistant will discover a new device and entities automatically.

## MQTT topics
Device namespace uses a sanitized machine name: `ha-win/<device-id>`.

### Buttons (auto created via discovery)
- `ha-win/<device-id>/restart/set`
- `ha-win/<device-id>/shutdown/set`
- `ha-win/<device-id>/standby/set`

### Notifications
- Command topic: `ha-win/<device-id>/notify`

Payload options:
- Plain text: `Hello from Home Assistant`
- JSON: `{ "title": "Door", "message": "Front door opened" }`


## Update
The program has a check for updates button. This will check for a new version and if available it will install it.
You can also activate the auto check for updates in the App tab.


## Uninstall
When installed with the installer, run the uninstall from the settings page or manually from the program files directory.