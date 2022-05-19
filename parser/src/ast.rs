//! Abstract syntax tree used for parsing and output

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub line: u32,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Page { span: Span, titles: Vec<String> },

    Intersection { span: Span, set1: Box<Expr>, set2: Box<Expr> },
    Union { span: Span, set1: Box<Expr>, set2: Box<Expr> },
    Difference { span: Span, set1: Box<Expr>, set2: Box<Expr> },
    Xor { span: Span, set1: Box<Expr>, set2: Box<Expr> },

    Link { span: Span, target: Box<Expr>, modifier: Modifier },
    BackLink { span: Span, target: Box<Expr>, modifier: Modifier },
    Embed {span: Span, target: Box<Expr>, modifier: Modifier },
    InCategory { span: Span, target: Box<Expr>, modifier: Modifier },
    Prefix { span: Span, target: Box<Expr>, modifier: Modifier },

    Toggle { span: Span, target: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    // Applies to all operations
    pub result_limit: Option<i64>,
    pub resolve_redirects: bool,
    // Only available for certain operations
    pub namespace: Option<std::collections::HashSet<i64>>,
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
