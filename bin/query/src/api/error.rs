use core::fmt::{Display, Formatter};
use std::error::Error;

#[derive(Debug, Clone)]
pub enum APIDataProviderError {
    Api(mwapi::Error),
    TitleCodec(mwtitle::Error),
}

impl From<mwapi::Error> for APIDataProviderError {
    fn from(value: mwapi::Error) -> Self {
        Self::Api(value)
    }
}

impl From<mwtitle::Error> for APIDataProviderError {
    fn from(value: mwtitle::Error) -> Self {
        Self::TitleCodec(value)
    }
}

impl Display for APIDataProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(e) => e.fmt(f),
            Self::TitleCodec(e) => e.fmt(f),
        }
    }
}

impl Error for APIDataProviderError {}
