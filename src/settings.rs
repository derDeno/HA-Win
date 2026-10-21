use crate::i18n::{self, Language};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Settings {
    pub broker_host: String,
    pub broker_port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub auto_start: bool,
    pub namespace: String,
    pub auto_check_updates: bool,
    pub client_id: String,
    pub show_window_on_startup: bool,
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            broker_host: String::new(),
            broker_port: 1883,
            username: String::new(),
            password: String::new(),
            use_tls: false,
            auto_start: false,
            namespace: String::new(),
            auto_check_updates: false,
            client_id: String::new(),
            show_window_on_startup: true,
            language: Language::Automatic,
        }
    }
}

impl Settings {
    pub fn device_id(&self) -> String {
        if self.namespace.trim().is_empty() {
            sanitize(&machine_name())
        } else {
            sanitize(&self.namespace)
        }
    }

    pub fn validate(&self) -> Result<()> {
        let t = self.language.strings();
        if self.broker_port == 0 {
            bail!("{}", t.invalid_port);
        }
        if self.broker_host.contains(['/', '\\', '\0']) {
            bail!("{}", t.invalid_host);
        }
        if self.client_id.contains('\0') {
            bail!("{}", t.invalid_client_id);
        }
        Ok(())
    }
}

pub fn machine_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "pc".into())
}

pub fn sanitize(value: &str) -> String {
    let name: String = value
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if matches!(c, '-' | '_' | ' ') {
                Some('_')
            } else {
                None
            }
        })
        .collect();
    if name.is_empty() {
        "pc".into()
    } else {
        name
    }
}

pub fn settings_path() -> Result<PathBuf> {
    let root = PathBuf::from(std::env::var_os("APPDATA").context(&i18n::text().appdata_missing)?);
    let path = root.join("HA Win/settings.json");
    let legacy = root.join("HaWin/settings.json");
    if !path.exists() && legacy.exists() {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::copy(legacy, &path).context(&i18n::text().migration_failed)?;
    }
    Ok(path)
}

pub fn load(path: &Path) -> Result<Settings> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(json.trim_start_matches('\u{feff}'))
            .context(&i18n::text().settings_read_failed),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<()> {
    settings.validate()?;
    let folder = path.parent().context(&i18n::text().invalid_settings_path)?;
    fs::create_dir_all(folder)?;
    let mut file = tempfile::NamedTempFile::new_in(folder)?;
    file.write_all(serde_json::to_string_pretty(settings)?.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path)
        .context(&i18n::text().settings_save_failed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_existing_topic_names() {
        assert_eq!(sanitize("  My-PC 01! "), "my_pc_01");
        assert_eq!(sanitize("Büro_PC"), "büro_pc");
        assert_eq!(sanitize("/+#"), "pc");
    }

    #[test]
    fn reads_dotnet_settings_and_roundtrips_atomically() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            r#"{"BrokerHost":"büro.local","Password":"päss","Namespace":"Office-PC","UseTls":true}"#,
        )?;
        let mut settings = load(&path)?;
        assert_eq!(settings.broker_port, 1883);
        assert!(settings.show_window_on_startup);
        assert_eq!(settings.language, Language::Automatic);
        assert_eq!(settings.device_id(), "office_pc");
        save(&path, &settings)?;
        settings.broker_port = 8883;
        save(&path, &settings)?;
        assert_eq!(load(&path)?, settings);
        assert!(fs::read_to_string(&path)?.contains("\"BrokerHost\""));
        Ok(())
    }

    #[test]
    fn rejects_invalid_settings_without_overwriting() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(&path, "broken json")?;
        assert!(load(&path).is_err());
        let invalid = Settings {
            broker_port: 0,
            ..Default::default()
        };
        assert!(save(&path, &invalid).is_err());
        assert_eq!(fs::read_to_string(path)?, "broken json");
        Ok(())
    }

    #[test]
    fn language_preference_roundtrips_and_unknown_languages_use_automatic() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        for language in Language::ALL {
            let settings = Settings {
                language,
                ..Default::default()
            };
            save(&path, &settings)?;
            assert_eq!(load(&path)?.language, language);
        }
        fs::write(
            &path,
            r#"{"Language":"french","BrokerHost":"broker.local"}"#,
        )?;
        let settings = load(&path)?;
        assert_eq!(settings.language, Language::Automatic);
        assert_eq!(settings.broker_host, "broker.local");
        let invalid = Settings {
            language: Language::German,
            broker_port: 0,
            ..Default::default()
        };
        assert_eq!(
            invalid.validate().unwrap_err().to_string(),
            Language::German.strings().invalid_port
        );
        Ok(())
    }
}
