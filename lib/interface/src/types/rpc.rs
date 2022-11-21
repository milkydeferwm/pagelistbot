//! Types used by RPC.

#![cfg(feature = "rpc")]
#[cfg(feature = "use_serde")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct NewHostConfig {
    pub api_endpoint: String,
    pub username: String,
    pub password: String,
    pub onsite_config: String,
    pub prefer_bot_edit: bool,
    pub db_name: Option<String>,
}
