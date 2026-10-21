<p align="center"><img src="icon.png" width="128" alt="HA Win icon"></p>
<h1 align="center">HA Win</h1>

A small native Windows 10/11 client connecting your PC to Home Assistant over MQTT.
This branch replaces the previous C#/WPF application with Rust. No .NET runtime,
WebView, or browser is required.

## Features

- Home Assistant MQTT discovery: restart, shutdown, standby, and notifications
- Native settings window with Status, MQTT, and App tabs
- Tray icon: click to open settings, right-click to open or exit
- Closing the settings window keeps the client running in the tray
- MQTT credentials, TLS using the Windows certificate store, connection test, and reconnect
- Windows toast notifications with tray balloon fallback
- Start with Windows, optional hidden startup, optional update check on launch
- GitHub update check and installer download, with confirmation before installation

## Install and run

Download `HA-Win-Setup.exe` from a release built from this Rust branch, or extract
`HaWin-win-x64.zip` and run `HaWin.exe`. The portable executable can also run on its
own. Existing releases before 0.2.0 contain the C# version.

1. Open settings from the tray icon.
2. Enter your MQTT host, port, and optional credentials. Enable TLS if required by
   your broker (usually port 8883). Use a hostname matching its certificate.
3. Test the connection, then Save.
4. Home Assistant discovers the device automatically when its MQTT integration is enabled.

The Namespace defaults to the PC name. Spaces and hyphens become underscores,
letters become lowercase, and other punctuation is removed. Keep your existing
namespace and client ID when switching versions to retain the same HA entities.
Stop the C# client before starting the Rust client to avoid duplicate MQTT client IDs.

Settings are read from `%APPDATA%\HA Win\settings.json`, using the same PascalCase
JSON fields as the C# version. Legacy `%APPDATA%\HaWin\settings.json` is copied on
first launch if needed. Saves replace the file atomically; invalid JSON is reported
without overwriting it. MQTT passwords remain in this local file as in the original.
Uninstalling preserves these settings.

## MQTT

| Purpose | Topic | Payload |
| --- | --- | --- |
| Restart | `ha-win/<device-id>/restart/set` | `PRESS` |
| Shutdown | `ha-win/<device-id>/shutdown/set` | `PRESS` |
| Standby | `ha-win/<device-id>/standby/set` | `PRESS` |
| Notification | `ha-win/<device-id>/notify` | Plain text or JSON |
| Availability | `ha-win/<device-id>/status` | `online` / `offline` (retained) |

Notification JSON: `{ "title": "Door", "message": "Front door opened" }`.
Missing or non-string fields fall back to the default title/plain payload.

Discovery topics and unique IDs match the original client. Discovery is retained
and republished after reconnect and on Home Assistant's `homeassistant/status`
birth message. The client publishes `offline` on exit and sets a last will for
unexpected disconnects. Retained action and notification commands are ignored so
old commands do not replay on reconnect. Restrict publishing to these command
topics to trusted Home Assistant users through your MQTT broker's ACLs.

## Build (Windows x64)

Install [Rust stable](https://www.rust-lang.org/tools/install) with the MSVC target
and [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
with **Desktop development with C++** and a current Windows SDK.

```powershell
cargo run --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
```

The release executable is `target\release\HaWin.exe`; its application icon and
version metadata are embedded. Tests cover settings compatibility, discovery,
command parsing, update selection, reconnect, and an MQTT exchange with a local test broker.
Tests never run PC power actions or install updates.

The UI uses native Win32 controls through `native-windows-gui`, MQTT uses `rumqttc`,
and toast/power integration uses the Windows APIs. Network operations run outside
the UI thread. To debug in VS Code, install rust-analyzer and CodeLLDB.

## Packaging

With [Inno Setup 6](https://jrsoftware.org/isinfo.php) installed:

```powershell
cargo build --release --locked
New-Item -ItemType Directory -Force artifacts/publish
Copy-Item target/release/HaWin.exe, LICENSE, README.md artifacts/publish
& 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe' /DAppVersion=0.2.0 installer/ha-win.iss
```

The installer retains the existing product identity and installation directory for
upgrades. CI checks formatting, Clippy, tests, and release compilation on Windows.
The release workflow creates the ZIP and installer and drafts a GitHub release
after merging a PR to `main`, or when manually triggered. Bump `Cargo.toml` before
each release; the version tag must not already exist.

The former C# implementation is available in the `dev` branch and Git history.
