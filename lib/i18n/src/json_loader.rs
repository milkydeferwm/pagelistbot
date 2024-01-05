//! JSON Loader for `I18nProvider`.

#![cfg(feature = "json")]

use crate::I18nProvider;
use std::{collections::HashMap, io::Read};

#[derive(Debug, Clone, serde::Deserialize)]
struct I18nJsonFile {
    #[allow(dead_code)]
    #[serde(rename = "@metadata", default)]
    metadata: serde_json::Value,

    #[serde(flatten)]
    resource: HashMap<String, String>,
}

impl I18nProvider {
    pub fn try_load_json<R: Read>(reader: R) -> Result<Self, serde_json::Error> {
        let f: I18nJsonFile = serde_json::from_reader(reader)?;
        Ok(Self::from_resource(f.resource))
    }

    pub fn load_json<R: Read>(reader: R) -> Self {
        Self::try_load_json(reader).unwrap_or_else(|e| {
            tracing::warn!("cannot load i18n file: {}", e);
            Self::new()
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use crate::I18nProvider;

    #[test]
    fn test_json_loader() {
        let json_str = "{
            \"@metadata\": {
                \"authors\": [
                    \"alice\",
                    \"bob\"
                ]
            },
            \"foo\": \"bar\",
            \"foofoo\": \"barbar\"
        }";
        let reader = Cursor::new(json_str);
        let i18n = I18nProvider::try_load_json(reader).unwrap();
        assert_eq!(i18n.get_text("foo"), "bar");
        assert_eq!(i18n.get_text("foofoo"), "barbar");
    }

    #[test]
    fn test_json_loader_no_metadata() {
        let json_str = "{
            \"foo\": \"bar\",
            \"foofoo\": \"barbar\"
        }";
        let reader = Cursor::new(json_str);
        let i18n = I18nProvider::try_load_json(reader).unwrap();
        assert_eq!(i18n.get_text("foo"), "bar");
        assert_eq!(i18n.get_text("foofoo"), "barbar");
    }
}
