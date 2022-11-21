//! Main entry of Page List Bot

mod arg;
mod rpc_server;
mod startup;

use std::{collections, sync::Arc, path::PathBuf};
use futures::{prelude::*, stream::FuturesUnordered, future::Fuse};
use interface::PageListBotRpcServer;
use jsonrpsee::server::ServerBuilder;
use rpc_server::ServerImpl;
use tokio::{sync::RwLock, signal};
use tracing::{event, Level};

#[tokio::main]
async fn main() {
    // the core, also the heart, of page list bot.
    let host_map = Arc::new(RwLock::new(collections::HashMap::new()));
    // process arguments
    let args = arg::build_args().get_matches();
    if let Some(startup_file) = args.get_one::<PathBuf>("startup") {
        // read startup file.
        let startup_str = std::fs::read_to_string(startup_file);
        if let Ok(startup_str) = startup_str {
            let startup = serde_json::from_str::<startup::StartupConfig>(&startup_str);
            if let Ok(startup) = startup {
                let mut host_map = host_map.write().await;
                for (k, h) in startup.sites.iter() {
                    if let Some(cred) = startup.login.get(&h.login) {
                        // try create `Host`
                        let host = host::Host::try_new(
                            &cred.username,
                            &cred.password,
                            &h.api_endpoint,
                            &h.on_site_config,
                            h.prefer_bot_edit,
                        ).await;
                        if let Ok(host) = host {
                            host_map.insert(k.to_owned(), host);
                            event!(Level::INFO, name=k, "host created on startup.")
                        } else {
                            let err = host.unwrap_err();
                            event!(Level::WARN, name=k, ?err, "cannot create host on startup.")
                        }
                    } else {
                        event!(Level::WARN, name=k, login=&h.login, "cannot find requested login credential in startup file.")
                    }
                }
            } else {
                let err = startup.unwrap_err();
                event!(Level::WARN, ?err, "cannot decode startup file.")
            }
        } else {
            let err = startup_str.unwrap_err();
            event!(Level::WARN, ?err, "cannot read startup file.")
        }
    }
    // read addr and port
    let addr = args.get_one::<String>("addr").unwrap();
    let port = args.get_one::<u16>("port").unwrap();
    // set up server
    let serv = ServerImpl::new(host_map.clone());
    let server = ServerBuilder::default().build(format!("{}:{}", addr, port)).await.unwrap();

	let handle = server.start(serv.into_rpc()).unwrap();
    let mut handle_stop = {
        let handle = handle.clone();
        Box::pin(handle.stopped().fuse())
    };
    // server running and wait for ctrl-c
    futures::select! {
        _ = handle_stop => {
            event!(Level::ERROR, "RPC server unexpetedly stopped.");
        },
        _ = signal::ctrl_c().fuse() => {
            // stop the server
            event!(Level::INFO, "ctrl-c hit, stop RPC server.");
            handle_stop.set(Fuse::terminated());
            if let Err(e) = handle.stop() {
                event!(Level::ERROR, ?e, "cannot stop RPC server.");
            } else {
                handle.stopped().await; // wait for server to truly stop.
            }
        }
    }
    // clean up hosts.
    let cleanup = async move {
        let mut hostmap = host_map.write().await;
        hostmap.drain()
            .into_iter()
            .map(|(_, h)| async move { h.shutdown().await; })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;
    };
    let mut cleanup = Box::pin(cleanup.fuse());
    event!(Level::INFO, "clean up hosts.");
    futures::select! {
        _ = cleanup => {},   // no nothing, everything is fine.
        _ = signal::ctrl_c().fuse() => {
            // second ctrl-c hit, force kill everything.
            cleanup.set(Fuse::terminated());
            event!(Level::WARN, "second ctrl-c hit, force kill.");
        }
    }
}
