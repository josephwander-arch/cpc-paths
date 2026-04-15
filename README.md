# cpc-paths

Portable path discovery for CPC MCP servers.

## What it does

Resolves three core CPC paths (Volumes, install, backups) plus per-server data directories with consistent precedence: env var → config file → auto-detect → prompt (interactive only) → error.

## Usage

```rust
use cpc_paths;

fn main() -> anyhow::Result<()> {
    let volumes = cpc_paths::volumes_path()?;
    let install = cpc_paths::install_path()?;
    let workflow_data = cpc_paths::data_path("workflow")?;

    println!("Volumes: {}", volumes.display());
    println!("Install: {}", install.display());
    println!("Workflow data: {}", workflow_data.display());

    // Diagnostic report
    let report = cpc_paths::health_check();
    println!("{:#?}", report);

    Ok(())
}
```

## Resolution precedence

For volumes/install/backups:
1. Env var (`CPC_VOLUMES_PATH`, `CPC_INSTALL_PATH`, `CPC_BACKUPS_PATH`)
2. `.cpc-config.toml` in platform config dir
3. Platform auto-detect (checks known candidates)
4. Interactive prompt (only if stdin is a tty — never for MCP servers running stdio)
5. `Error::NotFound` or `Error::Ambiguous`

For data paths: defaults to `%LOCALAPPDATA%\CPC\{server}-data\` (Windows) / equivalent on macOS/Linux. Created if missing.

## Config file location

| Platform | Path |
|---|---|
| Windows | `%APPDATA%\CPC\.cpc-config.toml` |
| macOS | `~/Library/Application Support/CPC/.cpc-config.toml` |
| Linux | `~/.config/cpc/.cpc-config.toml` |

## Non-interactive guarantee

When stdin is not a tty (CI, MCP server stdio mode, scheduled tasks), cpc-paths NEVER prompts. It returns `Error::NotFound` or `Error::Ambiguous` with actionable hints.

This is critical for MCP servers — a stdin prompt would deadlock the JSON-RPC channel. Servers that use cpc-paths should call resolution at startup and log+exit nonzero on `Error`.

## Versioning

- v0.1.x — Windows verified, macOS/Linux candidates in place but untested
- v0.2.0 — macOS verified
- v1.0.0 — All three platforms verified

## License

Apache 2.0
