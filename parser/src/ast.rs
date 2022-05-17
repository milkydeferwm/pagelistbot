//! Abstract syntax tree used for parsing and output

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span<'a> {
    pub offset: usize,
    pub line: u32,
    pub column: usize,
    pub fragment: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a> {
    Page { span: Span<'a>, titles: Vec<&'a str> },

    Intersection { span: Span<'a>, set1: Box<Expr<'a>>, set2: Box<Expr<'a>> },
    Union { span: Span<'a>, set1: Box<Expr<'a>>, set2: Box<Expr<'a>> },
    Difference { span: Span<'a>, set1: Box<Expr<'a>>, set2: Box<Expr<'a>> },
    Xor { span: Span<'a>, set1: Box<Expr<'a>>, set2: Box<Expr<'a>> },

    Link { span: Span<'a>, target: Box<Expr<'a>>, modifier: Modifier },
    BackLink { span: Span<'a>, target: Box<Expr<'a>>, modifier: Modifier },
    Embed {span: Span<'a>, target: Box<Expr<'a>>, modifier: Modifier },
    InCategory { span: Span<'a>, target: Box<Expr<'a>>, modifier: Modifier },
    Prefix { span: Span<'a>, target: Box<Expr<'a>>, modifier: Modifier },

    Toggle { span: Span<'a>, target: Box<Expr<'a>> },
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
