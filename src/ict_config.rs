use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub totp: Totp,
    pub logs: Logs,
    pub pi: Pi,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Totp {
    pub sha: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Logs {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pi {
    pub close_duration: u64,
}

pub fn load_config(config_file_path: &str) -> Result<Settings,ConfigError> {
    let builder = Config::builder()
        .add_source(File::with_name(config_file_path))
        .build()?;

    builder.try_deserialize()
}
