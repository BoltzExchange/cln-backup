use crate::provider::Provider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::info;
use s3::creds::Credentials;
use s3::{Bucket, Region};

#[derive(Debug, Clone)]
pub struct S3 {
    path: String,
    bucket: Box<Bucket>,
}

impl S3 {
    pub async fn new(
        endpoint: &str,
        bucket: &str,
        path: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self> {
        info!("Using S3 bucket {} at {}", bucket, endpoint);

        let bucket = Bucket::new(
            bucket,
            Region::Custom {
                endpoint: endpoint.to_string(),
                region: "".to_string(),
            },
            Credentials::new(Some(access_key), Some(secret_key), None, None, None)?,
        )?
        .with_path_style();
        if !bucket.exists().await? {
            return Err(anyhow!("S3 bucket does not exist"));
        }

        Ok(Self {
            bucket,
            path: path
                .to_string()
                .strip_suffix("/")
                .unwrap_or(path)
                .to_string(),
        })
    }

    fn join_path(&self, path: &str) -> String {
        if self.path.is_empty() {
            return path.to_string();
        }

        format!("{}/{}", self.path, path)
    }
}

#[async_trait]
impl Provider for S3 {
    async fn put(&self, path: &str, data: &[u8]) -> anyhow::Result<()> {
        self.bucket.put_object(self.join_path(path), data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::S3;
    use s3::creds::Credentials;
    use s3::{Bucket, Region};

    #[test]
    fn test_join_path() {
        let mut s3 = S3 {
            path: "".to_string(),
            bucket: Bucket::new(
                "",
                Region::Custom {
                    region: "".to_string(),
                    endpoint: "".to_string(),
                },
                Credentials::new(Some(""), Some(""), None, None, None).unwrap(),
            )
            .unwrap(),
        };

        assert_eq!(s3.join_path("f1"), "f1".to_string());

        s3.path = "f2".to_string();
        assert_eq!(s3.join_path("f1"), "f2/f1".to_string());
    }
}
