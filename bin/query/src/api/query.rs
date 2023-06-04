use core::convert::Infallible;
use futures::{stream::unfold, Stream};
use mwapi::Client;
use mwtitle::TitleCodec;
use provider::PageInfo;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use super::APIDataProviderError;
use trio_result::TrioResult;

// FIXME: Fix a bug in `ResponseBody`, then change back to auto derived version.
//#[query(
//    prop = "info",
//    inprop = "associatedpage|subjectid|talkid",
//)]
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Response {
    #[serde(default)]
    pub batchcomplete: bool,
    #[serde(default)]
    #[serde(rename = "continue")]
    pub continue_: ::std::collections::HashMap<String, String>,
    pub query: Option<ResponseBody>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ResponseBody {
    pub pages: Vec<ResponseItem>,
    #[serde(default)]
    pub normalized: Vec<mwapi_responses::normalize::Normalized>,
    #[serde(default)]
    pub redirects: Vec<mwapi_responses::normalize::Redirect>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ResponseItem {
    pub associatedpage: String,
    pub contentmodel: String,
    #[serde(default)]
    pub invalid: bool,
    pub invalidreason: Option<String>,
    pub lastrevid: Option<u64>,
    pub length: Option<u32>,
    #[serde(default)]
    pub missing: bool,
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub ns: i32,
    pub pageid: Option<u32>,
    pub pagelanguage: String,
    pub pagelanguagedir: String,
    pub pagelanguagehtmlcode: String,
    #[serde(default)]
    pub redirect: bool,
    pub subjectid: Option<u32>,
    pub talkid: Option<u32>,
    pub title: String,
    pub touched: Option<::mwapi_responses::timestamp::Timestamp>,
}

#[derive(Debug, Clone)]
struct QueryState {
    api: Client,
    title_codec: TitleCodec,
    param: HashMap<String, String>,
    continue_: Option<HashMap<String, String>>,
    cache: VecDeque<PageInfo>,
}

pub(super) type QueryStream = impl Stream<Item=TrioResult<PageInfo, Infallible, APIDataProviderError>> + Send;

pub(super) fn query_complete(
    api: Client,
    title_codec: TitleCodec,
    param: HashMap<String, String>
) -> QueryStream {
    // FIXME: Remove manual manipulation of param, once `mwapi_responses` is used again.
    let mut param = param;
    param.extend([
        ("inprop".to_string(), "associatedpage|subjectid|talkid".to_string()),
        ("action".to_string(), "query".to_string()),
        ("formatversion".to_string(), "2".to_string()),
        ("format".to_string(), "json".to_string()),
        ("prop".to_string(), "info".to_string()),
    ]);
    let start_state = QueryState {
        api,
        title_codec,
        param,
        continue_: None,
        cache: VecDeque::new(),
    };
    let st = unfold(start_state, |state: QueryState| async move {
        let QueryState { api, title_codec, param, mut continue_, mut cache } = state;
        // considering miser mode, we loop until
        // 1. we have cached results
        // 2. we receive empty continue params, meaning query ends
        loop {
            // we have cache, return
            if !cache.is_empty() {
                let item = cache.pop_front().unwrap();
                let new_state = QueryState {
                    api, title_codec, param, continue_, cache,
                };
                return Some((TrioResult::Ok(item), new_state));
            }
            // we have empty continue params, return
            if let Some(continue_) = continue_.as_ref() {
                if continue_.is_empty() {
                    return None;
                }
            }
            // the query shall continue
            // set up query params with continued information.
            let mut query_params = param.clone();
            if let Some(continue_) = continue_ {
                query_params.extend(continue_);
            }
            // try get response, if error then return the error and put the stream to dead state.
            // FIXME: change back to query_response once `mwapi_responses` is used again.
            let resp: Response = match api.get(query_params).await {
                Ok(resp) => resp,
                Err(e) => {
                    let dead_state = QueryState {
                        api, title_codec, param,
                        continue_: Some(HashMap::new()),
                        cache: VecDeque::new(),
                    };
                    return Some((TrioResult::Err(e.into()), dead_state));
                }
            };
            // register new continue param
            continue_ = Some(resp.continue_);
            // read response and extract page info.
            // in a rare miser-mode scenario, `query` field in the response may not exist.
            // in such case the loop is continued.
            if let Some(query) = resp.query {
                for page in query.pages {
                    // get information for subject page, if error then return the error and put the stream to dead state.
                    let thispage_title = {
                        let t = title_codec.new_title(&page.title);
                        if let Err(e) = t {
                            let dead_state = QueryState {
                                api, title_codec, param,
                                continue_: Some(HashMap::new()),
                                cache: VecDeque::new(),
                            };
                            return Some((TrioResult::Err(e.into()), dead_state));
                        }
                        Some(t.unwrap())
                    };
                    let thispage_exists = Some(!page.missing);
                    let thispage_redirect = Some(page.redirect);
    
                    // `associatedpage` is as follows:
                    // Normally: exists, point to the page title regardless of whether it exists.
                    // Topic namespace: exists, "Special:Badtitle/NS2601:XXXXXXXXX"
                    // Special namespaces: does not exist.
    
                    // get information for associated page, if error then return the error and put the stream to dead state.
                    let associated_title = {
                        // FIXME: use `unwrap_or` once this has become an `Option<String>`.
                        // let title = page.associatedpage.unwrap_or("Special:Badtitle");
                        let title = &page.associatedpage;
                        let t = title_codec.new_title(title);
                        if let Err(e) = t {
                            let dead_state = QueryState {
                                api, title_codec, param,
                                continue_: Some(HashMap::new()),
                                cache: VecDeque::new(),
                            };
                            return Some((TrioResult::Err(e.into()), dead_state));
                        }
                       Some(t.unwrap())
                    };
                    // If one of `subjectid` (of a talk page) or `talkid` (of a subject page) exists, then the associated page exists.
                    let associated_exists = Some(page.subjectid.is_some() || page.talkid.is_some());
                    // Unfortunately we cannot determine whether the associated page is a redirect or not.
    
                    let page_info = PageInfo::new(thispage_title, thispage_exists, thispage_redirect, associated_title, associated_exists, None);
                    cache.push_back(page_info);
                }
            }
            // upon next loop should return from cache or continue next query (if miser mode) or end.
        }
    });
    Box::pin(st)
}
