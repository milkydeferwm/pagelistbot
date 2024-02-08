//! Page List Bot API daemon process.
//! 
//! This program manages all API connections to the MediaWiki instance.
//! Every API call should be proxied through this program.
//! 
//! In addition to holding connections, this program also checks the startup script periodically,
//! adding or removing connections as necessary, 
//! and refreshing existing connections.

use clap::Parser;
use pagelistbot_api_daemon_interface::APIServiceInterfaceServer;
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing_subscriber::prelude::*;

mod connection;
mod rpc;

#[derive(Debug, Clone, Parser)]
struct Arg {
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    bind_all: bool,
    #[arg(short = 'p', long = "port", default_value_t = pagelistbot_api_daemon_interface::DEFAULT_BACKEND_PORT)]
    port: u16,
}

/// `APIConnection` is the main interface to interact with. It contains
/// * A `mwapi::Client` object, where all data transmission is done.
/// * A `serde_json::Value` object holding the raw siteinfo query response.
/// * A boolean flag indicating whether this client has `bot` user right.
/// * A boolean flag indication whether this client has `apihighlimits` user right.
#[derive(Debug, Clone)]
struct APIConnection {
    client: mwapi::Client,
    site_info: serde_json::Value,
    bot: bool,
    apihighlimits: bool,
}

#[tokio::main]
async fn main() {
    let arg = Arg::parse();
    let config_path = arg.config.to_owned().unwrap_or(pagelistbot_env::pagelistbot_home().join("config.toml"));
    let api_store: Arc<RwLock<HashMap<String, APIConnection>>> = Arc::new(RwLock::new(HashMap::new()));
    // set up log writer
    let (non_blocking_logfile, _logfile_guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(pagelistbot_env::pagelistbot_log(), "api-backend.log"));
    let (non_blocking_stderr, _stderr_guard) = tracing_appender::non_blocking(std::io::stderr());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_logfile)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_stderr)
                .with_ansi(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
        )
        .init();
    // set up refresh routine
    let refresh_handle = {
        let api_store = api_store.clone();
        let config_path = config_path.clone();
        tokio::spawn(load_config(config_path, api_store))
    };
    // set up RPC server
    let server_handle = {
        let addr = if arg.bind_all { "0.0.0.0" } else { "127.0.0.1" };
        let port = arg.port;
        tracing::info!("API backend serving at `{}:{}`", addr, port);
        let api_store = api_store.clone();
        let serv = rpc::APIServiceImpl::new(api_store);
        let server = jsonrpsee::server::ServerBuilder::default().build(format!("{addr}:{port}")).await.unwrap();
        server.start(serv.into_rpc())
    };

    tokio::select! {
        _ = server_handle.stopped() => {
            tracing::error!("RPC server unexceptedly stopped");
        },
        res = refresh_handle => {
            match res {
                Err(e) => {
                    tracing::error!(error=e.to_string(), "cannot join refresh handle");
                },
                Ok(_) => unreachable!(),
            }
            tracing::error!("API refresh service unexceptedly stopped");
        },
        res = tokio::signal::ctrl_c() => {
            match res {
                Err(e) => {
                    tracing::error!(error=e.to_string(), "cannot listen to signal");
                },
                Ok(_) => tracing::info!("ctrl-c received, shut down API backend"),
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiLoginConfig {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    api: String,
}

type ConfigFile = HashMap<String, ApiLoginConfig>;

async fn load_config<P>(path: P, store: Arc<RwLock<HashMap<String, APIConnection>>>) -> !
where
    P: AsRef<Path>,
{
    loop {
        '_mainscope: {
            let config = match fs::read_to_string(path.as_ref()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(warning=e.to_string(), "cannot read configuration file");
                    break '_mainscope;
                }
            };
            let config = match toml::from_str::<ConfigFile>(&config) {
                Ok(x) => x,
                Err(e) => {
                    tracing::warn!(warning=e.to_string(), "cannot parse configuration file");
                    break '_mainscope;
                }
            };
            // update the hashmap.
            let mut store = store.write().await;
            // flush out all connections that no longer exist in the configuration.
            store.retain(|k, _| {
                let preserve = config.contains_key(k);
                if !preserve {
                    tracing::info!("dropped `{}`", k);
                }
                preserve
            });
            // add or replace other connections.
            for (k, v) in config {
                if let Some(new_connection) = connection::get_provider(&v.api, &v.username, &v.password).await {
                    // replace the old connection with the new one.
                    // the old one is automatically dropped.
                    tracing::info!("added `{}`", &k);
                    store.insert(k, new_connection);
                } else {
                    // new connection generation failed, drop the existing connection.
                    // TODO: or should we retain the existing connection?
                    tracing::warn!("dropped `{}`", &k);
                    store.remove(&k);
                }
            }
            break '_mainscope;
        }
        tokio::time::sleep(Duration::from_secs(3600)).await;  // update once per hour.
    }
}
