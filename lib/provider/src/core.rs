use crate::{
    config::{LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig},
    pageinfo::PageInfo,
};
use futures::{
    Stream, StreamExt,
    stream::{iter, Flatten, Iter},
};
use mwtitle::Title;
use std::vec::IntoIter;
use trio_result::TrioResult;

pub trait DataProvider {
    type Error;
    type Warn;
    type PageInfoStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type PageInfoRawStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type LinksStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type BacklinksStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type EmbedsStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type CategoryMembersStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;
    type PrefixStream: Stream<Item = TrioResult<PageInfo, Self::Warn, Self::Error>>;

    /// Get a stream of input pages' information. Input is `mwtitle::Title`.
    fn get_page_info<T: IntoIterator<Item = Title>>(&self, titles: T) -> Self::PageInfoStream;
    /// Get a stream of input pages' information. Input is raw title string.
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(&self, titles_raw: T) -> Self::PageInfoRawStream;
    /// Get a stream of input pages' internal links.
    fn get_links(&self, title: &Title, config: &LinksConfig) -> Self::LinksStream;
    /// Get a stream of input pages' back links.
    fn get_backlinks(&self, title: &Title, config: &BackLinksConfig) -> Self::BacklinksStream;
    /// Get a stream of pages in which the given pages are embedded.
    fn get_embeds(&self, title: &Title, config: &EmbedsConfig) -> Self::EmbedsStream;
    /// Get a stream of pages inside the given category pages.
    fn get_category_members(&self, title: &Title, config: &CategoryMembersConfig) -> Self::CategoryMembersStream;
    /// Get a stream of pages containing the given prefix.
    fn get_prefix(&self, title: &Title, config: &PrefixConfig) -> Self::PrefixStream;

    fn get_links_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &LinksConfig) -> Flatten<Iter<IntoIter<Self::LinksStream>>> {
        let streams = titles.into_iter().map(|t| self.get_links(&t, config)).collect::<Vec<_>>();
        iter(streams).flatten()
    }

    fn get_backlinks_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &BackLinksConfig) -> Flatten<Iter<IntoIter<Self::BacklinksStream>>> {
        let streams = titles.into_iter().map(|t| self.get_backlinks(&t, config)).collect::<Vec<_>>();
        iter(streams).flatten()
    }

    fn get_embeds_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &EmbedsConfig) -> Flatten<Iter<IntoIter<Self::EmbedsStream>>> {
        let streams = titles.into_iter().map(|t| self.get_embeds(&t, config)).collect::<Vec<_>>();
        iter(streams).flatten()
    }

    fn get_category_members_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &CategoryMembersConfig) -> Flatten<Iter<IntoIter<Self::CategoryMembersStream>>> {
        let streams = titles.into_iter().map(|t| self.get_category_members(&t, config)).collect::<Vec<_>>();
        iter(streams).flatten()
    }

    fn get_prefix_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &PrefixConfig) -> Flatten<Iter<IntoIter<Self::PrefixStream>>> {
        let streams = titles.into_iter().map(|t| self.get_prefix(&t, config)).collect::<Vec<_>>();
        iter(streams).flatten()
    }
}
