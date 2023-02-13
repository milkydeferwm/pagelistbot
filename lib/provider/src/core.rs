use core::{cmp::Ordering, fmt};
use std::error::Error;

use futures::TryStream;
use mwtitle::Title;
use interface::types::ast::Modifier;

/// a struct holding the queried wiki page information.
#[derive(Debug, Clone)]
pub struct PageInfo {
    title: Option<Title>,
    exists: Option<bool>,
    redirect: Option<bool>,
}

impl PageInfo {
    /// creates a new `PageInfo` instance.
    pub fn new(title: Option<Title>, exists: Option<bool>, redirect: Option<bool>) -> Self {
        Self { title, exists, redirect }
    }
    /// get a reference to the title, returns an error if such value is not known aka not stored.
    pub fn get_title(&self) -> Result<&Title, PageInfoError> {
        self.title.as_ref().ok_or(PageInfoError::UnknownValue)
    }

    /// get a bool indicating whether this page exists on the wiki, returns an error if such value is not known aka not stored.
    pub fn get_exists(&self) -> Result<bool, PageInfoError> {
        self.exists.ok_or(PageInfoError::UnknownValue)
    }

    /// get a bool indicating whether this page is a redirect page, returns an error if such value is not known aka not stored.
    pub fn get_isredir(&self) -> Result<bool, PageInfoError> {
        self.redirect.ok_or(PageInfoError::UnknownValue)
    }
}

impl From<PageInfo> for Title {
    fn from(f: PageInfo) -> Self {
        f.title.unwrap()
    }
}

impl PartialOrd for PageInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.title.partial_cmp(&other.title)
    }
}

impl Ord for PageInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.title.cmp(&other.title)
    }
}

impl Eq for PageInfo {}
impl PartialEq for PageInfo {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

pub type Pair<T> = (T, T);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageInfoError {
    UnknownValue,
}

impl Error for PageInfoError {}
impl fmt::Display for PageInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownValue => write!(f, "unknown value"),
        }
    }
}

pub trait DataProvider {
    type Error: Error;
    type PageInfoStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type PageInfoRawStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type LinksStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type BacklinksStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type EmbedsStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type CategoryMembersStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;
    type PrefixStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    /// Get a stream of input pages' information. Input is `mwtitle::Title`.
    fn get_page_info<T: IntoIterator<Item = Title>>(&self, titles: T) -> Self::PageInfoStream;
    /// Get a stream of input pages' information. Input is raw title string.
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(&self, titles_raw: T) -> Self::PageInfoRawStream;
    /// Get a stream of input pages' internal links.
    fn get_links<T: IntoIterator<Item = Title>>(&self, titles: T, modifier: &Modifier) -> Self::LinksStream;
    /// Get a stream of input pages' back links.
    fn get_backlinks<T: IntoIterator<Item = Title>>(&self, titles: T, modifier: &Modifier) -> Self::BacklinksStream;
    /// Get a stream of pages in which the given pages are embedded.
    fn get_embeds<T: IntoIterator<Item = Title>>(&self, titles: T, modifier: &Modifier) -> Self::EmbedsStream;
    /// Get a stream of pages inside the given category pages.
    fn get_category_members<T: IntoIterator<Item = Title>>(&self, titles: T, modifier: &Modifier) -> Self::CategoryMembersStream;
    /// Get a stream of pages containing the given prefix.
    fn get_prefix<T: IntoIterator<Item = Title>>(&self, titles: T, modifier: &Modifier) -> Self::PrefixStream;
}
