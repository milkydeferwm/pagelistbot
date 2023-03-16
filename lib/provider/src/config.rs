//! Configuration structs for `DataProvider` trait.

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterRedirect {
    NoRedirect,
    OnlyRedirect,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinksConfig {
    pub namespace: Option<HashSet<i32>>,
    pub resolve_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackLinksConfig {
    pub direct: bool,
    pub filter_redirects: Option<FilterRedirect>,
    pub namespace: Option<HashSet<i32>>,
    pub resolve_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmbedsConfig {
    pub filter_redirects: Option<FilterRedirect>,
    pub namespace: Option<HashSet<i32>>,
    pub resolve_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CategoryMembersConfig {
    pub namespace: Option<HashSet<i32>>,
    pub resolve_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrefixConfig {
    pub filter_redirects: Option<FilterRedirect>,
    pub namespace: Option<HashSet<i32>>,
}