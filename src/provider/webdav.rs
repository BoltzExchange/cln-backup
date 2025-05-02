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
    pub fn new(endpoint: String, user: Option<String>, password: Option<String>) -> Result<WebDav> {
        info!("Using WebDAV endpoint: {endpoint}");

        let mut auth = Auth::Anonymous;
        if let Some(user) = user {
            if let Some(password) = password {
                auth = Auth::Basic(user, password);
            }
        }

        let client = ClientBuilder::new()
            .set_host(endpoint)
            .set_auth(auth)
            .build()?;

        Ok(WebDav { client })
    }
}

#[async_trait]
impl Provider for WebDav {
    async fn put(&self, path: &str, data: &[u8]) -> Result<()> {
        self.client.put(path, Vec::from(data)).await?;
        Ok(())
    }
}
