use anyhow::Result;
use log::error;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub path: Option<String>,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebDavConfig {
    pub endpoint: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub s3: Option<Vec<S3Config>>,
    pub webdav: Option<WebDavConfig>,
}

impl Config {
    pub fn load(config_path: &PathBuf) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path).map_err(|e| {
            error!(
                "Failed to read config file {}: {}",
                config_path.display(),
                e
            );
            e
        })?;

        toml::from_str(&config_content).map_err(|e| {
            error!("Failed to parse config file: {e}");
            e.into()
        })
    }
}
