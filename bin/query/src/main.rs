//! Page list bot query execution core.

#![allow(incomplete_features)]
#![feature(is_some_and)]
#![feature(type_alias_impl_trait)]

mod api;
use api::APIDataProvider;

use ast::Expression;
use clap::Parser;
use intorinf::IntOrInf;
use mwapi::{Assert, ErrorFormat, Client};
use mwtitle::{SiteInfoResponse, TitleCodec};
use nom::error::VerboseError;
use serde::Deserialize;
use solver::{Solver, tree::TreeSolver};
use std::{
    collections::{HashMap, HashSet},
    process::ExitCode,
};
use tokio::time::{Duration, timeout};

#[derive(Debug, Parser)]
pub struct Arg {
    /// The user name used for query. Leave it blank if you want to execute query anonymously
    #[arg(short, long, default_value_t = String::new())]
    user: String,
    /// The password of the corresponding user. Ignored if `user` is blank.
    #[arg(short, long, default_value_t = String::new())]
    password: String,
    /// The API endpoint. This is the URL of `api.php`.
    #[arg(short, long)]
    site: String,
    /// The query string.
    #[arg(short, long)]
    query: String,
    /// Maximum time allowed for query, in seconds.
    #[arg(short, long, default_value_t = 120)]
    timeout: u64,
    /// Default maximum query result limit, if it is not overridden by `.limit()` expression modifier.
    #[arg(short, long, default_value_t = 10000)]
    limit: i32,
}

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

const FAILURE_PARSE: u8 = 100;
const FAILURE_INIT: u8 = 101;
const FAILURE_QUERY: u8 = 102;
const FAILURE_TIMEOUT: u8 = 103;

#[tokio::main]
async fn main() -> ExitCode {
    let arg = Arg::parse();
    // parse the expression first. only continue if parse successful.
    let expr = match Expression::parse::<VerboseError<_>>(&arg.query) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(FAILURE_PARSE);
        }
    };
    // attempt to connect to website.
    let (api, title_codec, api_highlimit) = {
        let mut builder = Client::builder(&arg.site)
            .set_errorformat(ErrorFormat::default());
        if !arg.user.is_empty() { // login with credential
            builder = builder
                .set_botpassword(&arg.user, &arg.password)
                .set_assert(Assert::User)
                .set_user_agent(&format!("Page List Bot v{} / User:{} / {}", env!("CARGO_PKG_VERSION"), &arg.user, env!("CARGO_PKG_REPOSITORY")));
        } else {
            builder = builder
                .set_assert(Assert::Anonymous)
                .set_user_agent(&format!("Page List Bot v{} / Anonymous user / {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_REPOSITORY")));
        }
        let api = match builder.build().await {
            Ok(api) => api,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(FAILURE_INIT);
            },
        };
        // api_highlimit check
        let api_highlimit = match api.post::<_, UserInfoResponse>(HashMap::from_iter([
            ("action", "query"),
            ("meta", "userinfo"),
            ("uiprop", "rights"),
        ])).await {
            Ok(ui) => ui.query.userinfo.rights.contains("apihighlimits"),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(FAILURE_INIT);
            },
        };
        // site info fetch
        let title_codec = {
            let site_info = match api.post::<_, SiteInfoResponse>(HashMap::from_iter([
                ("action", "query"),
                ("meta", "siteinfo"),
                ("siprop", "general|namespaces|namespacealiases|interwikimap")
            ])).await {
                Ok(si) => si.query,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(FAILURE_INIT);
                },
            };
            match TitleCodec::from_site_info(site_info) {
                Ok(tc) => tc,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(FAILURE_INIT);
                },
            }
        };
        (api, title_codec, api_highlimit)
    };
    // perform query.
    let provider = APIDataProvider::new(&api, &title_codec, api_highlimit);
    let solver = TreeSolver::new(provider, IntOrInf::from(arg.limit));
    let query_result = match timeout(Duration::from_secs(arg.timeout), solver.solve(&expr)).await {
        Ok(res) => res,
        Err(_) => {
            eprintln!("Timeout after {} seconds", arg.timeout);
            return ExitCode::from(FAILURE_TIMEOUT);
        },
    };
    match query_result {
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(FAILURE_QUERY)
        },
        Ok(ans) => {
            for warn in ans.warnings {
                eprintln!("{warn}");
            }
            for item in ans.titles {
                println!("{}", title_codec.to_pretty(&item));
            }
            ExitCode::SUCCESS
        }
    }
}
