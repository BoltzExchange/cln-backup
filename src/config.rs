use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    #[cfg_attr(feature = "zstd", default)]
    Zstd,
    #[cfg_attr(all(feature = "gzip", not(feature = "zstd")), default)]
    Gzip,
    #[cfg_attr(all(not(feature = "zstd"), not(feature = "gzip")), default)]
    None,
}

#[cfg(feature = "s3")]
#[derive(Clone, Debug, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path: Option<String>,
    pub access_key: String,
    pub secret_key: String,
}

#[cfg(feature = "webdav")]
#[derive(Clone, Debug, Deserialize)]
pub struct WebDavConfig {
    pub endpoint: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub compression: CompressionType,
    #[cfg(feature = "s3")]
    pub s3: Option<Vec<S3Config>>,
    #[cfg(feature = "webdav")]
    pub webdav: Option<Vec<WebDavConfig>>,
}

impl Config {
    pub fn load(config_path: &PathBuf) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path)?;
        Ok(toml::from_str(&config_content)?)
    }
}
