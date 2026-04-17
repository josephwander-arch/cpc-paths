use crate::ConfigKey;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Raw TOML structure mirroring .cpc-config.toml.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CpcConfig {
    #[serde(default)]
    pub paths: PathsSection,
    #[serde(default)]
    pub metadata: MetadataSection,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PathsSection {
    pub volumes: Option<String>,
    pub install: Option<String>,
    pub backups: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetadataSection {
    pub schema_version: Option<u32>,
    pub last_updated: Option<String>,
    pub last_updated_by: Option<String>,
}

/// Read .cpc-config.toml from `config_file`. Returns default (empty) if the file does not exist.
pub fn read_config(config_file: &Path) -> Result<CpcConfig, crate::error::Error> {
    if !config_file.exists() {
        return Ok(CpcConfig::default());
    }
    let content = fs::read_to_string(config_file)?;
    let cfg: CpcConfig = toml::from_str(&content)?;
    Ok(cfg)
}

/// Write a single key/value into .cpc-config.toml, creating the file and parent dirs if needed.
pub fn write_config_key(
    config_file: &Path,
    key: ConfigKey,
    value: &Path,
) -> Result<(), crate::error::Error> {
    // Read existing or start fresh.
    let mut cfg = read_config(config_file)?;

    let val_str = value.to_string_lossy().into_owned();
    match key {
        ConfigKey::VolumesPath => cfg.paths.volumes = Some(val_str),
        ConfigKey::InstallPath => cfg.paths.install = Some(val_str),
        ConfigKey::BackupsPath => cfg.paths.backups = Some(val_str),
    }

    cfg.metadata.schema_version = Some(1);
    cfg.metadata.last_updated = Some(Utc::now().to_rfc3339());
    cfg.metadata.last_updated_by = Some(format!("cpc-paths v{}", env!("CARGO_PKG_VERSION")));

    // Ensure parent dir exists.
    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let comment = "# CPC configuration file — written by cpc-paths auto-detect.\n\
                   # Manual edits survive subsequent auto-detects.\n\n";
    let body = toml::to_string_pretty(&cfg)?;
    fs::write(config_file, format!("{comment}{body}"))?;

    Ok(())
}

/// Read the Volumes path from config, if present and the path exists on disk.
pub fn config_volumes(cfg: &CpcConfig) -> Option<PathBuf> {
    cfg.paths.volumes.as_ref().and_then(|s| {
        let p = PathBuf::from(s);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    })
}

/// Read the install path from config, if present and the path exists on disk.
pub fn config_install(cfg: &CpcConfig) -> Option<PathBuf> {
    cfg.paths.install.as_ref().and_then(|s| {
        let p = PathBuf::from(s);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    })
}

/// Read the backups path from config, if present and the path exists on disk.
pub fn config_backups(cfg: &CpcConfig) -> Option<PathBuf> {
    cfg.paths.backups.as_ref().and_then(|s| {
        let p = PathBuf::from(s);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    })
}
