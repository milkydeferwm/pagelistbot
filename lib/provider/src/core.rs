use crate::{
    config::{LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig},
    pageinfo::PageInfo,
};
use futures::{Stream, StreamExt};
use mwtitle::Title;
use trio_result::TrioResult;

/*
pub trait DataProvider:
    PageInfoProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn> +
    LinksProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn> +
    BackLinksProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn> +
    EmbedsProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn> +
    CategoryMembersProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn> +
    PrefixProvider<Error = <Self as DataProvider>::Error, Warn = <Self as DataProvider>::Warn>
{
    type Error;
    type Warn;
}

pub trait PageInfoProvider {
    type Error;
    type Warn;

    /// Get a stream of input pages' information. Input is `mwtitle::Title`.
    fn get_page_info<T: IntoIterator<Item = Title>>(&self, titles: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;
    /// Get a stream of input pages' information. Input is raw title string.
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(&self, titles_raw: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;
}

pub trait LinksProvider {
    type Error;
    type Warn;

    /// Get a stream of input pages' internal links.
    fn get_links(&self, title: Title, config: &LinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_links_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &LinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_links(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}

pub trait BackLinksProvider {
    type Error;
    type Warn;

    /// Get a stream of input pages' back links.
    fn get_backlinks(&self, title: Title, config: &BackLinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_backlinks_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &BackLinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_backlinks(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}

pub trait EmbedsProvider {
    type Error;
    type Warn;

    /// Get a stream of pages in which the given pages are embedded.
    fn get_embeds(&self, title: Title, config: &EmbedsConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_embeds_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &EmbedsConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_embeds(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}

pub trait CategoryMembersProvider {
    type Error;
    type Warn;

    /// Get a stream of pages inside the given category pages.
    fn get_category_members(&self, title: Title, config: &CategoryMembersConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_category_members_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &CategoryMembersConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_category_members(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}

pub trait PrefixProvider {
    type Error;
    type Warn;

    /// Get a stream of pages containing the given prefix.
    fn get_prefix(&self, title: Title, config: &PrefixConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_prefix_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &PrefixConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_prefix(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}
*/

pub trait DataProvider {
    type Error;
    type Warn;

    /// Get a stream of input pages' information. Input is `mwtitle::Title`.
    fn get_page_info<T: IntoIterator<Item = Title>>(&self, titles: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;
    /// Get a stream of input pages' information. Input is raw title string.
    fn get_page_info_from_raw<T: IntoIterator<Item = String>>(&self, titles_raw: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;
    /// Get a stream of input pages' internal links.
    fn get_links(&self, title: Title, config: &LinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_links_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &LinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_links(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
    /// Get a stream of input pages' back links.
    fn get_backlinks(&self, title: Title, config: &BackLinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_backlinks_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &BackLinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_backlinks(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
    /// Get a stream of pages in which the given pages are embedded.
    fn get_embeds(&self, title: Title, config: &EmbedsConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_embeds_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &EmbedsConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_embeds(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
    /// Get a stream of pages inside the given category pages.
    fn get_category_members(&self, title: Title, config: &CategoryMembersConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_category_members_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &CategoryMembersConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_category_members(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
    /// Get a stream of pages containing the given prefix.
    fn get_prefix(&self, title: Title, config: &PrefixConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>>;

    fn get_prefix_multi<T: IntoIterator<Item=Title>>(&self, titles: T, config: &PrefixConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        let streams = titles.into_iter()
            .map(|t| self.get_prefix(t, config))
            .collect::<Vec<_>>();
        futures::stream::iter(streams).flatten()
    }
}
