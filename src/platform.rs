use std::path::PathBuf;

/// Testing seam: if `CPC_TEST_NO_CANDIDATES=1`, all candidate lists return empty.
/// This allows integration tests to force the "no auto-detect candidates" path
/// without depending on real machine state.
fn candidates_disabled() -> bool {
    std::env::var("CPC_TEST_NO_CANDIDATES").as_deref() == Ok("1")
}

/// Returns the current platform string.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// Auto-detect candidates for the Volumes path.
pub fn volumes_candidates() -> Vec<PathBuf> {
    if candidates_disabled() {
        return vec![];
    }
    #[cfg(target_os = "windows")]
    {
        let mut candidates = vec![
            PathBuf::from(r"C:\My Drive\Volumes"),
            PathBuf::from(r"D:\My Drive\Volumes"),
        ];
        if let Ok(profile) = std::env::var("USERPROFILE") {
            candidates.push(PathBuf::from(&profile).join("Google Drive").join("Volumes"));
        }
        candidates
    }
    #[cfg(target_os = "macos")]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(
                home.join("Library")
                    .join("Mobile Documents")
                    .join("com~apple~CloudDocs")
                    .join("Volumes"),
            );
            candidates.push(home.join("Google Drive").join("Volumes"));
            candidates.push(home.join("Google Drive").join("My Drive").join("Volumes"));
        }
        candidates
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(home.join("Google Drive").join("Volumes"));
            candidates.push(home.join("gdrive").join("Volumes"));
            candidates.push(home.join(".cpc").join("volumes"));
        }
        candidates
    }
}

/// Auto-detect candidates for the install path (where server binaries live).
pub fn install_candidates() -> Vec<PathBuf> {
    if candidates_disabled() {
        return vec![];
    }
    #[cfg(target_os = "windows")]
    {
        let mut candidates = vec![PathBuf::from(r"C:\CPC\servers")];
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local_app_data).join("CPC").join("servers"));
        }
        if let Ok(program_files) = std::env::var("PROGRAMFILES") {
            candidates.push(PathBuf::from(&program_files).join("CPC").join("servers"));
        }
        candidates
    }
    #[cfg(target_os = "macos")]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(home.join(".cpc").join("servers"));
        }
        candidates.push(PathBuf::from("/usr/local/lib/cpc/servers"));
        candidates
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(home.join(".cpc").join("servers"));
        }
        candidates.push(PathBuf::from("/usr/local/lib/cpc/servers"));
        candidates.push(PathBuf::from("/opt/cpc/servers"));
        candidates
    }
}

/// Auto-detect candidates for the backups path.
pub fn backups_candidates() -> Vec<PathBuf> {
    if candidates_disabled() {
        return vec![];
    }
    #[cfg(target_os = "windows")]
    {
        let mut candidates = vec![];
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local_app_data).join("CPC").join("backups"));
        }
        candidates.push(PathBuf::from(r"C:\CPC\backups"));
        candidates
    }
    #[cfg(target_os = "macos")]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(home.join(".cpc").join("backups"));
            candidates.push(
                home.join("Library")
                    .join("Application Support")
                    .join("CPC")
                    .join("backups"),
            );
        }
        candidates
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut candidates = vec![];
        if let Some(home) = home_dir() {
            candidates.push(home.join(".cpc").join("backups"));
        }
        candidates
    }
}

#[cfg(not(target_os = "windows"))]
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
