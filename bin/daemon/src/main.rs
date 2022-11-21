//! Main entry of Page List Bot

mod arg;
mod rpc_server;

use std::{collections, sync::Arc};
use futures::{prelude::*, stream::FuturesUnordered, future::Fuse};
use interface::PageListBotRpcServer;
use jsonrpsee::server::ServerBuilder;
use rpc_server::ServerImpl;
use tokio::{sync::RwLock, signal};
use tracing::{event, Level};

const DEFAULT_SERV_PORT: u16 = 7378; // "SERV"

#[tokio::main]
async fn main() {
    // process arguments
    let args = arg::build_args().get_matches();
    if !args.get_flag("skip") {
        // read startup file. currently not implemented.
        unimplemented!()
    }
    // read addr and port
    let addr = args.get_one::<String>("addr").unwrap();
    let port = args.get_one::<u16>("port").unwrap_or(&DEFAULT_SERV_PORT);
    // set up server
    let host_map = Arc::new(RwLock::new(collections::HashMap::new()));
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
