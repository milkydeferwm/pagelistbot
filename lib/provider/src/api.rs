//! Data Provider using MediaWiki Action API

use std::collections::{HashMap, BTreeSet};

use mwtitle::Title;
use parser::ast::{Modifier, NumberOrInf};
use crate::{PagePair, DataProvider, PageInfo};
use std::error::Error;
use mwapi_responses::prelude::*;

#[query(
    prop = "info",
    inprop = "associatedpage|subjectid|talkid",
)]
struct Response;

#[derive(Debug, Clone, Copy)]
pub struct APIDataProvider<'a> {
    api: &'a mwapi::Client,
    title_codec: &'a mwtitle::TitleCodec,
}

impl<'a> APIDataProvider<'a> {

    pub fn new(api: &'a mwapi::Client, title_codec: &'a mwtitle::TitleCodec) -> Self {
        APIDataProvider {
            api,
            title_codec
        }
    }

    /// Helper function to collect API query results.
    /// `limit` must be `Some(_)`.
    /// 
    /// Returns a `Result<T, E>`, where
    /// * `T` contains two parts: `set` is the result set, `warn` is the warnings during the query.
    /// * `E` is the unrecoverable error encountered during the query.
    #[inline]
    async fn api_with_limit(&self, params: &[(String, String)], limit: Option<NumberOrInf<usize>>) -> Result<(BTreeSet<PagePair>, Vec<APIDataProviderError>), APIDataProviderError> {
        if limit.is_none() {
            return Err(APIDataProviderError::LimitUnknown);
        }
        let limit = limit.unwrap();
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<APIDataProviderError> = Vec::new();
        let mut continue_param = HashMap::new();
        loop {
            let mut params = params.to_vec();
            // insert continue params
            params.extend(continue_param.into_iter());
            let query_resp: Response = self.api.post(params).await?;
            // continue
            continue_param = query_resp.continue_.clone();
            // data
            for page in query_resp.query.pages {
                let thispage_title = self.title_codec.new_title(&page.title)?;
                let thispage_exists = !page.missing;
                let thispage_redirect = Some(page.redirect);
                // `associatedpage` is as follows:
                // Normally: exists, point to the page title regardless of whether it exists.
                // Topic namespace: exists, "Special:Badtitle/NS2601:XXXXXXXXX"
                // Special namespaces: does not exist.
                let associated_title = {
                    // FIXME: use `unwrap_or` once this has become an `Option<String>`.
                    // let title = page.associatedpage.unwrap_or("Special:Badtitle");
                    let title = &page.associatedpage;
                    self.title_codec.new_title(title)?
                };
                // If one of `subjectid` (of a talk page) or `talkid` (of a subject page) exists, then the associated page exists.
                let associated_exists = page.subjectid.is_some() || page.talkid.is_some();
                // Unfortunately we cannot determine whether the associated page is a redirect or not.

                let pair: PagePair = (
                    PageInfo {
                        title: thispage_title,
                        exists: thispage_exists,
                        redirect: thispage_redirect,
                    },
                    PageInfo {
                        title: associated_title,
                        exists: associated_exists,
                        redirect: None,
                    }
                );
                set.insert(pair);
            }
            // check limit
            if let NumberOrInf::Finite(lim) = &limit {
                if set.len() > (*lim) {
                    // clip
                    set = BTreeSet::from_iter(set.into_iter().take(*lim));
                    // warn
                    warn.push(APIDataProviderError::LimitExceededAPI(limit));
                    return Ok((set, warn));
                }
            }
            // or check finished
            if continue_param.is_empty() {
                return Ok((set, warn));
            }
        }
    }

}

/// Implementations for the `DataProvider` trait.
/// 
/// Each function returns a `PagePair` object that contains
/// * For this page
///   * Page title (name, namespace)
///   * Is the page missing?
///   * Is the page a redirect?
///   * Its associated page
/// * For associated page
///   * Page title (name, namespace)
///   * Is the page missing?
///   * Its associated page (the original page itself, obviously)
#[async_trait::async_trait]
impl<'a> DataProvider for APIDataProvider<'a> {
    type Error = APIDataProviderError;

