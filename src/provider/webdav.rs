use crate::provider::Provider;
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use reqwest_dav::{Auth, Client, ClientBuilder};

#[derive(Debug, Clone)]
pub struct WebDav {
    client: Client,
}

impl WebDav {
    pub fn new(config: crate::config::WebDavConfig) -> Result<WebDav> {
        info!("Using WebDAV endpoint: {}", config.endpoint);

        let auth = match (config.user, config.password) {
            (Some(user), Some(password)) => Auth::Basic(user, password),
            (None, None) => Auth::Anonymous,
            (Some(_), None) => {
                return Err(anyhow::anyhow!(
                    "WebDAV user provided but password is missing"
                ));
            }
            (None, Some(_)) => {
                return Err(anyhow::anyhow!(
                    "WebDAV password provided but user is missing"
                ));
            }
        };

        let client = ClientBuilder::new()
            .set_host(config.endpoint)
            .set_auth(auth)
            .build()?;

        Ok(WebDav { client })
    }
}

#[async_trait]
impl Provider for WebDav {
    async fn put(&self, path: &str, data: &[u8]) -> Result<()> {
        self.client.put(path, data.to_vec()).await?;
        Ok(())
    }
}
