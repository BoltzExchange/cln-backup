use crate::backup::Backup;
use crate::compression::{Compression, Gzip};
use crate::provider::s3::S3;
use crate::provider::Provider;
use anyhow::{anyhow, Result};
use cln_plugin::{Builder, RpcMethodBuilder};
use log::info;

mod backup;
mod command;
mod compression;
mod options;
mod provider;
mod subscription;
mod utils;

#[derive(Clone)]
struct State<B, C>
where
    B: Provider + Clone + Send + 'static,
    C: Compression + Clone + Send + 'static,
{
    backup: Backup<B, C>,
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var(
        "CLN_PLUGIN_LOG",
        "cln_plugin=trace,backup=trace,debug,info,warn,error",
    );

    let plugin = match Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .dynamic()
        .option(options::ENDPOINT)
        .option(options::BUCKET)
        .option(options::PATH)
        .option(options::ACCESS_KEY)
        .option(options::SECRET_KEY)
        .subscribe("channel_state_changed", subscription::channel_state_changed)
        .rpcmethod_from_builder(
            RpcMethodBuilder::new("staticbackup-upload", command::upload)
                .description("Uploads a static backup of all channels"),
        )
        .configure()
        .await?
    {
        Some(plugin) => plugin,
        None => return Err(anyhow!("could not build plugin")),
    };

    let endpoint = options::get_option("S3 endpoint", plugin.option(&options::ENDPOINT))?;
    let bucket = options::get_option("S3 bucket", plugin.option(&options::BUCKET))?;
    let path = plugin.option(&options::PATH)?;
    let access_key = options::get_option("S3 access key", plugin.option(&options::ACCESS_KEY))?;
    let secret_key = options::get_option("S3 secret key", plugin.option(&options::SECRET_KEY))?;

    let s3 = S3::new(&endpoint, &bucket, &path, &access_key, &secret_key).await?;
    let backup = Backup::new(s3, Gzip {}, &plugin.configuration().rpc_file).await?;

    let plugin = plugin
        .start(State {
            backup: backup.clone(),
        })
        .await?;

    info!(
        "Starting plugin {}-{}{}",
        utils::built_info::PKG_VERSION,
        utils::built_info::GIT_COMMIT_HASH_SHORT.unwrap_or(""),
        if utils::built_info::GIT_DIRTY.unwrap_or(false) {
            "-dirty"
        } else {
            ""
        }
    );

    backup.backup().await?;
    plugin.join().await?;

    info!("Stopped plugin");
    Ok(())
}
