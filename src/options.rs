use anyhow::{anyhow, Result};
use cln_plugin::options;

#[cfg(feature = "s3")]
pub const S3_ENDPOINT: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-endpoint",
    "S3 endpoint to upload backups to",
);

#[cfg(feature = "s3")]
pub const S3_BUCKET: options::StringConfigOption =
    options::ConfigOption::new_str_no_default("backup-s3-bucket", "S3 bucket to upload backups to");

#[cfg(feature = "s3")]
pub const S3_PATH: options::DefaultStringConfigOption = options::ConfigOption::new_str_with_default(
    "backup-s3-path",
    "",
    "Directory in the S3 bucket save the backups in",
);

#[cfg(feature = "s3")]
pub const S3_ACCESS_KEY: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-access-key",
    "Access key of the S3 bucket",
);

#[cfg(feature = "s3")]
pub const S3_SECRET_KEY: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-secret-key",
    "Secret key of the S3 bucket",
);

#[cfg(feature = "webdav")]
pub const WEBDAV_ENDPOINT: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-webdav-endpoint",
    "WebDAV endpoint to upload backups to",
);

#[cfg(feature = "webdav")]
pub const WEBDAV_USER: options::StringConfigOption =
    options::ConfigOption::new_str_no_default("backup-webdav-user", "WebDAV username");

#[cfg(feature = "webdav")]
pub const WEBDAV_PASSWORD: options::StringConfigOption =
    options::ConfigOption::new_str_no_default("backup-webdav-password", "WebDAV password");

pub fn get_option<T>(name: &str, res: Result<Option<T>>) -> Result<T> {
    match res {
        Ok(v) => match v {
            Some(v) => Ok(v),
            None => Err(anyhow!("no {} set", name)),
        },
        Err(err) => Err(anyhow!("invalid {}: {}", name, err)),
    }
}
