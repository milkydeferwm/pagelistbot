//! Main entry of Page List Bot

use std::{collections, sync::Arc};

mod rpc_server;

#[tokio::main]
async fn main() {
    let host_map = Arc::new(collections::HashMap::new());
    
}
