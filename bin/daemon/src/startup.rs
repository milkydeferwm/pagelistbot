//! Startup type definition.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupConfig {
    pub login: HashMap<String, LoginCredential>,
    pub sites: HashMap<String, StartupSiteConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginCredential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupSiteConfig {
    pub login: String,
    #[serde(alias = "api")]
    pub api_endpoint: String,
    #[serde(alias = "bot")]
    #[serde(default)]
    pub prefer_bot_edit: bool,
    #[serde(alias = "onsite")]
    pub on_site_config: String,
    #[serde(alias = "db")]
    #[serde(default)]
    pub db_name: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_startup_parse() {
        let input = r#"
        {
            "login": {
                "example": {
                    "username": "Example",
                    "password": "Yipee~!"
                },
                "another": {
                    "username": "KawaiiKoneko",
                    "password": "nyan"
                }
            },
            "sites": {
                "enwiki": {
                    "login": "another",
                    "api": "https://en.wikipedia.org/w/api.php",
                    "on_site_config": "User:Example",
                    "db_name": "enwiki_p"
                },
                "meta": {
                    "login": "example",
                    "api_endpoint": "https://meta.wikimedia.org/w/api.php",
                    "prefer_bot_edit": true,
                    "onsite": "User:Example"
                }
            }
        }"#;
        let output = StartupConfig {
            login: HashMap::from_iter([
                ("another".to_owned(), LoginCredential {
                    username: "KawaiiKoneko".to_owned(),
                    password: "nyan".to_owned(),
                }),
                ("example".to_owned(), LoginCredential {
                    username: "Example".to_owned(),
                    password: "Yipee~!".to_owned(),
                }),
            ]),
            sites: HashMap::from_iter([
                ("enwiki".to_owned(), StartupSiteConfig {
                    login: "another".to_owned(),
                    api_endpoint: "https://en.wikipedia.org/w/api.php".to_owned(),
                    prefer_bot_edit: false,
                    on_site_config: "User:Example".to_owned(),
                    db_name: Some("enwiki_p".to_owned()),
                }),
                ("meta".to_owned(), StartupSiteConfig {
                    login: "example".to_owned(),
                    api_endpoint: "https://meta.wikimedia.org/w/api.php".to_owned(),
                    prefer_bot_edit: true,
                    on_site_config: "User:Example".to_owned(),
                    db_name: None,
                }),
            ]),
        };
        assert_eq!(serde_json::from_str::<StartupConfig>(input).unwrap(), output);
    }
}
