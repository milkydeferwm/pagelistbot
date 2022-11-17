//! Error types.

use std::{error, fmt};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PageListBotError {
    /// There is already a host by the same name.
    HostAlreadyExists,
    /// There is no such host by the specified name.
    HostDoesNotExist,
    /// There is another kind of host error.
    /// The `HostError` is first converted into string, then passed back.
    HostError(String),
}

impl error::Error for PageListBotError {}
impl fmt::Display for PageListBotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostAlreadyExists => write!(f, "there is already a host by the same name"),
            Self::HostDoesNotExist => write!(f, "there is no host by the specified name"),
            Self::HostError(e) => write!(f, "{}", e),
        }
    }
}
