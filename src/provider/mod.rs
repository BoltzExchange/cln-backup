use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::task::JoinSet;

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
        let mut set = JoinSet::new();

        for p in &self.providers {
            let p = p.clone();
            let path = path.to_owned();
            let data = data.to_owned();

            set.spawn(async move { p.put(&path, &data).await });
        }

        for res in set.join_all().await.iter() {
            if let Err(err) = res {
                return Err(anyhow!("{}", err));
            }
        }

        Ok(())
    }
}
