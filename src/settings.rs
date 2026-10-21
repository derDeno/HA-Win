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
        if self.broker_port == 0 {
            bail!("Broker port must be between 1 and 65535.");
        }
        if self.broker_host.contains(['/', '\\', '\0']) {
            bail!("Enter a broker hostname or IP address without a URL scheme.");
        }
        if self.client_id.contains('\0') {
            bail!("Client ID contains an invalid character.");
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
    let root = PathBuf::from(std::env::var_os("APPDATA").context("APPDATA is not set")?);
    let path = root.join("HA Win/settings.json");
    let legacy = root.join("HaWin/settings.json");
    if !path.exists() && legacy.exists() {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::copy(legacy, &path).context("Could not migrate existing settings")?;
    }
    Ok(path)
}

pub fn load(path: &Path) -> Result<Settings> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(json.trim_start_matches('\u{feff}'))
            .context("Could not read settings.json"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<()> {
    settings.validate()?;
    let folder = path.parent().context("Invalid settings path")?;
    fs::create_dir_all(folder)?;
    let mut file = tempfile::NamedTempFile::new_in(folder)?;
    file.write_all(serde_json::to_string_pretty(settings)?.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path)
        .context("Could not replace settings.json")?;
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
}
