# Changelog

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
