//! Module related to MediaWiki login and information retrieval.

use crate::APIConnection;
use mwapi::{Client, Assert, ErrorFormat};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfoResponse {
    #[serde(default)]
    batchcomplete: bool,
    #[serde(rename = "continue")]
    #[serde(default)]
    continue_: HashMap<String, String>,
    query: UserInfoResponseBody,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfoResponseBody {
    userinfo: UserInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfo {
    id: u32,
    name: String,
    #[serde(default)]
    anon: bool,
    rights: HashSet<String>,
}

pub(crate) async fn get_provider(site: &str, user: &str, password: &str) -> Option<APIConnection> {
    // attempt to connect to website.
    let mut builder = Client::builder(site)
        .set_errorformat(ErrorFormat::default());
    if !user.is_empty() { // login with credential
        builder = builder
            .set_botpassword(user, password)
            .set_assert(Assert::User)
            .set_user_agent(&format!("Page List Bot version {} logged in as `User:{}`; report issues to `{}`", env!("CARGO_PKG_VERSION"), user, env!("CARGO_PKG_REPOSITORY")));
    } else {
        builder = builder
            .set_assert(Assert::Anonymous)
            .set_user_agent(&format!("Page List Bot version {} not logged in; report issues to `{}`", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_REPOSITORY")));
    }
    let api = match builder.build().await {
        Ok(x) => x,
        Err(e) => {
            tracing::warn!(warning=?e, site=site, user=user, "cannot log in");
            return None;
        },
    };
    // api_highlimit check
    let (bot, apihighlimits) = match api.post::<_, UserInfoResponse>(HashMap::from_iter([
        ("action", "query"),
        ("meta", "userinfo"),
        ("uiprop", "rights"),
    ])).await {
        Ok(ui) => {
            let rights = ui.query.userinfo.rights;
            (rights.contains("bot"), rights.contains("apihighlimits"))
        },
        Err(e) => {
            tracing::warn!(warning=?e, site=site, user=user, "cannot fetch user information");
            return None;
        },
    };
    // site info fetch
    let site_info = match api.post_value(HashMap::from_iter([
        ("action", "query"),
        ("meta", "siteinfo"),
        ("siprop", "general|namespaces|namespacealiases|interwikimap"),
    ])).await {
        Ok(si) => si,
        Err(e) => {
            tracing::warn!(warning=?e, site=site, user=user, "cannot fetch site information");
            return None;
        },
    };

    Some(APIConnection { client: api, site_info, bot, apihighlimits })
}
