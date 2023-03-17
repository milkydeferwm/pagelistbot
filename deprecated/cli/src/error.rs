use core::fmt;

#[derive(Debug)]
pub(crate) enum ClientError {
    RPCError(jsonrpsee::core::Error),
    PageListBotError(interface::PageListBotError),
}

impl std::error::Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RPCError(e) => e.fmt(f),
            Self::PageListBotError(e) => e.fmt(f),
        }
    }
}

impl From<jsonrpsee::core::Error> for ClientError {
    fn from(value: jsonrpsee::core::Error) -> Self {
        Self::RPCError(value)
    }
}

impl From<interface::PageListBotError> for ClientError {
    fn from(value: interface::PageListBotError) -> Self {
        Self::PageListBotError(value)
    }
}
