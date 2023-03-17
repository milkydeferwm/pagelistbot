use core::future::{Ready, ready};
use futures::{StreamExt, future::Either, stream::{self, Once, Iter, Flatten}};
use itertools::Itertools;
use mwapi::Client;
use mwtitle::{Title, TitleCodec};
use provider::{
    DataProvider, PageInfo,
    FilterRedirect, LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
use std::{collections::HashMap, vec::IntoIter};

mod error;
mod query;
use error::APIDataProviderError;
use query::{QueryStream, query_complete};

#[derive(Debug, Clone, Copy)]
pub struct APIDataProvider<'p> {
    api: &'p Client,
    title_codec: &'p TitleCodec,
    api_highlimit: bool,
}

impl<'p> APIDataProvider<'p> {
    pub fn new(api: &'p Client, title_codec: &'p TitleCodec, api_highlimit: bool) -> Self {
        APIDataProvider {
            api,
            title_codec,
            api_highlimit,
        }
    }
}

impl<'p> DataProvider for APIDataProvider<'p> {
    type Error = APIDataProviderError;

    // impl for `pageinfo`.
    type PageInfoStream = Flatten<Iter<IntoIter<QueryStream<'p>>>>;

    /// Fetch a set of pages' basic information.
    /// This function essentially calls 
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&titles=<titles>```
    /// 
    /// This function is called by `Page` expression. It is assumed that nobody would **hand-write** thousands of page names in a query.
    /// 
    /// This function is not intended to be called during some intermediate step, because at that time there would already be thousands of pages to be queried.
    fn get_page_info<T: IntoIterator<Item=Title>>(&self, titles: T) -> Self::PageInfoStream {
        let chunk_size = if self.api_highlimit { 500 } else { 50 };
        let title_chunks: Vec<Vec<Title>> = titles.into_iter()
            .chunks(chunk_size).into_iter()
            .map(|f| f.collect())
            .collect();
        let mut streams = Vec::with_capacity(title_chunks.len());
        for title_chunk in title_chunks {
            let params = HashMap::from_iter([
                ("titles".to_string(), title_chunk.into_iter().map(|t| self.title_codec.to_pretty(&t)).join("|"))
            ]);
            streams.push(query_complete(self.api, self.title_codec, params));
        }
        stream::iter(streams).flatten()
    }

    // impl for `pageinfo`.
    type PageInfoRawStream = Either<Self::PageInfoStream, Once<Ready<Result<PageInfo, Self::Error>>>>;

    /// Basically the same as `get_page_info`, but convert from string.
    fn get_page_info_from_raw<T: IntoIterator<Item=String>>(&self, titles_raw: T) -> Self::PageInfoRawStream {
        // try convert all
        let titles: Result<Vec<Title>, Self::Error> = titles_raw.into_iter()
            .map(|raw| self.title_codec.new_title(&raw))
            .try_collect()
            .map_err(|e| e.into());
        match titles {
            Ok(titles) => Either::Left(self.get_page_info(titles)),
            Err(e) => Either::Right(stream::once(ready(Err(e)))),
        }
    }

    // impl for `links`.
    type LinksStream = QueryStream<'p>;

    /// Fetch a page's links on that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=links&gplnamespace=<ns>&gpllimit=max&redirects=<resolve>&titles=<titles>```
    /// 
    /// This function is called by `Link` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_links(&self, title: &Title, config: &LinksConfig) -> Self::LinksStream {
        let param = {
            let mut tmp = HashMap::<String, String>::from_iter([
                ("generator".to_string(), "links".to_string()),
                ("titles".to_string(), self.title_codec.to_pretty(title)),
                ("gpllimit".to_string(), "max".to_string()),
            ]);
            if config.resolve_redirects {
                tmp.insert("redirects".to_string(), "1".to_string());
            }
            if let Some(ns) = config.namespace.as_ref() {
                tmp.insert("gplnamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|"));
            }
            tmp
        };
        query_complete(self.api, self.title_codec, param)
    }

    // impl for `backlinks`
    type BacklinksStream = QueryStream<'p>;

    /// Fetch a page's backlinks to that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=backlinks&gblnamespace=<ns>&gbllimit=max&gbltitle=<title>&gblfilterredir=<filter>&gblredirect=<direct>&redirects=<resolve>```
    /// 
    /// This function is called by `LinkTo` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_backlinks(&self, title: &Title, config: &BackLinksConfig) -> Self::BacklinksStream {
        let param = {
            let mut tmp = HashMap::<String, String>::from_iter([
                ("generator".to_string(), "backlinks".to_string()),
                ("gbltitle".to_string(), self.title_codec.to_pretty(title)),
                ("gbllimit".to_string(), "max".to_string()),
            ]);
            if let Some(filter_redirects) = config.filter_redirects {
                tmp.insert(
                    "gblfilterredir".to_string(),
                    match filter_redirects {
                        FilterRedirect::NoRedirect => "nonredirects".to_string(),
                        FilterRedirect::OnlyRedirect => "redirects".to_string(),
                    }
                );
            }
            if !config.direct {
                tmp.insert("gblredirect".to_string(), "1".to_string());
            }
            if config.resolve_redirects {
                tmp.insert("redirects".to_string(), "1".to_string());
            }
            if let Some(ns) = &config.namespace {
                tmp.insert("gblnamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|"));
            }
            tmp
        };
        query_complete(self.api, self.title_codec, param)
    }

    // impl for `embeds`.
    type EmbedsStream = QueryStream<'p>;

    /// Fetch a page's embeds.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=embeddedin&geinamespace=<ns>&geilimit=max&geititle=<title>&geifilterredir=<filter>&redirects=<resolve>```
    /// 
    /// This function is called by `Embed` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_embeds(&self, title: &Title, config: &EmbedsConfig) -> Self::EmbedsStream {
        let param = {
            let mut tmp = HashMap::<String, String>::from_iter([
                ("generator".to_string(), "embeddedin".to_string()),
                ("geititle".to_string(), self.title_codec.to_pretty(title)),
                ("geilimit".to_string(), "max".to_string()),
            ]);
            if let Some(filter_redirects) = config.filter_redirects {
                tmp.insert(
                    "geifilterredir".to_string(),
                    match filter_redirects {
                        FilterRedirect::NoRedirect => "nonredirects".to_string(),
                        FilterRedirect::OnlyRedirect => "redirects".to_string(),
                    }
                );
            }
            if config.resolve_redirects {
                tmp.insert("redirects".to_string(), "1".to_string());
            }
            if let Some(ns) = &config.namespace {
                tmp.insert("geinamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|"));
            }
            tmp
        };
        query_complete(self.api, self.title_codec, param)
    }

    // impl for `categorymember`.
    type CategoryMembersStream = QueryStream<'p>;

    /// Fetch a category's members.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=categorymembers&gcmtitle=<title>&gcmlimit=max&gcmnamespace=<ns>&gcmtype=<...>&redirects=<resolve>```
    /// 
    /// This function is called by `InCat` expression.
    fn get_category_members(&self, title: &Title, config: &CategoryMembersConfig) -> Self::CategoryMembersStream {
        let param = {
            let mut tmp = HashMap::<String, String>::from_iter([
                ("generator".to_string(), "categorymembers".to_string()),
                ("gcmtitle".to_string(), self.title_codec.to_pretty(title)),
                ("gcmlimit".to_string(), "max".to_string()),
            ]);
            if config.resolve_redirects {
                tmp.insert("redirects".to_string(), "1".to_string());
            }
            if let Some(ns) = config.namespace.as_ref() {
                tmp.insert("gcmnamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|"));

                let mut ns = ns.to_owned();
                let mut cmtype = Vec::new();
                if ns.remove(&6) {
                    cmtype.push("file".to_string());
                }
                if ns.remove(&14) {
                    cmtype.push("subcat".to_string());
                }
                if !ns.is_empty() {
                    cmtype.push("page".to_string());
                }
                tmp.insert("gcmtype".to_string(), cmtype.join("|"));
            }
            tmp
        };
        query_complete(self.api, self.title_codec, param)
    }

    // impl for `prefix`.
    type PrefixStream = QueryStream<'p>;

    /// Fetch a page's subpages.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=allpages&gapprefix=<title>&gaplimit=max&gapnamespace=<title>&gapfilterredir=<filter>```
    /// 
    /// This function is called by `Prefix` expression.
    /// A warning will be thrown if `titles` contains more than one page.
    /// This function ignores the `resolve` modifier.
    fn get_prefix(&self, title: &Title, config: &PrefixConfig) -> Self::PrefixStream {
        let param = {
            let mut tmp = HashMap::<String, String>::from_iter([
                ("generator".to_string(), "allpages".to_string()),
                ("gaptitle".to_string(), title.dbkey().to_string()),
                ("gapnamespace".to_string(), title.namespace().to_string()),
                ("gaplimit".to_string(), "max".to_string()),
            ]);
            if let Some(filter_redirects) = config.filter_redirects {
                tmp.insert(
                    "gapfilterredir".to_string(),
                    match filter_redirects {
                        FilterRedirect::NoRedirect => "nonredirects".to_string(),
                        FilterRedirect::OnlyRedirect => "redirects".to_string(),
                    }
                );
            }
            tmp
        };
        query_complete(self.api, self.title_codec, param)
    }
}
