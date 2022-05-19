#![cfg(feature="parse")]

use std::collections::HashSet;

use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModifierBuilder {
    // Applies to all operations
    result_limit: Option<i64>,
    resolve_redirects: bool,
    // Only available for certain operations
    namespace: Option<HashSet<i64>>,
    categorymembers_recursion_depth: Option<i64>,
    filter_redirects: Option<RedirectFilterStrategy>,
    backlink_trace_redirects: bool,
}

impl ModifierBuilder {
    pub(crate) fn new() -> Self {
        ModifierBuilder {
            result_limit: None,
            resolve_redirects: false,
            namespace: None,
            categorymembers_recursion_depth: None,
            filter_redirects: None,
            backlink_trace_redirects: true,
        }
    }

    pub(crate) fn build(self) -> Modifier {
        Modifier {
            result_limit: self.result_limit,
            resolve_redirects: self.resolve_redirects,
            namespace: self.namespace,
            categorymembers_recursion_depth: self.categorymembers_recursion_depth.unwrap_or(0),
            filter_redirects: self.filter_redirects.unwrap_or(RedirectFilterStrategy::All),
            backlink_trace_redirects: self.backlink_trace_redirects,
        }
    }

    pub(crate) fn result_limit(mut self, value: i64) -> Self {
        if let Some(rl) = self.result_limit {
            // If value < 0, whether or not rl is also < 0, keeping the original value intact is always semantically correct
            if value >= 0 {
                if rl < 0 {
                    self.result_limit = Some(value);
                } else {
                    self.result_limit = Some(i64::min(rl, value));
                }
            }
        } else {
            self.result_limit = Some(value);
        }
        self
    }

    pub(crate) fn resolve_redirects(mut self) -> Self {
        self.resolve_redirects = true;
        self
    }

    pub(crate) fn namespace(mut self, value: &HashSet<i64>) -> Self {
        if let Some(ns) = self.namespace {
            self.namespace = Some(ns.intersection(value).copied().collect())
        } else {
            self.namespace = Some(value.to_owned())
        }
        self
    }

    pub(crate) fn categorymembers_recursion_depth(mut self, value: i64) -> Self {
        if let Some(rd) = self.categorymembers_recursion_depth {
            // If value < 0, whether or not rl is also < 0, keeping the original value intact is always semantically correct
            if value >= 0 {
                if rd < 0 {
                    self.categorymembers_recursion_depth = Some(value);
                } else {
                    self.categorymembers_recursion_depth = Some(i64::min(rd, value));
                }
            }
        } else {
            self.categorymembers_recursion_depth = Some(value);
        }
        self
    }

    pub(crate) fn no_redirect(mut self) -> Self {
        self.filter_redirects = Some(RedirectFilterStrategy::NoRedirect);
        self
    }

    pub(crate) fn only_redirect(mut self) -> Self {
        self.filter_redirects = Some(RedirectFilterStrategy::OnlyRedirect);
        self
    }

    pub(crate) fn direct_backlink(mut self) -> Self {
        self.backlink_trace_redirects = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModifierType {
    ResultLimit(i64),
    ResolveRedirects,

    Namespace(HashSet<i64>),
    RecursionDepth(i64),
    NoRedirect,
    OnlyRedirect,
    DirectBacklink,
}
