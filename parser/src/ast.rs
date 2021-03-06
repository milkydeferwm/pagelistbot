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
    pub result_limit: Option<NumberOrInf<usize>>,
    pub resolve_redirects: bool,
    // Only available for certain operations
    pub namespace: Option<BTreeSet<i32>>,
    pub categorymembers_recursion_depth: NumberOrInf<usize>,
    pub filter_redirects: RedirectFilterStrategy,
    pub backlink_trace_redirects: bool,
}

impl Modifier {
    pub fn new() -> Self {
        Self {
            result_limit: None,
            resolve_redirects: false,
            namespace: None,
            categorymembers_recursion_depth: NumberOrInf::Finite(0),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberOrInf<T> {
    Finite(T),
    Infinity,
}

impl<T> PartialOrd for NumberOrInf<T>
where
    T: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match (self, other) {
            (Self::Infinity, Self::Infinity) => Some(core::cmp::Ordering::Equal),
            (Self::Infinity, _) => Some(core::cmp::Ordering::Greater),
            (_, Self::Infinity) => Some(core::cmp::Ordering::Less),
            (Self::Finite(a), Self::Finite(b)) => a.partial_cmp(b),
        }
    }
}

impl<T> Ord for NumberOrInf<T>
where
    T: Ord
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Infinity, Self::Infinity) => core::cmp::Ordering::Equal,
            (Self::Infinity, _) => core::cmp::Ordering::Greater,
            (_, Self::Infinity) => core::cmp::Ordering::Less,
            (Self::Finite(a), Self::Finite(b)) => a.cmp(b),
        }
    }
}

impl<T> core::fmt::Display for NumberOrInf<T>
where
    T: core::fmt::Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infinity => write!(f, "infinity"),
            Self::Finite(v) => v.fmt(f),
        }
    }
}