    /// Fetch a set of pages' basic information.
    /// This function essentially calls 
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&titles=<titles>```
    /// 
    /// This function is called by `Page` expression. It is assumed that nobody would **hand-write** thousands of page names in a query.
    /// 
    /// This function is not intended to be called during some intermediate step, because at that time there would already be thousands of pages to be queried.
    async fn get_page_info(&self, titles: &BTreeSet<String>) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        let mut normalized_titles: BTreeSet<Title> = BTreeSet::new();
        for raw_title in titles {
            let t = self.title_codec.new_title(raw_title)?;
            normalized_titles.insert(t);
        }
        let normalized_titles: Vec<String> = normalized_titles.into_iter().map(|t| self.title_codec.to_pretty(&t)).collect();

        // Query in chunks of 50. Although we can query in chunks of 500 if we have `apihighlimits`, we don't really know that.
        let chunks = normalized_titles.chunks(50);
        for chunk in chunks {
            let (this_set, this_warn) = self.api_with_limit(&[
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("titles".to_string(), chunk.join("|"))
            ], Some(NumberOrInf::Infinity)).await?;
            // merge into total response
            set.extend(this_set);
            warn.extend(this_warn);
        }

        Ok((set, warn))
    }

    /// Fetch a page's links on that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=links&gplnamespace=<ns>&gpllimit=max&redirects=<resolve>&titles=<titles>```
    /// 
    /// This function is called by `Link` expression. A warning will be thrown if `titles` contains more than one page.
    async fn get_links(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        const QUERY_PAGE_LIMIT: usize = 1;

        if !titles.is_empty() {
            let title: Vec<String> = titles.iter().take(QUERY_PAGE_LIMIT).map(|t| self.title_codec.to_pretty(t)).collect();
            if titles.len() > QUERY_PAGE_LIMIT {
                warn.push(APIDataProviderError::TooManyPages(QUERY_PAGE_LIMIT, titles.len(), title.clone()));
            }

            let mut params = vec![
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("generator".to_string(), "links".to_string()),
                ("gpllimit".to_string(), "max".to_string()),
            ];
            if let Some(ns) = &modifier.namespace {
                if ns.is_empty() {
                    // early return no query
                    return Ok((set, warn));
                } else {
                    params.push(("gplnamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|")));
                }
            }
            if modifier.resolve_redirects {
                params.push(("redirects".to_string(), "1".to_string()));
            }

            // Iterate in chunks of 50, this is an API call that allows multiple titles, although we still only allow to query for one page
            let chunks = title.chunks(50);
            for chunk in chunks {
                params = params.clone();
                params.push(("titles".to_string(), chunk.join("|")));
                let (this_set, this_warn) = self.api_with_limit(&params, modifier.result_limit).await?;
                // merge into total response
                set.extend(this_set);
                warn.extend(this_warn);
                if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                    if set.len() > (*lim) {
                        warn.push(APIDataProviderError::LimitExceeded(modifier.result_limit.unwrap()));
                        set = BTreeSet::from_iter(set.into_iter().take(*lim));
                        break;
                    }
                }
            }
        }

        Ok((set, warn))
    }

    /// Fetch a page's backlinks to that page.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=backlinks&gblnamespace=<ns>&gbllimit=max&gbltitle=<title>&gblfilterredir=<filter>&gblredirect=<direct>&redirects=<resolve>```
    /// 
    /// This function is called by `LinkTo` expression. A warning will be thrown if `titles` contains more than one page.
    async fn get_backlinks(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        const QUERY_PAGE_LIMIT: usize = 1;

        if !titles.is_empty() {
            let title: Vec<String> = titles.iter().take(QUERY_PAGE_LIMIT).map(|t| self.title_codec.to_pretty(t)).collect();
            if titles.len() > QUERY_PAGE_LIMIT {
                warn.push(APIDataProviderError::TooManyPages(QUERY_PAGE_LIMIT, titles.len(), title.clone()));
            }

            let mut params = vec![
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("generator".to_string(), "backlinks".to_string()),
                ("gbllimit".to_string(), "max".to_string()),
                ("gblfilterredir".to_string(), modifier.filter_redirects.to_string()),
            ];
            if let Some(ns) = &modifier.namespace {
                if ns.is_empty() {
                    // early return no query
                    return Ok((set, warn));
                } else {
                    params.push(("gblnamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|")));
                }
            }
            if modifier.backlink_trace_redirects {
                params.push(("gblredirect".to_string(), "1".to_string()));
            }
            if modifier.resolve_redirects {
                params.push(("redirects".to_string(), "1".to_string()));
            }

            for title in title {
                params = params.clone();
                params.push(("gbltitle".to_string(), title));
                let (this_set, this_warn) = self.api_with_limit(&params, modifier.result_limit).await?;
                // merge into total response
                set.extend(this_set);
                warn.extend(this_warn);
                if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                    if set.len() > (*lim) {
                        warn.push(APIDataProviderError::LimitExceeded(modifier.result_limit.unwrap()));
                        set = BTreeSet::from_iter(set.into_iter().take(*lim));
                        break;
                    }
                }
            }
        }

        Ok((set, warn))
    }

    /// Fetch a page's embeds.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=embeddedin&geinamespace=<ns>&geilimit=max&geititle=<title>&geifilterredir=<filter>&redirects=<resolve>```
    /// 
    /// This function is called by `Embed` expression. A warning will be thrown if `titles` contains more than one page.
    async fn get_embeds(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        const QUERY_PAGE_LIMIT: usize = 1;

        if !titles.is_empty() {
            let title: Vec<String> = titles.iter().take(QUERY_PAGE_LIMIT).map(|t| self.title_codec.to_pretty(t)).collect();
            if titles.len() > QUERY_PAGE_LIMIT {
                warn.push(APIDataProviderError::TooManyPages(QUERY_PAGE_LIMIT, titles.len(), title.clone()));
            }

            let mut params = vec![
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("generator".to_string(), "embeddedin".to_string()),
                ("geilimit".to_string(), "max".to_string()),
                ("geifilterredir".to_string(), modifier.filter_redirects.to_string()),
            ];
            if let Some(ns) = &modifier.namespace {
                if ns.is_empty() {
                    // early return no query
                    return Ok((set, warn));
                } else {
                    params.push(("geinamespace".to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|")));
                }
            }
            if modifier.resolve_redirects {
                params.push(("redirects".to_string(), "1".to_string()));
            }

            for title in title {
                params = params.clone();
                params.push(("geititle".to_string(), title));
                let (this_set, this_warn) = self.api_with_limit(&params, modifier.result_limit).await?;
                // merge into total response
                set.extend(this_set);
                warn.extend(this_warn);
                if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                    if set.len() > (*lim) {
                        warn.push(APIDataProviderError::LimitExceeded(modifier.result_limit.unwrap()));
                        set = BTreeSet::from_iter(set.into_iter().take(*lim));
                        break;
                    }
                }
            }
        }

        Ok((set, warn))
    }

    /// Fetch a category's members.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=categorymembers&gcmtitle=<title>&gcmlimit=max&gcmnamespace=<ns>&gcmtype=<...>&redirects=<resolve>```
    /// 
    /// This function is called by `InCat` expression.
    async fn get_category_members(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        use std::collections::VecDeque;

        let mut ns_clone = modifier.namespace.clone();
        let mut result_has_ns_category = true;
        let mut result_has_ns_file = true;
        if let Some(ns) = ns_clone.as_mut() {
            result_has_ns_category = ns.remove(&14); // NS_CATEGORY = 14
            result_has_ns_file = ns.remove(&6); // NS_FILE = 6
        }

        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        const QUERY_PAGE_LIMIT: usize = 1;

        if !titles.is_empty() {
            // Filter out category pages
            let title: Vec<String> = titles.iter().filter(|t| t.namespace() == 14).take(QUERY_PAGE_LIMIT).map(|t| self.title_codec.to_pretty(t)).collect();
            if titles.len() > QUERY_PAGE_LIMIT {
                warn.push(APIDataProviderError::TooManyPages(QUERY_PAGE_LIMIT, titles.len(), title.clone()));
            }

            let mut params = vec![
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("generator".to_string(), "categorymembers".to_string()),
                ("gcmlimit".to_string(), "max".to_string()),
            ];
            if modifier.resolve_redirects {
                params.push(("redirects".to_string(), "1".to_string()));
            }

            for title in &title {
                // bfs search
                let mut visited: BTreeSet<String> = BTreeSet::from_iter([title.to_owned()]);
                let mut queue: VecDeque<(String, usize)> = VecDeque::from_iter([(title.to_owned(), 0)]);
                while let Some((this_cat, this_depth)) = queue.pop_front() {
                    // fill out remaining params
                    let mut params = params.clone();
                    params.push(("gcmtitle".to_string(), this_cat));
                    // determine what cmtype and cmnamespace should we insert
                    let mut cmtype: Vec<String> = Vec::new();
                    let mut cmnamespace: BTreeSet<i32> = BTreeSet::new();
                    // init cmnamespace with ns_clone
                    if let Some(ns) = &ns_clone {
                        if !ns.is_empty() {
                            cmtype.push("page".to_string());
                            cmnamespace.extend(ns);
                        }
                    } else {
                        // no namespace constraint at all, include every page
                        cmtype.push("page".to_string());
                    }
                    if result_has_ns_file {
                        cmtype.push("file".to_string());
                        cmnamespace.insert(6);
                    }
                    if result_has_ns_category || NumberOrInf::Finite(this_depth) < modifier.categorymembers_recursion_depth {
                        cmtype.push("subcat".to_string());
                        cmnamespace.insert(14);
                    }
                    if ns_clone.is_some() {
                        params.push(("gcmnamespace".to_string(), cmnamespace.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("|")));
                    }
                    params.push(("gcmtype".to_string(), cmtype.join("|")));

                    let (mut this_set, this_warn) = self.api_with_limit(&params, modifier.result_limit).await?;

                    // filter out subcategories and add to queue
                    if NumberOrInf::Finite(this_depth) < modifier.categorymembers_recursion_depth {
                        for pp in this_set.iter().filter(|&t| t.0.title.namespace() == 14) {
                            let subc = self.title_codec.to_pretty(&pp.0.title);
                            if !visited.contains(&subc) {
                                visited.insert(subc.clone());
                                queue.push_back((subc.clone(), this_depth + 1));
                            }
                        }
                    }
                    // merge into total response
                    if !result_has_ns_category {
                        this_set.retain(|f| f.0.title.namespace() != 14);
                    }
                    set.extend(this_set);
                    warn.extend(this_warn);
                    if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                        if set.len() > (*lim) {
                            break;
                        }
                    }
                }
                // check for limit
                if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                    if set.len() > (*lim) {
                        // unwrap is safe: `None` is catched in `self::api_with_limit` and error is already returned
                        warn.push(APIDataProviderError::LimitExceeded(modifier.result_limit.unwrap()));
                        set = BTreeSet::from_iter(set.into_iter().take(*lim));
                        break;
                    }
                }
            }
        }

        Ok((set, warn))
    }

    /// Fetch a page's subpages.
    /// This function essentially calls
    /// ```action=query&prop=info&inprop=associatedpage|subjectid|talkid&generator=allpages&gapprefix=<title>&gaplimit=max&gapnamespace=<title>&gapfilterredir=<filter>```
    /// 
    /// This function is called by `Prefix` expression.
    /// A warning will be thrown if `titles` contains more than one page.
    /// This function ignores the `resolve` modifier.
    async fn get_prefix(&self, titles: &BTreeSet<Title>, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Self::Error>), Self::Error> {
        let mut set: BTreeSet<PagePair> = BTreeSet::new();
        let mut warn: Vec<Self::Error> = Vec::new();

        const QUERY_PAGE_LIMIT: usize = 1;

        if !titles.is_empty() {
            let work_titles: Vec<&Title> = titles.iter().take(QUERY_PAGE_LIMIT).collect();
            if titles.len() > QUERY_PAGE_LIMIT {
                warn.push(APIDataProviderError::TooManyPages(QUERY_PAGE_LIMIT, titles.len(), work_titles.iter().map(|t| self.title_codec.to_pretty(t)).collect::<Vec<String>>()));
            }

            let mut params = vec![
                ("action".to_string(), "query".to_string()),
                ("prop".to_string(), "info".to_string()),
                ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
                ("generator".to_string(), "allpages".to_string()),
                ("gaplimit".to_string(), "max".to_string()),
                ("gapfilterredir".to_string(), modifier.filter_redirects.to_string()),
            ];

            for title in work_titles {
                params = params.clone();
                if let Some(ns) = &modifier.namespace {
                    if !ns.contains(&title.namespace()) {
                        // early continue no query
                        continue;
                    } else {
                        params.push(("gapnamespace".to_string(), title.namespace().to_string()));
                    }
                }
                params.push(("gaptitle".to_string(), title.dbkey().to_string()));
                let (this_set, this_warn) = self.api_with_limit(&params, modifier.result_limit).await?;
                // merge into total response
                set.extend(this_set);
                warn.extend(this_warn);
                if let NumberOrInf::Finite(lim) = &modifier.result_limit.unwrap() {
                    if set.len() > (*lim) {
                        warn.push(APIDataProviderError::LimitExceeded(modifier.result_limit.unwrap()));
                        set = BTreeSet::from_iter(set.into_iter().take(*lim));
                        break;
                    }
                }
            }
        }

        Ok((set, warn))
    }

}

