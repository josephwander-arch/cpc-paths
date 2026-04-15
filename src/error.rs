use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{path_type} not found. Set {env_var} or create {config_file}")]
    NotFound {
        path_type: String,
        env_var: &'static str,
        config_file: PathBuf,
    },

    #[error("Multiple candidates for {path_type} (non-interactive): {candidates:?}")]
    Ambiguous {
        path_type: String,
        candidates: Vec<PathBuf>,
    },

    #[error("Config file I/O error: {0}")]
    ConfigIo(#[from] std::io::Error),

    #[error("Config file parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Config file write error: {0}")]
    ConfigWrite(#[from] toml::ser::Error),
}
