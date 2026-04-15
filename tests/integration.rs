/// Integration tests for cpc-paths.
///
/// # Isolation strategy
///
/// Env vars are process-global. To prevent races between parallel test threads
/// we hold a global `ENV_LOCK` mutex for the duration of every test that touches
/// env vars. Tests are otherwise independent — each gets its own `TempDir` for
/// `CPC_CONFIG_DIR`.
///
/// # Testing seams
/// - `CPC_CONFIG_DIR` — redirects config file location to a temp dir
/// - `CPC_TEST_NO_CANDIDATES=1` — makes platform::*_candidates() return empty,
///   forcing the "no auto-detect candidates" code path without depending on real
///   machine state
use cpc_paths::{self, ConfigKey, ResolutionMethod};
use std::{
    env,
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};
use tempfile::TempDir;

/// Global serialization lock for tests that mutate env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the global lock, reset all CPC env vars, point config dir at `tmp`,
/// and invalidate the path cache. Returns (TempDir, lock guard) — both must
/// stay alive for the test duration.
fn isolated() -> (TempDir, MutexGuard<'static, ()>) {
    let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    env::set_var("CPC_CONFIG_DIR", tmp.path());
    env::remove_var("CPC_VOLUMES_PATH");
    env::remove_var("CPC_INSTALL_PATH");
    env::remove_var("CPC_BACKUPS_PATH");
    env::remove_var("CPC_TEST_NO_CANDIDATES");
    cpc_paths::invalidate_cache();
    (tmp, guard)
}

// ---------------------------------------------------------------------------
// 1. env_var_takes_precedence
// ---------------------------------------------------------------------------
#[test]
fn env_var_takes_precedence() {
    let (_tmp, _lock) = isolated();
    let vol_dir = tempfile::tempdir().expect("vol tempdir");
    env::set_var("CPC_VOLUMES_PATH", vol_dir.path());
    cpc_paths::invalidate_cache();

    let result = cpc_paths::volumes_path().expect("should resolve via env");
    assert_eq!(result, vol_dir.path(), "should return the env-var path");
}

// ---------------------------------------------------------------------------
// 2. config_file_used_when_no_env
// ---------------------------------------------------------------------------
#[test]
fn config_file_used_when_no_env() {
    let (tmp, _lock) = isolated();

    // Create a real directory so existence check passes.
    let vol_dir = tempfile::tempdir().expect("vol tempdir");

    // Write a config file manually.
    let config_path = tmp.path().join(".cpc-config.toml");
    let toml_content = format!(
        "[paths]\nvolumes = {:?}\n",
        vol_dir.path().to_string_lossy()
    );
    fs::write(&config_path, toml_content).expect("write config");

    // Disable auto-detect so we don't pick up real machine paths.
    env::set_var("CPC_TEST_NO_CANDIDATES", "1");
    cpc_paths::invalidate_cache();

    let result = cpc_paths::volumes_path().expect("should resolve via config");
    assert_eq!(result, vol_dir.path());
}

