//! Interface between backend daemon and frontends.
//! In a K8s environment, JSON-RPC is used for communication.
//! This crate defines the communication structs.

pub mod rpc;
pub mod error;

// convenient re-exports
pub use rpc::{PageListBotRpcClient, PageListBotRpcServer};
pub use error::PageListBotError;
