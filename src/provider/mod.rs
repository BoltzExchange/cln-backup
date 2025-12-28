use async_trait::async_trait;
use futures::future;
use std::sync::Arc;

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "webdav")]
pub mod webdav;

#[async_trait]
pub trait Provider {
    async fn put(&self, path: &str, data: &[u8]) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct MultiProvider {
    providers: Vec<Arc<dyn Provider + Send + Sync>>,
}

impl MultiProvider {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add(&mut self, p: Arc<dyn Provider + Send + Sync>) {
        self.providers.push(p);
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[async_trait]
impl Provider for MultiProvider {
    async fn put(&self, path: &str, data: &[u8]) -> anyhow::Result<()> {
        future::try_join_all(self.providers.iter().map(|p| p.put(path, data))).await?;

        Ok(())
    }
}
