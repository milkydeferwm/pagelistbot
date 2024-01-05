//! Minimal i18n manager for Page List Bot.
//! The manager just reads a localisation file on the disk, and store it as a key-value pair. Just that.

use std::collections::HashMap;

mod json_loader;

#[derive(Debug, Clone, Default)]
pub struct I18nProvider {
    resource: HashMap<String, String>
}

impl I18nProvider {

    /// Retrive the localised text given the resource key. Returns `None` if such resource does not exist.
    #[inline]
    pub fn try_get_text(&self, key: &str) -> Option<String> {
        self.resource.get(key).cloned()
    }

    /// Retrive the localised text given the resource key. If such resource does not exist, `(key)` is returned.
    #[inline]
    pub fn get_text(&self, key: &str) -> String {
        self.try_get_text(key).unwrap_or_else(|| {
            tracing::warn!(key=key, "i18n resource not found");
            format!("({key})")
        })
    }

    /// Create a new, empty provider.
    pub fn new() -> Self {
        Self { resource: HashMap::new() }
    }

    pub fn from_resource(res: HashMap<String, String>) -> Self {
        Self {
            resource: res
        }
    }

}

#[cfg(test)]
mod test {
    use crate::I18nProvider;
    use std::collections::HashMap;

    #[test]
    fn test_get_text() {
        let mapping = HashMap::from_iter([
            ("foo".to_string(), "bar".to_string()),
        ]);
        let i18n = I18nProvider::from_resource(mapping);
        assert_eq!(i18n.get_text("foo"), "bar");
        assert_eq!(i18n.get_text("foofoo"), "(foofoo)");
    }
}
