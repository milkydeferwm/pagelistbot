//! Interface between backend daemon and frontends.
//! In a K8s environment, JSON-RPC is used for communication.
//! This crate defines the communication structs.

#[cfg(feature = "rpc")]
pub mod rpc;
pub mod error;
pub mod types;

// convenient re-exports
#[cfg(feature = "rpc")]
pub use rpc::{PageListBotRpcClient, PageListBotRpcServer};
pub use error::PageListBotError;
