use crate::{
    config::{read_config, write_config_key},
    error::Error,
    platform, ConfigKey, ResolutionMethod,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    sync::Mutex,
};

/// In-process cache entry: resolved path + how it was resolved.
#[derive(Clone)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub method: ResolutionMethod,
}

/// Global in-process cache for the three main paths.
pub struct PathCache {
    pub volumes: Option<CacheEntry>,
    pub install: Option<CacheEntry>,
    pub backups: Option<CacheEntry>,
}

static CACHE: Lazy<Mutex<PathCache>> = Lazy::new(|| {
    Mutex::new(PathCache {
        volumes: None,
        install: None,
        backups: None,
    })
});

pub fn invalidate() {
    let mut cache = CACHE.lock().expect("cache lock poisoned");
    cache.volumes = None;
    cache.install = None;
    cache.backups = None;
}

/// Descriptor for a path type — drives the generic resolution algorithm.
pub struct PathDescriptor {
    pub name: &'static str,
    pub env_var: &'static str,
    pub config_key: ConfigKey,
    pub candidates_fn: fn() -> Vec<PathBuf>,
}

pub static VOLUMES_DESC: PathDescriptor = PathDescriptor {
    name: "Volumes",
    env_var: "CPC_VOLUMES_PATH",
    config_key: ConfigKey::VolumesPath,
    candidates_fn: platform::volumes_candidates,
};

pub static INSTALL_DESC: PathDescriptor = PathDescriptor {
    name: "Install",
    env_var: "CPC_INSTALL_PATH",
    config_key: ConfigKey::InstallPath,
    candidates_fn: platform::install_candidates,
};

pub static BACKUPS_DESC: PathDescriptor = PathDescriptor {
    name: "Backups",
    env_var: "CPC_BACKUPS_PATH",
    config_key: ConfigKey::BackupsPath,
    candidates_fn: platform::backups_candidates,
};

pub fn resolve(desc: &PathDescriptor) -> Result<(PathBuf, ResolutionMethod)> {
    // 1. Cache.
    {
        let cache = CACHE.lock().expect("cache lock poisoned");
        let hit = match desc.config_key {
            ConfigKey::VolumesPath => cache.volumes.clone(),
            ConfigKey::InstallPath => cache.install.clone(),
            ConfigKey::BackupsPath => cache.backups.clone(),
        };
        if let Some(entry) = hit {
            return Ok((entry.path, ResolutionMethod::Cache));
        }
    }

    // 2. Env var.
    if let Ok(val) = std::env::var(desc.env_var) {
        let p = PathBuf::from(&val);
        if p.exists() {
            let method = ResolutionMethod::EnvVar(desc.env_var.to_string());
            store_cache(desc.config_key, p.clone(), method.clone());
            return Ok((p, method));
        }
    }

    // 3. Config file.
    let config_file = crate::config_file();
    let cfg = read_config(&config_file).unwrap_or_default();
    let from_config = match desc.config_key {
        ConfigKey::VolumesPath => crate::config::config_volumes(&cfg),
        ConfigKey::InstallPath => crate::config::config_install(&cfg),
        ConfigKey::BackupsPath => crate::config::config_backups(&cfg),
    };
    if let Some(p) = from_config {
        store_cache(desc.config_key, p.clone(), ResolutionMethod::ConfigFile);
        return Ok((p, ResolutionMethod::ConfigFile));
    }

    // 4. Auto-detect.
    let candidates: Vec<PathBuf> = (desc.candidates_fn)()
        .into_iter()
        .filter(|p| p.exists())
        .collect();

    match candidates.len() {
        0 => {
            // 5a. Interactive prompt.
            if io::stdin().is_terminal() {
                eprint!("{} path not found. Enter path: ", desc.name);
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let p = PathBuf::from(input.trim());
                // Persist to config.
                let _ = write_config_key(&config_file, desc.config_key, &p);
                store_cache(desc.config_key, p.clone(), ResolutionMethod::AutoDetect);
                return Ok((p, ResolutionMethod::AutoDetect));
            }
            // 5b. Non-interactive: error.
            Err(Error::NotFound {
                path_type: desc.name.to_string(),
                env_var: desc.env_var,
                config_file,
            }
            .into())
        }
        1 => {
            let p = candidates.into_iter().next().unwrap();
            // Persist to config.
            let _ = write_config_key(&config_file, desc.config_key, &p);
            store_cache(desc.config_key, p.clone(), ResolutionMethod::AutoDetect);
            Ok((p, ResolutionMethod::AutoDetect))
        }
        _ => {
            // Multiple candidates.
            if io::stdin().is_terminal() {
                eprintln!("Multiple {} candidates found:", desc.name);
                for (i, c) in candidates.iter().enumerate() {
                    eprintln!("  [{}] {}", i + 1, c.display());
                }
                eprint!("Enter number: ");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let idx: usize = input.trim().parse().unwrap_or(1);
                let p = candidates
                    .get(idx.saturating_sub(1))
                    .cloned()
                    .unwrap_or_else(|| candidates[0].clone());
                let _ = write_config_key(&config_file, desc.config_key, &p);
                store_cache(desc.config_key, p.clone(), ResolutionMethod::AutoDetect);
                Ok((p, ResolutionMethod::AutoDetect))
            } else {
                Err(Error::Ambiguous {
                    path_type: desc.name.to_string(),
                    candidates,
                }
                .into())
            }
        }
    }
}

fn store_cache(key: ConfigKey, path: PathBuf, method: ResolutionMethod) {
    let mut cache = CACHE.lock().expect("cache lock poisoned");
    let entry = CacheEntry { path, method };
    match key {
        ConfigKey::VolumesPath => cache.volumes = Some(entry),
        ConfigKey::InstallPath => cache.install = Some(entry),
        ConfigKey::BackupsPath => cache.backups = Some(entry),
    }
}

/// Attempt resolution without errors — returns None on failure. Used by health_check.
pub fn try_resolve(desc: &PathDescriptor) -> (Option<PathBuf>, ResolutionMethod) {
    match resolve(desc) {
        Ok((p, m)) => (Some(p), m),
        Err(_) => (None, ResolutionMethod::None),
    }
}
