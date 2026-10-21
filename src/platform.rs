use anyhow::{bail, Context, Result};
use std::{os::windows::process::CommandExt, process::Command};
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    Win32::System::Power::SetSuspendState,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE},
    RegKey,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_ID: &str = "HA Win";

pub fn auto_start_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ)
        .ok()
        .is_some_and(|key| {
            [APP_ID, "HaWin"]
                .iter()
                .any(|name| key.get_value::<String, _>(name).is_ok())
        })
}

pub fn set_auto_start(enabled: bool) -> Result<()> {
    let (key, _) =
        RegKey::predef(HKEY_CURRENT_USER).create_subkey_with_flags(RUN_KEY, KEY_WRITE)?;
    if enabled {
        key.set_value(
            APP_ID,
            &format!("\"{}\"", std::env::current_exe()?.display()),
        )?;
    } else {
        delete_value(&key, APP_ID)?;
    }
    delete_value(&key, "HaWin")?;
    Ok(())
}

fn delete_value(key: &RegKey, name: &str) -> Result<()> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn register_notifications() -> Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(format!(r"Software\Classes\AppUserModelId\{APP_ID}"))?;
    key.set_value("DisplayName", &APP_ID)?;
    key.set_value(
        "IconUri",
        &std::env::current_exe()?.to_string_lossy().as_ref(),
    )?;
    Ok(())
}

fn xml_escape(value: &str) -> String {
    value
        .chars()
        .filter(|&c| matches!(c, '\t' | '\n' | '\r') || c >= ' ')
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn show_notification(title: &str, message: &str) -> Result<()> {
    let xml = XmlDocument::new()?;
    xml.LoadXml(&HSTRING::from(format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>",
        xml_escape(title), xml_escape(message)
    )))?;
    let toast = ToastNotification::CreateToastNotification(&xml)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID))?.Show(&toast)?;
    Ok(())
}

pub fn device_action(action: &str) -> Result<()> {
    match action {
        "restart" | "shutdown" => {
            let status = Command::new("shutdown.exe")
                .args([if action == "restart" { "/r" } else { "/s" }, "/t", "0"])
                .creation_flags(0x08000000)
                .status()
                .context("Could not run shutdown.exe")?;
            if !status.success() {
                bail!("Windows rejected the {action} command ({status}).");
            }
        }
        "standby" => {
            // SetSuspendState requires the shutdown privilege, enabled only for this call.
            enable_shutdown_privilege()?;
            let result = unsafe { SetSuspendState(false, true, true) };
            let error = std::io::Error::last_os_error();
            disable_shutdown_privilege();
            if !result {
                bail!("Windows could not enter standby: {error}");
            }
        }
        _ => bail!("Unknown device action"),
    }
    Ok(())
}

fn enable_shutdown_privilege() -> Result<()> {
    set_shutdown_privilege(true)
}
fn disable_shutdown_privilege() {
    let _ = set_shutdown_privilege(false);
}

fn set_shutdown_privilege(enabled: bool) -> Result<()> {
    use windows::Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_NOT_ALL_ASSIGNED, HANDLE},
        Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
            TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )?;
        let result = (|| -> Result<()> {
            let mut privileges = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                ..Default::default()
            };
            LookupPrivilegeValueW(
                None,
                windows::core::w!("SeShutdownPrivilege"),
                &mut privileges.Privileges[0].Luid,
            )?;
            if enabled {
                privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
            }
            AdjustTokenPrivileges(token, false, Some(&privileges), 0, None, None)?;
            if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
                bail!("Windows did not grant the standby privilege.");
            }
            Ok(())
        })();
        let _ = CloseHandle(token);
        result
    }
}

pub fn open_url(url: &str) -> Result<()> {
    use windows::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    let result = unsafe {
        ShellExecuteW(
            None,
            windows::core::w!("open"),
            &HSTRING::from(url),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as usize <= 32 {
        bail!("Windows could not open {url}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_untrusted_notification_xml() {
        assert_eq!(
            xml_escape("<script> & \"x\"\0"),
            "&lt;script&gt; &amp; &quot;x&quot;"
        );
    }
}
