use crate::compression::Compression;
use crate::provider::Provider;
use crate::State;
use cln_plugin::Plugin;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct UploadResponse {}

pub async fn upload<B, C>(plugin: Plugin<State<B, C>>, _args: Value) -> anyhow::Result<Value>
where
    B: Provider + Clone + Send + 'static,
    C: Compression + Clone + Send + 'static,
{
    plugin.state().backup.backup().await?;
    Ok(serde_json::to_value(UploadResponse {})?)
}
