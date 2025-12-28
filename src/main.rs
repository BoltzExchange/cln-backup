use crate::backup::Backup;
use crate::compression::{Compression, Gzip};
use crate::config::Config;
use crate::provider::{MultiProvider, Provider};
use anyhow::{Result, anyhow};
use cln_plugin::{Builder, RpcMethodBuilder, options};
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;

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
        if let Some(s3_configs) = config.s3 {
            for s3_config in s3_configs {
                if let Err(err) = setup_s3(&mut multi_provider, &s3_config).await {
                    warn!("Setting up S3 failed: {err}");
                }
            }
        }
    }

    #[cfg(feature = "webdav")]
    {
        if let Some(webdav_config) = config.webdav
            && let Err(err) = setup_webdav(&mut multi_provider, &webdav_config)
        {
            warn!("Setting up WebDav failed: {err}");
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
async fn setup_s3(m: &mut MultiProvider, config: &crate::config::S3Config) -> Result<()> {
    m.add(Arc::new(
        S3::new(
            &config.endpoint,
            &config.bucket,
            config.path.as_deref().unwrap_or(""),
            &config.access_key,
            &config.secret_key,
        )
        .await?,
    ));

    Ok(())
}

#[cfg(feature = "webdav")]
fn setup_webdav(m: &mut MultiProvider, config: &crate::config::WebDavConfig) -> Result<()> {
    m.add(Arc::new(WebDav::new(
        config.endpoint.clone(),
        config.user.clone(),
        config.password.clone(),
    )?));

    Ok(())
}
