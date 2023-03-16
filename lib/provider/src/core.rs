use crate::{
    config::{LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig},
    pageinfo::{Pair, PageInfo},
};
use futures::Stream;
use mwtitle::Title;

pub trait DataProvider {
    type Error;
    type PageInfoStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type PageInfoRawStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type LinksStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type BacklinksStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type EmbedsStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type CategoryMembersStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;
    type PrefixStream: Stream<Item = Result<Pair<PageInfo>, Self::Error>>;

    /// Get a stream of input pages' information. Input is `mwtitle::Title`.
    fn get_page_info<T: IntoIterator<Item = Title>>(&self, titles: T) -> Self::PageInfoStream;
    /// Get a stream of input pages' information. Input is raw title string.
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(&self, titles_raw: T) -> Self::PageInfoRawStream;
    /// Get a stream of input pages' internal links.
    fn get_links<T: IntoIterator<Item = Title>>(&self, titles: T, config: &LinksConfig) -> Self::LinksStream;
    /// Get a stream of input pages' back links.
    fn get_backlinks<T: IntoIterator<Item = Title>>(&self, titles: T, config: &BackLinksConfig) -> Self::BacklinksStream;
    /// Get a stream of pages in which the given pages are embedded.
    fn get_embeds<T: IntoIterator<Item = Title>>(&self, titles: T, config: &EmbedsConfig) -> Self::EmbedsStream;
    /// Get a stream of pages inside the given category pages.
    fn get_category_members<T: IntoIterator<Item = Title>>(&self, titles: T, config: &CategoryMembersConfig) -> Self::CategoryMembersStream;
    /// Get a stream of pages containing the given prefix.
    fn get_prefix<T: IntoIterator<Item = Title>>(&self, titles: T, config: &PrefixConfig) -> Self::PrefixStream;
}


