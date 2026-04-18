# cpc-paths

Portable path discovery for CPC MCP servers.

**Part of [CPC](https://github.com/josephwander-arch) (Cognitive Performance Computing)** — a multi-agent AI orchestration platform. Related repos: [manager](https://github.com/josephwander-arch/manager) · [local](https://github.com/josephwander-arch/local) · [hands](https://github.com/josephwander-arch/hands) · [workflow](https://github.com/josephwander-arch/workflow) · [cpc-breadcrumbs](https://github.com/josephwander-arch/cpc-breadcrumbs)

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
cpc-paths = { git = "https://github.com/josephwander-arch/cpc-paths.git", tag = "v0.1.0" }
```

## What it does

Resolves three core CPC paths (Volumes, install, backups) plus per-server data directories with consistent precedence: env var → config file → auto-detect → prompt (interactive only) → error.

cpc-paths uses a two-tier resolution model: the primary tier checks env vars and config files for explicit paths, while the fallback tier auto-detects from known platform-specific candidates. This means zero configuration on standard installs while still supporting custom layouts.

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

## Build from Source

```bash
git clone https://github.com/josephwander-arch/cpc-paths.git
cd cpc-paths
cargo build
```

This is a library crate — no binary is produced. Requires Rust stable toolchain.

## Requirements

- Rust stable toolchain
- Windows 10/11, macOS, or Linux (Windows fully verified; macOS/Linux have candidates but are less tested)

## Versioning

- v0.1.x — Windows verified, macOS/Linux candidates in place but untested
- v0.2.0 — macOS verified
- v1.0.0 — All three platforms verified

## Contributing

Issues welcome; PRs considered but this is primarily maintained as part of the CPC stack.

## License

Licensed under Apache-2.0 — see [LICENSE](LICENSE).
