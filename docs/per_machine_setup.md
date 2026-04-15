# Per-Machine Setup

cpc-paths is a library, not a server. It's used by CPC MCP servers (hands, manager, local, workflow). Per-machine setup happens in those server repos — see their respective `docs/per_machine_setup.md`.

This crate's only per-machine concern is `.cpc-config.toml` at:
- Windows: `%APPDATA%\CPC\.cpc-config.toml`
- macOS: `~/Library/Application Support/CPC/.cpc-config.toml`
- Linux: `~/.config/cpc/.cpc-config.toml`

If env vars are set and config file exists, cpc-paths uses them. If neither, auto-detect runs.
