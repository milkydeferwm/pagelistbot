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
/*
/// common trait for all required methods a data provider should offer.
pub trait DataProvider<'a>: Send + Sync {
    type Error;

    fn get_page_info<T: IntoIterator<Item=Title>>(&'a self, titles: T) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
    fn get_links<T: IntoIterator<Item=Title>>(&'a self, titles: T, modifier: &Modifier) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
    fn get_backlinks<T: IntoIterator<Item=Title>>(&'a self, titles: T, modifier: &Modifier) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
    fn get_embeds<T: IntoIterator<Item=Title>>(&'a self, titles: T, modifier: &Modifier) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
    fn get_category_members<T: IntoIterator<Item=Title>>(&'a self, titles: T, modifier: &Modifier) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
    fn get_prefix<T: IntoIterator<Item=Title>>(&'a self, titles: T, modifier: &Modifier) -> impl Stream<Item = Result<Pair<PageInfo>, Self::Error>> + 'a;
}
*/

pub trait DataProvider:
    PageInfoProvider<Error = <Self as DataProvider>::Error>
    + LinksProvider<Error = <Self as DataProvider>::Error>
    + BackLinksProvider<Error = <Self as DataProvider>::Error>
    + EmbedsProvider<Error = <Self as DataProvider>::Error>
    + CategoryMembersProvider<Error = <Self as DataProvider>::Error>
    + PrefixProvider<Error = <Self as DataProvider>::Error>
{
    type Error: Error;
}

impl<T> DataProvider for T
where
    T: PageInfoProvider
     + LinksProvider<Error = <T as PageInfoProvider>::Error>
     + BackLinksProvider<Error = <T as PageInfoProvider>::Error>
     + EmbedsProvider<Error = <T as PageInfoProvider>::Error>
     + CategoryMembersProvider<Error = <T as PageInfoProvider>::Error>
     + PrefixProvider<Error = <T as PageInfoProvider>::Error>,
    <T as PageInfoProvider>::Error: Error,
{
    type Error = <Self as PageInfoProvider>::Error;
}
/*
unsafe impl<T> Send for T
where
    T: DataProvider,
    <T as PageInfoProvider>::OutputStream: Send,
    <T as LinksProvider>::OutputStream: Send,
    <T as BackLinksProvider>::OutputStream: Send,
    <T as EmbedsProvider>::OutputStream: Send,
    <T as CategoryMembersProvider>::OutputStream: Send,
    <T as PrefixProvider>::OutputStream: Send,
    <T as DataProvider>::Error: Send,
{}
*/
/// trait that would get a page's information
pub trait PageInfoProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_page_info<T: IntoIterator<Item = Title>>(self, titles: T) -> Self::OutputStream;
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(self, titles_raw: T) -> Self::OutputStream;
}

/// trait that would get a stream of page's in-site links.
pub trait LinksProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_links<T: IntoIterator<Item = Title>>(self, titles: T, modifier: &Modifier) -> Self::OutputStream;
}

/// trait that would get a page's backlinks.
pub trait BackLinksProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_backlinks<T: IntoIterator<Item = Title>>(self, titles: T, modifier: &Modifier) -> Self::OutputStream;
}

/// trait that would get a list of all pages who transcludes this page.
pub trait EmbedsProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_embeds<T: IntoIterator<Item = Title>>(self, titles: T, modifier: &Modifier) -> Self::OutputStream;
}

/// trait that would get a category's direct members.
pub trait CategoryMembersProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_category_members<T: IntoIterator<Item = Title>>(self, titles: T, modifier: &Modifier) -> Self::OutputStream;
}

/// trait that would get a list of all pages who contains the specified prefix. This effectively means getting a page's subpages.
pub trait PrefixProvider {
    type Error: Error;
    type OutputStream: TryStream<Ok = Pair<PageInfo>, Error = Self::Error, Item = Result<Pair<PageInfo>, Self::Error>>;

    fn get_prefix<T: IntoIterator<Item = Title>>(self, titles: T, modifier: &Modifier) -> Self::OutputStream;
}
/*
macro_rules! mark_send {
    ($trait:ident) => {
        unsafe impl<T> Send for T
        where
            T: $trait,
            <T as $trait>::Error: Send,
            <T as $trait>::OutputStream: Send,
        {}
    }
}

mark_send!(PageInfoProvider);
mark_send!(LinksProvider);
mark_send!(BackLinksProvider);
mark_send!(EmbedsProvider);
mark_send!(CategoryMembersProvider);
mark_send!(PrefixProvider);
*/