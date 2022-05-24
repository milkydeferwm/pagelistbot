//! Abstract syntax tree used for parsing and output

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub begin: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Page { titles: BTreeSet<String> },

    Intersection { set1: Box<Node>, set2: Box<Node> },
    Union { set1: Box<Node>, set2: Box<Node> },
    Difference { set1: Box<Node>, set2: Box<Node> },
    Xor { set1: Box<Node>, set2: Box<Node> },

    Link { target: Box<Node>, modifier: Modifier },
    BackLink { target: Box<Node>, modifier: Modifier },
    Embed { target: Box<Node>, modifier: Modifier },
    InCategory { target: Box<Node>, modifier: Modifier },
    Prefix { target: Box<Node>, modifier: Modifier },

    Toggle { target: Box<Node> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    // Applies to all operations
    pub result_limit: Option<i64>,
    pub resolve_redirects: bool,
    // Only available for certain operations
    pub namespace: Option<BTreeSet<i64>>,
    pub categorymembers_recursion_depth: i64,
    pub filter_redirects: RedirectFilterStrategy,
    pub backlink_trace_redirects: bool,
}

impl Modifier {
    pub fn new() -> Self {
        Self {
            result_limit: None,
            resolve_redirects: false,
            namespace: None,
            categorymembers_recursion_depth: 0,
            filter_redirects: RedirectFilterStrategy::All,
            backlink_trace_redirects: true,
        }
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectFilterStrategy {
    NoRedirect,
    OnlyRedirect,
    All,
}

impl core::fmt::Display for RedirectFilterStrategy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoRedirect => write!(f, "nonredirects"),
            Self::OnlyRedirect => write!(f, "redirects"),
            Self::All => write!(f, "all"),
        }
    }
}
