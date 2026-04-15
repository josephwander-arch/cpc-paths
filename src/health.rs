use crate::{
    platform,
    resolver::{try_resolve, BACKUPS_DESC, INSTALL_DESC, VOLUMES_DESC},
    HealthReport, PathStatus, ResolutionMethod,
};
use std::path::PathBuf;

pub fn build_health_report() -> HealthReport {
    let (vol_path, vol_method) = try_resolve(&VOLUMES_DESC);
    let (inst_path, inst_method) = try_resolve(&INSTALL_DESC);
    let (back_path, back_method) = try_resolve(&BACKUPS_DESC);
    let config_file = crate::config_file();

    HealthReport {
        volumes: path_status(vol_path, vol_method),
        install: path_status(inst_path, inst_method),
        backups: path_status(back_path, back_method),
        config_file: PathStatus {
            exists: config_file.exists(),
            path: Some(config_file),
            resolved_via: ResolutionMethod::PlatformDefault,
            error: None,
        },
        platform: platform::platform_name().to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn path_status(path: Option<PathBuf>, method: ResolutionMethod) -> PathStatus {
    match path {
        Some(p) => {
            let exists = p.exists();
            PathStatus {
                path: Some(p),
                resolved_via: method,
                exists,
                error: None,
            }
        }
        None => PathStatus {
            path: None,
            resolved_via: ResolutionMethod::None,
            exists: false,
            error: Some("Resolution failed — see error variants for details".to_string()),
        },
    }
}
