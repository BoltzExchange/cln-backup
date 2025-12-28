use crate::compression::Compression;
use crate::provider::Provider;
use anyhow::Result;
use chrono::Utc;
use cln_rpc::ClnRpc;
use cln_rpc::model::requests::StaticbackupRequest;
use cln_rpc::model::responses::StaticbackupResponse;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Backup<B, C> {
    provider: Arc<Mutex<B>>,
    compression: C,
    rcp: Arc<Mutex<ClnRpc>>,
}

impl<B, C> Backup<B, C>
where
    B: Provider,
    C: Compression,
{
    pub async fn new(backup_provider: B, compression: C, rpc_file: &str) -> Result<Backup<B, C>> {
        Ok(Self {
            compression,
            provider: Arc::new(Mutex::new(backup_provider)),
            rcp: Arc::new(Mutex::new(ClnRpc::new(rpc_file).await?)),
        })
    }

    pub async fn backup(&self) -> Result<()> {
        let backup = self.get_data().await?;

        let path = format!(
            "scb-{}.json.{}",
            Self::get_time(),
            self.compression.file_suffix()
        );
        let data = self.compression.compress(&serde_json::to_vec(&backup)?)?;

        self.provider.lock().await.put(path.as_str(), &data).await?;

        info!("Uploaded {} with {} channels", path, backup.scb.len());

        Ok(())
    }

    async fn get_data(&self) -> Result<StaticbackupResponse> {
        Ok(self
            .rcp
            .lock()
            .await
            .call_typed(&StaticbackupRequest {})
            .await?)
    }

    fn get_time() -> String {
        let now = Utc::now().naive_utc();
        now.format("%Y-%m-%d-%H-%M-%S").to_string()
    }
}
