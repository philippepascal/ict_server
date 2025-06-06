use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub path: String,
}

pub fn load_config(config_file_path: &str) -> Result<Settings,ConfigError> {
    let builder = Config::builder()
        .add_source(File::with_name(config_file_path))
        .build()?;

    builder.try_deserialize()
}