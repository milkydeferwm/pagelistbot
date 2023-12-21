use async_stream::stream;
use core::convert::Infallible;
use futures::Stream;
use itertools::Itertools;
use jsonrpsee::core::ClientError;
use mwapi_responses::{query, ApiResponse};
use mwtitle::{Title, TitleCodec, SiteInfoResponse};
use pagelistbot_api_daemon_interface::APIServiceInterfaceClient;
use provider::{
    DataProvider, PageInfo,
    FilterRedirect, LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
use std::collections::HashMap;
use trio_result::TrioResult;

#[query(
    prop = "info",
    inprop = "associatedpage|subjectid|talkid",
)]
struct QueryResponse;

#[derive(Debug, Clone)]
pub struct APIDataProvider<B> {
    backend: B,
    key: String,
    title_codec: TitleCodec,
    apihighlimits: bool,
}

impl<B> APIDataProvider<B>
where
    B: APIServiceInterfaceClient + Sync,
{
    pub async fn new(connection: B, key: &str) -> Result<Self, APIDataProviderError> {
        let title_codec = {
            let siteinfo = connection.get_site_info(key).await?;
            let siteinfo: SiteInfoResponse = serde_json::from_value(siteinfo)?;
            TitleCodec::from_site_info(siteinfo.query)?
        };
        let apihighlimits = connection.get_apihighlimits(key).await?;
        Ok(APIDataProvider {
            backend: connection,
            key: key.to_owned(),
            title_codec,
            apihighlimits,
        })
    }

    pub fn to_pretty(&self, title: &Title) -> String {
        self.title_codec.to_pretty(title)
    }

    fn query_all(&self, mut params: HashMap<String, String>) -> impl Stream<Item=TrioResult<PageInfo, Infallible, APIDataProviderError>> + '_ {
        stream! {
            // set up query parameters
            params.insert("action".to_string(), "query".to_string());
            for (k, v) in QueryResponse::params() {
                params.insert(k.to_string(), v.to_string());
            }
            // set up continue
            let mut continue_: Option<HashMap<String, String>> = None;
            while !(continue_.as_ref().is_some_and(|c| c.is_empty())) {
                // insert continue params, if needed.
                let mut params = params.clone();
                if let Some(continue_) = continue_ {
                    params.extend(continue_);
                }
                // try get response, if error then return the error.
                let resp: QueryResponse = {
                    match self.backend.post_value(&self.key, params).await {
                        Ok(x) => match serde_json::from_value(x) {
                            Ok(v) => v,
                            Err(e) => { yield TrioResult::Err(e.into()); return; },
                        },
                        Err(e) => { yield TrioResult::Err(e.into()); return; },
                    }
                };
                // register new continue param.
                continue_ = Some(resp.continue_);
                // read response and extract page info.
                for page in resp.query.pages {
                    // get information for subject page, if error then return the error.
                    let thispage_title = match self.title_codec.new_title(&page.title) {
                        Ok(t) => Some(t),
                        Err(e) => { yield TrioResult::Err(e.into()); return; },
                    };
                    let thispage_exists = Some(!page.missing);
                    let thispage_redirect = Some(page.redirect);

                    let associated_title = match self.title_codec.new_title(&page.associatedpage) {
                        Ok(t) => Some(t),
                        Err(e) => { yield TrioResult::Err(e.into()); return; },
                    };
                    let associated_exists = Some(page.subjectid.is_some() || page.talkid.is_some());
                    let associated_redirect = None;

                    yield TrioResult::Ok(PageInfo::new(thispage_title, thispage_exists, thispage_redirect, associated_title, associated_exists, associated_redirect));
                }
            }
        }
    }
}

