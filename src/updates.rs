use crate::i18n;
use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use std::{io::Write, time::Duration};

const RELEASES: &str = "https://api.github.com/repos/derDeno/HA-Win/releases/latest";

#[derive(Clone, Debug)]
pub struct Update {
    pub version: Version,
    pub url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("HA-Win-Updater")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .https_only(true)
        .build()?)
}

fn release_update(release: Release, current: &str) -> Result<Option<Update>> {
    let latest = Version::parse(release.tag_name.trim().trim_start_matches(['v', 'V']))?;
    if latest <= Version::parse(current)? {
        return Ok(None);
    }
    let Some(asset) = release
        .assets
        .into_iter()
        .find(|a| a.name.eq_ignore_ascii_case("HA-Win-Setup.exe"))
    else {
        bail!("{}", i18n::text().missing_installer);
    };
    validate_url(&asset.browser_download_url)?;
    Ok(Some(Update {
        version: latest,
        url: asset.browser_download_url,
    }))
}

fn validate_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("github.com")
        || !parsed
            .path()
            .starts_with("/derDeno/HA-Win/releases/download/")
    {
        bail!("{}", i18n::text().invalid_download_url);
    }
    Ok(())
}

pub fn check() -> Result<Option<Update>> {
    let response = client()?.get(RELEASES).send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    release_update(
        response.error_for_status()?.json()?,
        env!("CARGO_PKG_VERSION"),
    )
}

pub fn install(update: &Update) -> Result<()> {
    validate_url(&update.url)?;
    let mut response = client()?.get(&update.url).send()?.error_for_status()?;
    let mut file = tempfile::Builder::new()
        .prefix("HA-Win-Setup-")
        .suffix(".exe")
        .tempfile()?;
    response.copy_to(&mut file)?;
    file.flush()?;
    file.as_file().sync_all()?;
    // Keep the downloaded installer alive after this process exits.
    let (_, path) = file
        .keep()
        .context(&i18n::text().preserve_installer_failed)?;
    crate::platform::open_url(
        path.to_str()
            .context(&i18n::text().invalid_installer_path)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_newer_releases_with_the_real_installer_are_used() {
        let release = |tag: &str| Release {
            tag_name: tag.into(),
            assets: vec![Asset {
                name: "HA-Win-Setup.exe".into(),
                browser_download_url:
                    "https://github.com/derDeno/HA-Win/releases/download/v1.1.0/HA-Win-Setup.exe"
                        .into(),
            }],
        };
        assert!(release_update(release("v0.1.9"), "1.0.0")
            .unwrap()
            .is_none());
        assert!(release_update(release("v1.0.0"), "1.0.0")
            .unwrap()
            .is_none());
        assert!(release_update(release("v1.1.0"), "1.0.0")
            .unwrap()
            .is_some());
        assert!(validate_url("https://evil.example/setup.exe").is_err());
        assert!(
            validate_url("http://github.com/derDeno/HA-Win/releases/download/v1/setup.exe")
                .is_err()
        );
    }
}