#[non_exhaustive]
#[derive(Debug)]
pub enum APIDataProviderError {
    APIFailure(mwapi_errors::Error),
    LimitUnknown,
    LimitExceeded(NumberOrInf<usize>),
    LimitExceededAPI(NumberOrInf<usize>),
    TooManyPages(usize, usize, Vec<String>),
}
impl Error for APIDataProviderError {}
unsafe impl Sync for APIDataProviderError {}

impl core::fmt::Display for APIDataProviderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::APIFailure(a) => write!(f, "API query failed: {}", a),
            Self::LimitUnknown => write!(f, "query limit unknown, please file a bug report"),
            Self::LimitExceeded(lim) => write!(f, "query limit `{}` exceeded, result might be incomplete", lim),
            Self::LimitExceededAPI(lim) => write!(f, "query limit `{}` exceeded when calling API, result might be incomplete", lim),
            Self::TooManyPages(limit, actual, used) => {
                write!(f, "cannot query more than {} pages (got {}), query only conducted on \"{}\"", limit, actual, used.join("\", \""))
            }
        }
    }
}

impl From<mwapi_errors::Error> for APIDataProviderError {
    fn from(e: mwapi_errors::Error) -> Self {
        Self::APIFailure(e)
    }
}

impl From<mwtitle::Error> for APIDataProviderError {
    fn from(e: mwtitle::Error) -> Self {
        Self::APIFailure(e.into())
    }
}