// ---------------------------------------------------------------------------
// 3. autodetect_when_no_config
// ---------------------------------------------------------------------------
#[test]
fn autodetect_when_no_config() {
    let (_tmp, _lock) = isolated();
    // No env var, no config file, auto-detect enabled.
    // On this machine real candidates may or may not exist — either outcome is valid.
    // We just verify: succeeds with a path OR fails with an actionable error.
    match cpc_paths::volumes_path() {
        Ok(p) => {
            assert!(!p.as_os_str().is_empty(), "auto-detect returned a non-empty path");
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("CPC_VOLUMES_PATH")
                    || msg.contains(".cpc-config.toml")
                    || msg.contains("not found"),
                "error should be actionable: {msg}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. cache_returns_same_value
// ---------------------------------------------------------------------------
#[test]
fn cache_returns_same_value() {
    let (_tmp, _lock) = isolated();
    let vol_dir = tempfile::tempdir().expect("vol tempdir");
    env::set_var("CPC_VOLUMES_PATH", vol_dir.path());
    cpc_paths::invalidate_cache();

    let first = cpc_paths::volumes_path().expect("first call");
    let second = cpc_paths::volumes_path().expect("second call");
    assert_eq!(first, second, "cache should return the same path");
}

// ---------------------------------------------------------------------------
// 5. invalidate_cache_forces_reresolve
// ---------------------------------------------------------------------------
#[test]
fn invalidate_cache_forces_reresolve() {
    let (_tmp, _lock) = isolated();

    let vol_dir1 = tempfile::tempdir().expect("vol tempdir 1");
    env::set_var("CPC_VOLUMES_PATH", vol_dir1.path());
    cpc_paths::invalidate_cache();
    let first = cpc_paths::volumes_path().expect("first resolve");
    assert_eq!(first, vol_dir1.path());

    let vol_dir2 = tempfile::tempdir().expect("vol tempdir 2");
    env::set_var("CPC_VOLUMES_PATH", vol_dir2.path());
    cpc_paths::invalidate_cache();
    let second = cpc_paths::volumes_path().expect("second resolve");
    assert_eq!(second, vol_dir2.path(), "should pick up new env var after invalidate");
}

// ---------------------------------------------------------------------------
// 6. non_interactive_returns_error_on_no_candidates (volumes)
// ---------------------------------------------------------------------------
#[test]
fn non_interactive_returns_error_on_no_candidates() {
    let (_tmp, _lock) = isolated();
    env::set_var("CPC_TEST_NO_CANDIDATES", "1");
    cpc_paths::invalidate_cache();

    // No env var, no config file, no candidates → Error::NotFound.
    let err = cpc_paths::volumes_path();
    assert!(err.is_err(), "zero candidates (non-interactive) must return error");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("CPC_VOLUMES_PATH") || msg.contains("not found"),
        "error should name the env var: {msg}"
    );
}

// ---------------------------------------------------------------------------
// 7. non_interactive_returns_error_on_no_candidates (install)
// ---------------------------------------------------------------------------
#[test]
fn non_interactive_install_no_candidates() {
    let (_tmp, _lock) = isolated();
    env::set_var("CPC_TEST_NO_CANDIDATES", "1");
    cpc_paths::invalidate_cache();

    let err = cpc_paths::install_path();
    assert!(err.is_err(), "zero candidates → error for install");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("CPC_INSTALL_PATH") || msg.contains("not found"),
        "error should name the env var: {msg}"
    );
}

// ---------------------------------------------------------------------------
// 8. data_path_creates_directory
// ---------------------------------------------------------------------------
#[test]
fn data_path_creates_directory() {
    let result = cpc_paths::data_path("test_server_cpcpaths_integration");
    let path = result.expect("data_path should succeed");
    assert!(path.exists(), "data directory should be created");
    assert!(path.is_dir(), "should be a directory");
    // Cleanup.
    let _ = fs::remove_dir_all(&path);
}

// ---------------------------------------------------------------------------
// 9. set_config_writes_toml
// ---------------------------------------------------------------------------
#[test]
fn set_config_writes_toml() {
    let (tmp, _lock) = isolated();
    let fake_vol = PathBuf::from("/tmp/cpc_test_volumes");

    cpc_paths::set_config(ConfigKey::VolumesPath, &fake_vol).expect("set_config should succeed");

    let config_path = tmp.path().join(".cpc-config.toml");
    assert!(config_path.exists(), "config file should be created");

    let content = fs::read_to_string(&config_path).expect("read config");
    assert!(
        content.contains("volumes"),
        "config should contain 'volumes' key: {content}"
    );
    assert!(
        content.contains("schema_version"),
        "config should contain metadata: {content}"
    );
}

// ---------------------------------------------------------------------------
// 10. health_check_reports_resolution_method
// ---------------------------------------------------------------------------
#[test]
fn health_check_reports_resolution_method() {
    let (_tmp, _lock) = isolated();
    let vol_dir = tempfile::tempdir().expect("vol tempdir");
    env::set_var("CPC_VOLUMES_PATH", vol_dir.path());
    cpc_paths::invalidate_cache();

    let report = cpc_paths::health_check();

    assert!(
        matches!(
            &report.volumes.resolved_via,
            ResolutionMethod::EnvVar(_) | ResolutionMethod::Cache
        ),
        "expected EnvVar or Cache, got {:?}",
        report.volumes.resolved_via
    );
    assert!(report.volumes.path.is_some(), "volumes path should be Some");
    assert!(!report.crate_version.is_empty(), "crate_version should be set");
    assert!(
        matches!(report.platform.as_str(), "windows" | "macos" | "linux"),
        "platform should be a known string: {}",
        report.platform
    );
}
