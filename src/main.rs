use crate::backup::Backup;
use crate::compression::{Compression, create_compression};
use crate::config::Config;
use crate::provider::{MultiProvider, Provider};
use anyhow::{Result, anyhow};
use cln_plugin::{Builder, RpcMethodBuilder, options};
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;

#[cfg(not(any(feature = "s3", feature = "webdav")))]
compile_error!("At least one of the following features must be enabled: s3, webdav");

#[cfg(feature = "s3")]
use crate::provider::s3::S3;

#[cfg(feature = "webdav")]
use crate::provider::webdav::WebDav;

mod backup;
mod command;
mod compression;
mod config;
mod provider;
mod subscription;
mod utils;

const BACKUP_CONFIG_PATH: options::DefaultStringConfigOption =
    options::ConfigOption::new_str_with_default(
        "backup-config-path",
        "backup.toml",
        "Path to the backup configuration file (absolute or relative to lightning directory)",
    );

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
    unsafe {
        std::env::set_var(
            "CLN_PLUGIN_LOG",
            "cln_plugin=trace,backup=trace,debug,info,warn,error",
        )
    };

    let plugin = match Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .dynamic()
        .option(BACKUP_CONFIG_PATH)
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

    let config_file = plugin.option(&BACKUP_CONFIG_PATH)?;
    let config_path = if Path::new(&config_file).is_absolute() {
        Path::new(&config_file).to_path_buf()
    } else {
        Path::new(&plugin.configuration().lightning_dir).join(config_file)
    };
    let config = Config::load(&config_path).map_err(|e| {
        anyhow!(
            "Failed to load config file {}: {}",
            config_path.display(),
            e
        )
    })?;

    let mut multi_provider = MultiProvider::new();

    #[cfg(feature = "s3")]
    {
        if let Some(configs) = config.s3 {
            for config in configs {
                match S3::new(config).await {
                    Ok(s3) => multi_provider.add(Arc::new(s3)),
                    Err(e) => warn!("Setting up S3 failed: {e}"),
                }
            }
        }
    }

    #[cfg(feature = "webdav")]
    {
        if let Some(configs) = config.webdav {
            for config in configs {
                match WebDav::new(config) {
                    Ok(webdav) => multi_provider.add(Arc::new(webdav)),
                    Err(e) => warn!("Setting up WebDav failed: {e}"),
                }
            }
        }
    }

    if multi_provider.is_empty() {
        return Err(anyhow!("No providers configured"));
    }

    let compression = create_compression(config.compression)?;

    let backup = Backup::new(
        multi_provider,
        compression,
        &plugin.configuration().rpc_file,
    )
    .await?;

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
