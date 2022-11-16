//! Traits and common data structures for data provider.

pub mod core;
#[cfg(feature="api")]
pub mod api;

// re-exports of core traits and types
pub use crate::core::{DataProvider, PageInfo, PagePair};
