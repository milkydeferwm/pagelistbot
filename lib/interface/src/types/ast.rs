//! Abstract syntax tree.

use std::{collections::BTreeSet, str::FromStr};
#[cfg(feature = "use_serde")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "use_serde")]
use serde_with::{SerializeDisplay, DeserializeFromStr};

/// A span object. `Span` contains full location details of the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct Span {
    pub begin_line: u32,
    pub begin_col: usize,
    pub begin_offset: usize,
    pub end_line: u32,
    pub end_col: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct Node {
    span: Span,
    expr: Expr,
}

impl Node {
    pub fn new(span: Span, expr: Expr) -> Self {
        Self { span, expr }
    }
    pub fn get_span(&self) -> Span {
        self.span
    }

    pub fn get_expr(&self) -> &Expr {
        &self.expr
    }

    pub fn get_titles(&self) -> &BTreeSet<String> {
        self.expr.get_titles()
    }

    pub fn get_child(&self) -> &Node {
        self.expr.get_child()
    }

    pub fn get_modifier(&self) -> &Modifier {
        self.expr.get_modifier()
    }

    pub fn get_child_left(&self) -> &Node {
        self.expr.get_child_left()
    }

    pub fn get_child_right(&self) -> &Node {
        self.expr.get_child_right()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
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

impl Expr {
    pub fn get_titles(&self) -> &BTreeSet<String> {
        match self {
            Expr::Page { titles } => titles,
            _ => panic!("not a `Page` node"),
        }
    }

    pub fn get_child(&self) -> &Node {
        match self {
            Expr::Link { target, .. } => target,
            Expr::BackLink { target, .. } => target,
            Expr::Embed { target, .. } => target,
            Expr::InCategory { target, .. } => target,
            Expr::Prefix { target, .. } => target,
            Expr::Toggle { target } => target,
            _ => panic!("node does not have a target"),
        }
    }

    pub fn get_modifier(&self) -> &Modifier {
        match self {
            Expr::Link { modifier, .. } => modifier,
            Expr::BackLink { modifier, .. } => modifier,
            Expr::Embed { modifier, .. } => modifier,
            Expr::InCategory { modifier, .. } => modifier,
            Expr::Prefix { modifier, .. } => modifier,
            _ => panic!("node does not have modifier"),
        }
    }

    pub fn get_child_left(&self) -> &Node {
        match self {
            Expr::Intersection { set1, .. } => set1,
            Expr::Union { set1, .. } => set1,
            Expr::Difference { set1, .. } => set1,
            Expr::Xor { set1, .. } => set1,
            _ => panic!("node does not have a left child"),
        }
    }

    pub fn get_child_right(&self) -> &Node {
        match self {
            Expr::Intersection { set2, .. } => set2,
            Expr::Union { set2, .. } => set2,
            Expr::Difference { set2, .. } => set2,
            Expr::Xor { set2, .. } => set2,
            _ => panic!("node does not have a right child"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "use_serde", derive(SerializeDisplay, DeserializeFromStr))]
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
            Self::Infinity => write!(f, "inf"),
            Self::Finite(v) => v.fmt(f),
        }
    }
}

impl<T> Default for NumberOrInf<T>
where
    T: Default
{
    fn default() -> Self {
        Self::Finite(T::default())
    }
}

impl<T> FromStr for NumberOrInf<T>
where
    T: FromStr
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inf" | "Inf" | "iNf" | "inF" | "INf" | "InF" | "iNF" | "INF" => Ok(Self::Infinity),
            s => T::from_str(s).map(Self::Finite),
        }
    }
}
