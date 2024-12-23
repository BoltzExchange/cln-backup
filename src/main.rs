use crate::backup::Backup;
use crate::compression::{Compression, Gzip};
use crate::provider::{MultiProvider, Provider};
use anyhow::{anyhow, Result};
use cln_plugin::{Builder, RpcMethodBuilder};
use log::{info, warn};
use std::sync::Arc;

#[cfg(feature = "s3")]
use crate::provider::s3::S3;

#[cfg(feature = "webdav")]
use crate::provider::webav::WebDav;

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

    let mut plugin = Builder::new(tokio::io::stdin(), tokio::io::stdout());

    #[cfg(feature = "s3")]
    {
        plugin = plugin
            .option(options::S3_ENDPOINT)
            .option(options::S3_BUCKET)
            .option(options::S3_PATH)
            .option(options::S3_ACCESS_KEY)
            .option(options::S3_SECRET_KEY);
    }

    #[cfg(feature = "webdav")]
    {
        plugin = plugin
            .option(options::WEBDAV_ENDPOINT)
            .option(options::WEBDAV_USER)
            .option(options::WEBDAV_PASSWORD);
    }

    let plugin = match plugin
        .dynamic()
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

    let mut multi_provider = MultiProvider::new();

    #[cfg(feature = "s3")]
    {
        if let Err(err) = setup_s3(&mut multi_provider, &plugin).await {
            warn!("Setting up S3 failed: {}", err);
        }
    }

    #[cfg(feature = "webdav")]
    {
        if let Err(err) = setup_webdav(&mut multi_provider, &plugin) {
            warn!("Setting up WebDav failed: {}", err);
        }
    }

    if multi_provider.is_empty() {
        return Err(anyhow!("No providers configured"));
    }

    let backup = Backup::new(multi_provider, Gzip {}, &plugin.configuration().rpc_file).await?;

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

#[cfg(feature = "s3")]
async fn setup_s3(
    m: &mut MultiProvider,
    plugin: &cln_plugin::ConfiguredPlugin<
        State<MultiProvider, Gzip>,
        tokio::io::Stdin,
        tokio::io::Stdout,
    >,
) -> Result<()> {
    let endpoint = options::get_option("S3 endpoint", plugin.option(&options::S3_ENDPOINT))?;
    let bucket = options::get_option("S3 bucket", plugin.option(&options::S3_BUCKET))?;
    let path = plugin.option(&options::S3_PATH)?;
    let access_key = options::get_option("S3 access key", plugin.option(&options::S3_ACCESS_KEY))?;
    let secret_key = options::get_option("S3 secret key", plugin.option(&options::S3_SECRET_KEY))?;

    m.add(Arc::new(
        S3::new(&endpoint, &bucket, &path, &access_key, &secret_key).await?,
    ));

    Ok(())
}

#[cfg(feature = "webdav")]
fn setup_webdav(
    m: &mut MultiProvider,
    plugin: &cln_plugin::ConfiguredPlugin<
        State<MultiProvider, Gzip>,
        tokio::io::Stdin,
        tokio::io::Stdout,
    >,
) -> Result<()> {
    let endpoint =
        options::get_option("WebDAV endpoint", plugin.option(&options::WEBDAV_ENDPOINT))?;
    let user = plugin.option(&options::WEBDAV_USER)?;
    let password = plugin.option(&options::WEBDAV_PASSWORD)?;

    m.add(Arc::new(WebDav::new(endpoint, user, password)?));

    Ok(())
}
