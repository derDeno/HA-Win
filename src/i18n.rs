use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    German,
    #[default]
    #[serde(other)]
    Automatic,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::Automatic, Self::English, Self::German];

    fn from_windows_language(id: u16) -> Self {
        // The primary language also covers regional variants such as de-AT and de-CH.
        if id & 0x03ff == 0x07 {
            Self::German
        } else {
            Self::English
        }
    }

    pub fn resolved(self) -> Self {
        match self {
            Self::Automatic => Self::from_windows_language(unsafe {
                windows::Win32::Globalization::GetUserDefaultUILanguage()
            }),
            explicit => explicit,
        }
    }

    pub fn strings(self) -> &'static Strings {
        static ENGLISH: OnceLock<Strings> = OnceLock::new();
        static GERMAN: OnceLock<Strings> = OnceLock::new();
        match self.resolved() {
            Self::German => GERMAN.get_or_init(|| {
                serde_json::from_str(include_str!("../locales/de.json"))
                    .expect("Invalid embedded German translations")
            }),
            _ => ENGLISH.get_or_init(|| {
                serde_json::from_str(include_str!("../locales/en.json"))
                    .expect("Invalid embedded English translations")
            }),
        }
    }
}

static GERMAN: AtomicBool = AtomicBool::new(false);

pub fn set_language(language: Language) {
    GERMAN.store(language.resolved() == Language::German, Ordering::Relaxed);
}

pub fn text() -> &'static Strings {
    if GERMAN.load(Ordering::Relaxed) {
        Language::German.strings()
    } else {
        Language::English.strings()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strings {
    pub window_title: String,
    pub status_tab: String,
    pub connection_caption: String,
    pub not_running: String,
    pub machine_caption: String,
    pub version_caption: String,
    pub app_tab: String,
    pub host_label: String,
    pub port_label: String,
    pub namespace_label: String,
    pub username_label: String,
    pub password_label: String,
    pub client_label: String,
    pub tls: String,
    pub notify_label: String,
    pub test_button: String,
    pub app_caption: String,
    pub auto_start: String,
    pub show_window: String,
    pub auto_updates: String,
    pub update_button: String,
    pub github_button: String,
    pub save_button: String,
    pub hide_button: String,
    pub open_item: String,
    pub exit_item: String,
    pub language_label: String,
    pub automatic: String,
    pub language_hint: String,
    pub notification_registration_failed: String,
    pub invalid_port: String,
    pub invalid_host: String,
    pub invalid_client_id: String,
    pub auto_start_failed: String,
    pub testing: String,
    pub checking: String,
    pub tray_connected: String,
    pub tray_disconnected: String,
    pub test_title: String,
    pub test_success: String,
    pub connection_failed: String,
    pub update_available: String,
    pub update_prompt: String,
    pub downloading: String,
    pub update_check: String,
    pub up_to_date: String,
    pub update_check_failed: String,
    pub update_failed: String,
    pub configure_broker: String,
    pub connecting: String,
    pub running: String,
    pub disconnected: String,
    pub discovery_failed: String,
    pub broker_required: String,
    pub connection_timeout: String,
    pub already_running: String,
    pub appdata_missing: String,
    pub migration_failed: String,
    pub settings_read_failed: String,
    pub invalid_settings_path: String,
    pub settings_save_failed: String,
    pub missing_installer: String,
    pub invalid_download_url: String,
    pub preserve_installer_failed: String,
    pub invalid_installer_path: String,
    pub shutdown_failed: String,
    pub action_rejected: String,
    pub standby_failed: String,
    pub unknown_action: String,
    pub standby_privilege_failed: String,
    pub open_failed: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn windows_language_detection_and_explicit_overrides() {
        for id in [0x0407, 0x0807, 0x0c07] {
            assert_eq!(Language::from_windows_language(id), Language::German);
        }
        for id in [0, 0x0409, 0x0809, 0x040c] {
            assert_eq!(Language::from_windows_language(id), Language::English);
        }
        assert_eq!(Language::English.resolved(), Language::English);
        assert_eq!(Language::German.resolved(), Language::German);
    }

    #[test]
    fn embedded_catalogs_are_complete_and_preserve_placeholders() {
        let english: BTreeMap<String, String> =
            serde_json::from_str(include_str!("../locales/en.json")).unwrap();
        let german: BTreeMap<String, String> =
            serde_json::from_str(include_str!("../locales/de.json")).unwrap();
        assert_eq!(
            english.keys().collect::<Vec<_>>(),
            german.keys().collect::<Vec<_>>()
        );
        let placeholders = |value: &str| -> BTreeSet<String> {
            value
                .split('{')
                .skip(1)
                .map(|s| s.split('}').next().unwrap().to_owned())
                .collect()
        };
        for (key, value) in &english {
            assert!(!value.trim().is_empty(), "{key}");
            assert!(!german[key].trim().is_empty(), "{key}");
            assert_eq!(placeholders(value), placeholders(&german[key]), "{key}");
        }
        assert_eq!(Language::English.strings().save_button, "Save");
        assert_eq!(Language::German.strings().save_button, "Speichern");
    }
}
