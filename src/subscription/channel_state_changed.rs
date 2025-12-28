use crate::State;
use crate::compression::Compression;
use crate::provider::Provider;
use anyhow::Result;
use cln_plugin::Plugin;
use log::error;
use serde_json::Value;

pub async fn channel_state_changed<B, C>(plugin: Plugin<State<B, C>>, _request: Value) -> Result<()>
where
    B: Provider + Clone + Send + 'static,
    C: Compression + Clone + Send + 'static,
{
    if let Err(err) = plugin.state().backup.backup().await {
        error!("Could not upload backup: {err}");
    }

    Ok(())
}
