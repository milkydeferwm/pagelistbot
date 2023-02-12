use crate::{PageInfo, Pair};
use futures::{stream::try_unfold, TryStream};
use mwapi::Client;
use mwtitle::TitleCodec;
use std::collections::{HashMap, VecDeque};

#[mwapi_responses::query(
    prop = "info",
    inprop = "associatedpage|subjectid|talkid",
)]
struct Response;

#[derive(Debug, Clone)]
struct QueryState {
    api: Client,
    title_codec: TitleCodec,
    param: HashMap<String, String>,
    continue_: Option<HashMap<String, String>>,
    cache: VecDeque<Pair<PageInfo>>,
}

pub(super) type QueryStream = impl TryStream<Ok=Pair<PageInfo>, Error=mwapi::Error, Item = Result<Pair<PageInfo>, mwapi::Error>> + Send;

pub(super) fn query_complete(api: Client, title_codec: TitleCodec, param: HashMap<String, String>) -> QueryStream {
    let start_state = QueryState {
        api,
        title_codec,
        param,
        continue_: None,
        cache: VecDeque::new(),
    };
    try_unfold(start_state, |state: QueryState| async move {
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
                return Ok(Some((item, new_state)));
            }
            // we have empty continue params, return
            if let Some(continue_) = continue_.as_ref() {
                if continue_.is_empty() {
                    return Ok(None);
                }
            }
            // the query shall continue
            let mut query_params = param.clone();
            if let Some(continue_) = continue_ {
                query_params.extend(continue_);
            }
            let resp: Response = api.query_response(query_params).await?;
            continue_ = Some(resp.continue_);
            for page in resp.query.pages {
                let thispage_title = {
                    let t = title_codec.new_title(&page.title);
                    if let Err(e) = t {
                        return Err(e.into());
                    }
                    Some(t.unwrap())
                };
                let thispage_exists = Some(!page.missing);
                let thispage_redirect = Some(page.redirect);
                // `associatedpage` is as follows:
                // Normally: exists, point to the page title regardless of whether it exists.
                // Topic namespace: exists, "Special:Badtitle/NS2601:XXXXXXXXX"
                // Special namespaces: does not exist.
                let associated_title = {
                    // FIXME: use `unwrap_or` once this has become an `Option<String>`.
                    // let title = page.associatedpage.unwrap_or("Special:Badtitle");
                    let title = &page.associatedpage;
                    let t = title_codec.new_title(title);
                    if let Err(e) = t {
                        return Err(e.into());
                    }
                   Some(t.unwrap())
                };
                // If one of `subjectid` (of a talk page) or `talkid` (of a subject page) exists, then the associated page exists.
                let associated_exists = Some(page.subjectid.is_some() || page.talkid.is_some());
                // Unfortunately we cannot determine whether the associated page is a redirect or not.
    
                let pair = (
                    PageInfo::new(thispage_title, thispage_exists, thispage_redirect),
                    PageInfo::new(associated_title, associated_exists, None)
                );
               cache.push_back(pair);
            }
            // upon next loop should return from cache or continue next query (if miser mode) or end.
        }
    })
}
