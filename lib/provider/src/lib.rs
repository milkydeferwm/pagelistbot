//! Traits and common data structures for data provider.

#![allow(incomplete_features)]
#![feature(is_some_and)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]

pub mod core;
#[cfg(feature = "api")]
pub mod api;

// re-exports of core traits and types
pub use crate::core::{DataProvider, PageInfo, PageInfoError, Pair};
