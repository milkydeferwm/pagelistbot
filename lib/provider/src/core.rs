//! Core traits and data structures for data provider.

use std::collections::BTreeSet;

use mwtitle::Title;
use interface::types::ast::Modifier;
// use std::error::Error;

/// Common type for a page.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageInfo {
    /// Title object itself
    pub title: Title,
    /// Whether this page exists (not `missing`)
    pub exists: bool,
    /// Whether this page is a redirect (this information can not be inferred when it is an associated page)
    pub redirect: Option<bool>,
}

/// A `PagePair` is a page's info and its associated page's info.
pub type PagePair = (PageInfo, PageInfo);

#[async_trait::async_trait]
pub trait DataProvider: core::fmt::Debug + Send + Sync {
    type Error: std::error::Error + Send + Sync;

    async fn get_page_info(&self, titles: &BTreeSet<String>) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;
    async fn get_links(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;
    async fn get_backlinks(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;
    async fn get_embeds(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;
    async fn get_category_members(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;
    async fn get_prefix(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error>;

}
