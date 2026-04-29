# Changelog

## [Unreleased]

## v0.1.2 - 2026-04-29

### Changed
- ci: bump GitHub Actions versions to latest (Node.js 20 deprecation)

## v0.1.1 — 2026-04-18

### Fixed

- **Backups path resolution fails with `Ambiguous` error** when both `%LOCALAPPDATA%\CPC\backups` and `C:\CPC\backups` exist. Added derive-from-install fallback: if install is resolved and `{install}/../backups/` exists, use it deterministically before auto-detect. Resolves as `PlatformDefault`.

### Added

- **GitHub Actions CI workflow** — push/PR to main runs mojibake scan, cargo check (x64 + ARM64), fmt, clippy, version alignment.
- **GitHub Actions release workflow** — `v*` tag push validates library builds on both targets.
- **SECURITY.md** — security policy and reporting instructions.
- Integration test `backups_derived_from_install` covering the new resolution path.

## v0.1.0 — 2026-04-15

Initial release. Windows-tested, macOS/Linux candidates in place but unverified.

### Added
- `volumes_path()`, `install_path()`, `backups_path()` resolution with cache → env var → config file → auto-detect → prompt → error precedence
- `data_path(server)` per-server data directory at platform-standard location (`%LOCALAPPDATA%\CPC\{server}-data\` on Windows)
- `config_dir()`, `config_file()`, `set_config()` for managing `.cpc-config.toml`
- `invalidate_cache()` for forcing re-resolution
- `health_check()` returns full diagnostic report (path + resolution method + exists)
- Apache 2.0 license

### Design choices
- No negative cache — failed resolution always retries on next call
- No log/tracing facade — caller wires own logging if needed
- No async runtime — synchronous resolution, fast enough for MCP server startup
