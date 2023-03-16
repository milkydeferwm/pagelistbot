//! Traits and common data structures for data provider.

pub mod config;
pub mod core;
pub mod pageinfo;

// re-exports of core traits and types
pub use crate::config::{
    FilterRedirect,
    LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
pub use crate::core::DataProvider;
pub use crate::pageinfo::{
    PageInfo, PageInfoError, Pair,
};
