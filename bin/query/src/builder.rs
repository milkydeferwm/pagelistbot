use mwapi::{Client, ErrorFormat, Assert};
use mwtitle::{SiteInfoResponse, TitleCodec};
use serde::Deserialize;
use crate::APIDataProvider;
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfoResponse {
    #[serde(default)]
    pub batchcomplete: bool,
    #[serde(rename = "continue")]
    #[serde(default)]
    pub continue_: HashMap<String, String>,
    pub query: UserInfoResponseBody,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfoResponseBody {
    pub userinfo: UserInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UserInfo {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub anon: bool,
    pub rights: HashSet<String>,
}

/// Creates an `APIDataProvider` instance.
pub async fn get_provider(site: &str, user: &str, password: &str) -> anyhow::Result<APIDataProvider> {
    // attempt to connect to website.
    let (api, title_codec, api_highlimit) = {
        let mut builder = Client::builder(site)
            .set_errorformat(ErrorFormat::default());
        if !user.is_empty() { // login with credential
            builder = builder
                .set_botpassword(user, password)
                .set_assert(Assert::User)
                .set_user_agent(&format!("Page List Bot v{} / User:{} / {}", env!("CARGO_PKG_VERSION"), user, env!("CARGO_PKG_REPOSITORY")));
        } else {
            builder = builder
                .set_assert(Assert::Anonymous)
                .set_user_agent(&format!("Page List Bot v{} / Anonymous user / {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_REPOSITORY")));
        }
        let api = builder.build().await?;
        // api_highlimit check
        let api_highlimit = api.post::<_, UserInfoResponse>(HashMap::from_iter([
            ("action", "query"),
            ("meta", "userinfo"),
            ("uiprop", "rights"),
        ])).await.map(|ui| ui.query.userinfo.rights.contains("apihighlimits"))?;
        // site info fetch
        let title_codec = {
            let site_info = api.post::<_, SiteInfoResponse>(HashMap::from_iter([
                ("action", "query"),
                ("meta", "siteinfo"),
                ("siprop", "general|namespaces|namespacealiases|interwikimap")
            ])).await.map(|si| si.query)?;
            TitleCodec::from_site_info(site_info)
        }?;
        (api, title_codec, api_highlimit)
    };
    // perform query.
    Ok(APIDataProvider::new(api, title_codec, api_highlimit))
}
