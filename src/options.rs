use anyhow::{anyhow, Result};
use cln_plugin::options;

pub const ENDPOINT: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-endpoint",
    "S3 endpoint to upload backups to",
);

pub const BUCKET: options::StringConfigOption =
    options::ConfigOption::new_str_no_default("backup-s3-bucket", "S3 bucket to upload backups to");

pub const PATH: options::DefaultStringConfigOption = options::ConfigOption::new_str_with_default(
    "backup-s3-path",
    "",
    "Directory in the S3 bucket save the backups in",
);

pub const ACCESS_KEY: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-access-key",
    "Access key of the S3 bucket",
);

pub const SECRET_KEY: options::StringConfigOption = options::ConfigOption::new_str_no_default(
    "backup-s3-secret-key",
    "Secret key of the S3 bucket",
);

pub fn get_option<T>(name: &str, res: Result<Option<T>>) -> Result<T> {
    match res {
        Ok(v) => match v {
            Some(v) => Ok(v),
            None => Err(anyhow!("no {} set", name)),
        },
        Err(err) => Err(anyhow!("invalid {}: {}", name, err)),
    }
}
