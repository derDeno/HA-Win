# HaWin

Windows 10/11 MQTT bridge for Home Assistant with built-in discovery.

## Highlights
- MQTT discovery for restart/shutdown/standby buttons
- Home Assistant MQTT notify integration to deliver PC notifications
- Tray icon and background operation
- Simple settings UI with auto-start toggle

## Requirements
- Home Assistant with the MQTT integration configured
- Windows 10/11
- .NET 8 runtime (SDK only required for building)

## Quick start
1. Run HaWin and open settings from the tray icon.
2. Configure your MQTT broker and save.
3. Home Assistant will discover a new device and entities automatically.

## MQTT topics
Device namespace uses a sanitized machine name: `ha-win/<device-id>`.

### Buttons (created via discovery)
- `ha-win/<device-id>/restart/set`
- `ha-win/<device-id>/shutdown/set`
- `ha-win/<device-id>/standby/set`

### Notifications
- Command topic: `ha-win/<device-id>/notify`

Payload options:
- Plain text: `Hello from Home Assistant`
- JSON: `{ "title": "Door", "message": "Front door opened" }`

## Home Assistant example
```yaml
service: mqtt.publish
data:
  topic: "ha-win/<device-id>/notify"
  payload: '{"title":"Reminder","message":"Time to stretch"}'
```

## Build
```powershell
dotnet build .\src\HaWin\HaWin.csproj
```

## Links
- https://github.com/derDeno/HA-Win
