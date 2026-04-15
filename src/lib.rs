//! cpc-paths — portable path discovery for CPC MCP servers.
//!
//! Resolves Volumes, install, and backups paths with the following precedence:
//! env var → `.cpc-config.toml` → auto-detect → interactive prompt → error.

pub mod config;
pub mod error;
pub mod health;
pub mod platform;
pub mod resolver;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub use error::Error;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionMethod {
    Cache,
    /// Env var name that resolved.
    EnvVar(String),
    ConfigFile,
    AutoDetect,
    PlatformDefault,
    /// Failed to resolve.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStatus {
    pub path: Option<PathBuf>,
    pub resolved_via: ResolutionMethod,
    pub exists: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub volumes: PathStatus,
    pub install: PathStatus,
    pub backups: PathStatus,
    pub config_file: PathStatus,
    /// "windows" | "macos" | "linux"
    pub platform: String,
    /// env!("CARGO_PKG_VERSION")
    pub crate_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    VolumesPath,
    InstallPath,
    BackupsPath,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Resolve the Volumes path (CPC knowledge base root).
///
/// Precedence: `CPC_VOLUMES_PATH` env → `.cpc-config.toml` → auto-detect → prompt → error.
pub fn volumes_path() -> Result<PathBuf> {
    let (p, _) = resolver::resolve(&resolver::VOLUMES_DESC)?;
    Ok(p)
}

/// Resolve the install path (where server binaries live).
pub fn install_path() -> Result<PathBuf> {
    let (p, _) = resolver::resolve(&resolver::INSTALL_DESC)?;
    Ok(p)
}

/// Resolve the backups path.
pub fn backups_path() -> Result<PathBuf> {
    let (p, _) = resolver::resolve(&resolver::BACKUPS_DESC)?;
    Ok(p)
}

/// Resolve a per-server data directory, creating it if it doesn't exist.
///
/// Resolves to:
/// - Windows: `%LOCALAPPDATA%\CPC\{server}-data\`
/// - macOS: `~/Library/Application Support/CPC/{server}-data/`
/// - Linux: `~/.local/share/CPC/{server}-data/`
pub fn data_path(server: &str) -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "CPC")
        .ok_or_else(|| anyhow::anyhow!("Could not determine project data directory"))?;
    // Use data_local_dir() → %LOCALAPPDATA% on Windows, ~/.local/share on Linux,
    // ~/Library/Application Support on macOS.
    let data_dir = dirs.data_local_dir().join(format!("{}-data", server));
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

/// Directory where `.cpc-config.toml` lives.
pub fn config_dir() -> PathBuf {
    // Allow tests to override via CPC_CONFIG_DIR.
    if let Ok(v) = std::env::var("CPC_CONFIG_DIR") {
        return PathBuf::from(v);
    }
    platform_config_dir()
}

/// Path to `.cpc-config.toml` within `config_dir()`.
pub fn config_file() -> PathBuf {
    config_dir().join(".cpc-config.toml")
}

/// Write a config entry. Creates `config_dir()` and `config_file()` as needed.
pub fn set_config(key: ConfigKey, value: &Path) -> Result<()> {
    config::write_config_key(&config_file(), key, value)?;
    Ok(())
}

/// Force re-detection on next call (clears in-process cache).
pub fn invalidate_cache() {
    resolver::invalidate();
}

/// Returns a full diagnostic report of all resolved paths and how each was resolved.
pub fn health_check() -> HealthReport {
    health::build_health_report()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn platform_config_dir() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("", "", "CPC") {
        // Use the config dir as per directories crate conventions.
        dirs.config_dir().to_path_buf()
    } else {
        // Fallback: next to the binary.
        PathBuf::from(".")
    }
}
