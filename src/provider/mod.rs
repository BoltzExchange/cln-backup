use async_trait::async_trait;

pub mod s3;

#[async_trait]
pub trait Provider {
    async fn put(&self, path: &str, data: &[u8]) -> anyhow::Result<()>;
}