impl<B> DataProvider for APIDataProvider<B>
where
    B: APIServiceInterfaceClient + Sync,
{
    type Error = APIDataProviderError;
    type Warn = Infallible;

    /// Fetch a set of pages' basic information.
    /// This function essentially calls 
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&titles=<titles>```
    /// 
    /// This function is called by `Page` expression. It is assumed that nobody would **hand-write** thousands of page names in a query.
    /// 
    /// This function is not intended to be called during some intermediate step, because at that time there would already be thousands of pages to be queried.
    fn get_page_info<T: IntoIterator<Item=Title>>(&self, titles: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            let chunk_size = if self.apihighlimits { 500 } else { 50 };
            let title_chunks: Vec<Vec<Title>> = titles.into_iter()
                .chunks(chunk_size).into_iter()
                .map(|f| f.collect())
                .collect();
            for title_chunk in title_chunks {
                let params = HashMap::from_iter([
                    ("titles".to_string(), title_chunk.into_iter().map(|t| self.title_codec.to_pretty(&t)).join("|"))
                ]);
                for await x in self.query_all(params) { yield x; }
            }
        }
    }

    /// Basically the same as `get_page_info`, but convert from string.
    fn get_page_info_from_raw<T: IntoIterator<Item=String>>(&self, titles_raw: T) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            // try convert all
            let titles: Result<Vec<Title>, Self::Error> = titles_raw.into_iter()
                .map(|raw| self.title_codec.new_title(&raw))
                .try_collect()
                .map_err(|e| e.into());
            match titles {
                Ok(titles) => for await item in self.get_page_info(titles) { yield item; },
                Err(e) => yield TrioResult::Err(e),
            }
        }
    }

    /// Fetch a page's links on that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=links&gplnamespace=<ns>&gpllimit=max&redirects=<resolve>&titles=<titles>```
    /// 
    /// This function is called by `Link` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_links(&self, title: Title, config: &LinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            let param = {
                let mut tmp = HashMap::<String, String>::from_iter([
                    ("generator".to_string(), "links".to_string()),
                    ("titles".to_string(), self.title_codec.to_pretty(&title)),
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
            for await x in self.query_all(param) { yield x; }
        }
    }

    /// Fetch a page's backlinks to that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=backlinks&gblnamespace=<ns>&gbllimit=max&gbltitle=<title>&gblfilterredir=<filter>&gblredirect=<direct>&redirects=<resolve>```
    /// 
    /// This function is called by `LinkTo` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_backlinks(&self, title: Title, config: &BackLinksConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            let param = {
                let mut tmp = HashMap::<String, String>::from_iter([
                    ("generator".to_string(), "backlinks".to_string()),
                    ("gbltitle".to_string(), self.title_codec.to_pretty(&title)),
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
            for await x in self.query_all(param) { yield x; }
        }
    }

    /// Fetch a page's embeds.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=embeddedin&geinamespace=<ns>&geilimit=max&geititle=<title>&geifilterredir=<filter>&redirects=<resolve>```
    /// 
    /// This function is called by `Embed` expression. A warning will be thrown if `titles` contains more than one page.
    fn get_embeds(&self, title: Title, config: &EmbedsConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            let param = {
                let mut tmp = HashMap::<String, String>::from_iter([
                    ("generator".to_string(), "embeddedin".to_string()),
                    ("geititle".to_string(), self.title_codec.to_pretty(&title)),
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
            for await x in self.query_all(param) { yield x; }
        }
    }

    /// Fetch a category's members.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=categorymembers&gcmtitle=<title>&gcmlimit=max&gcmnamespace=<ns>&gcmtype=<...>&redirects=<resolve>```
    /// 
    /// This function is called by `InCat` expression.
    fn get_category_members(&self, title: Title, config: &CategoryMembersConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
            let param = {
                let mut tmp = HashMap::<String, String>::from_iter([
                    ("generator".to_string(), "categorymembers".to_string()),
                    ("gcmtitle".to_string(), self.title_codec.to_pretty(&title)),
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
            for await x in self.query_all(param) { yield x; }
        }
    }

    /// Fetch a page's subpages.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=allpages&gapprefix=<title>&gaplimit=max&gapnamespace=<title>&gapfilterredir=<filter>```
    /// 
    /// This function is called by `Prefix` expression.
    /// A warning will be thrown if `titles` contains more than one page.
    /// This function ignores the `resolve` modifier.
    fn get_prefix(&self, title: Title, config: &PrefixConfig) -> impl Stream<Item=TrioResult<PageInfo, Self::Warn, Self::Error>> {
        stream! {
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
            for await x in self.query_all(param) { yield x; }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum APIDataProviderError {
    #[error(transparent)]
    Backend(#[from] ClientError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    TitleCodec(#[from] mwtitle::Error),
}
