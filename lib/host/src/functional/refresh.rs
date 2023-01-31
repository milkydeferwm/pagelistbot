//! Page List Bot API refreshment execution methods.

use std::{collections, sync::Arc};
use crate::InnerHostConfig;

// This type is exposed to crate root, because `try_new` method in `host_impls.rs` requires them.
/// The response struct when making a `userinfo` API call.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct UserInfoResponse {
    #[serde(default)]
    pub batchcomplete: bool,
    #[serde(rename = "continue")]
    #[serde(default)]
    pub continue_: collections::HashMap<String, String>,
    pub query: UserInfoResponseBody,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct UserInfoResponseBody {
    pub userinfo: UserInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct UserInfo {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub anon: bool,
    pub rights: collections::HashSet<String>,
}

pub(crate) struct RefresherExec<'exec> {
    pub host_config: Arc<InnerHostConfig>,
    pub api: &'exec mut mwapi::Client,
    pub siteinfo: &'exec mut mwtitle::SiteInfo,
    pub has_bot_flag: &'exec mut bool,
    pub has_apihighlimits_flag: &'exec mut bool,
}

impl<'exec> RefresherExec<'exec> {
    /// Make an API call and get site info.
    async fn get_siteinfo(&self) -> Result<mwtitle::SiteInfo, mwapi::Error> {
        let params = collections::HashMap::from_iter([
            ("action", "query"),
            ("meta", "siteinfo"),
            ("siprop", "general|namespaces|namespacealiases|interwikimap")
        ]);
        self.api.get::<_, mwtitle::SiteInfoResponse>(params)
            .await
            .map(|e| e.query)
    }
    /// Make an API call and get user info.
    async fn get_userinfo(&self) -> Result<UserInfo, mwapi::Error> {
        let params = collections::HashMap::from_iter([
            ("action", "query"),
            ("meta", "userinfo"),
            ("uiprop", "rights"),
        ]);
        self.api.get::<_, UserInfoResponse>(params)
            .await
            .map(|e| e.query.userinfo)
    }
    pub async fn execute(&mut self) -> RefresherSummary {
        // try get site info and user info.
        let siteinfo = self.get_siteinfo().await;
        let userinfo = self.get_userinfo().await;
        if siteinfo.is_ok() && userinfo.is_ok() {
            // validate successful, update and exit.
            *(self.siteinfo) = siteinfo.unwrap();
            *(self.has_bot_flag) = userinfo.unwrap().rights.contains("bot");
            RefresherSummary::Validated
        } else {
            // try refresh API
            let api = mwapi::Client::builder(&self.host_config.api_endpoint)
                .set_botpassword(&self.host_config.username, &self.host_config.password)
                .set_errorformat(mwapi::ErrorFormat::default())
                .set_assert(mwapi::Assert::User) // make sure we are signed in. no need to check bot flag.
                .set_user_agent(&format!("Page List Bot v{} / User:{} / {}", env!("CARGO_PKG_VERSION"), &self.host_config.username, env!("CARGO_PKG_REPOSITORY")))
                .build()
                .await;
            if let Err(e) = api {
                return RefresherSummary::NewClientFailed(e);
            }
            let api = api.unwrap();
            *(self.api) = api;
            let siteinfo = self.get_siteinfo().await;
            if let Err(e) = siteinfo {
                return RefresherSummary::NewSiteInfoFailed(e);
            }
            let siteinfo = siteinfo.unwrap();
            *(self.siteinfo) = siteinfo;
            let userinfo = self.get_userinfo().await;
            if let Err(e) = userinfo {
                return RefresherSummary::NewUserInfoFailed(e);
            }
            let userinfo = userinfo.unwrap();
            *(self.has_bot_flag) = userinfo.rights.contains("bot");
            *(self.has_apihighlimits_flag) = userinfo.rights.contains("apihighlimits");
            RefresherSummary::Refreshed
        }
    }
}

#[derive(Debug, Clone)]
pub enum RefresherSummary {
    /// The current API client is validated.
    Validated,
    /// The current API client is refreshed.
    Refreshed,
    /// Failed to build a new API client.
    NewClientFailed(mwapi::Error),
    /// Failed to grab new site info.
    NewSiteInfoFailed(mwapi::Error),
    /// Failed to grab new user info.
    NewUserInfoFailed(mwapi::Error),
}

impl From<RefresherSummary> for interface::types::status::refresher::PageListBotRefresherSummary {
    fn from(value: RefresherSummary) -> Self {
        match value {
            RefresherSummary::Validated => Self::Validated,
            RefresherSummary::Refreshed => Self::Refreshed,
            RefresherSummary::NewClientFailed(e) => Self::NewClientFailed(e.to_string()),
            RefresherSummary::NewSiteInfoFailed(e) => Self::NewSiteInfoFailed(e.to_string()),
            RefresherSummary::NewUserInfoFailed(e) => Self::NewUserInfoFailed(e.to_string()),
        }
    }
}
