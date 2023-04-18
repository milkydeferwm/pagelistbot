//! Modifier expressions.

use alloc::vec::Vec;
use crate::{Span, expose_span};
use crate::literal::{LitIntOrInf, LitInt};
use crate::token::{
    LeftParen, RightParen, Comma,
    Limit, Resolve, Ns, Depth, NoRedir, OnlyRedir, Direct,
};

#[cfg(feature = "parse")]
pub mod parse;

/// Mega container for all modifiers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Modifier {
    Limit(ModifierLimit),
    Resolve(ModifierResolve),
    Ns(ModifierNs),
    Depth(ModifierDepth),
    NoRedir(ModifierNoRedir),
    OnlyRedir(ModifierOnlyRedir),
    Direct(ModifierDirect),
}

impl Modifier {
    pub fn get_span(&self) -> &Span {
        match self {
            Self::Limit(x) => x.get_span(),
            Self::Resolve(x) => x.get_span(),
            Self::Ns(x) => x.get_span(),
            Self::Depth(x) => x.get_span(),
            Self::NoRedir(x) => x.get_span(),
            Self::OnlyRedir(x) => x.get_span(),
            Self::Direct(x) => x.get_span(),
        }
    }
}

/// Modifier expression that limit the query count.
/// `limit(xx)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierLimit {
    span: Span,
    pub limit: Limit,
    pub lparen: LeftParen,
    pub val: LitIntOrInf,
    pub rparen: RightParen,
}

/// Modifier expression that defines whether to resolve redirects.
/// `resolve` or `resolve()`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierResolve {
    span: Span,
    pub resolve: Resolve,
    pub lparen: Option<LeftParen>,
    pub rparen: Option<RightParen>,
}

/// Modifier expression that contrains the results inside certain namespaces.
/// `ns(xx,xx)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierNs {
    span: Span,
    pub ns: Ns,
    pub lparen: LeftParen,
    pub vals: Vec<LitInt>,
    pub commas: Vec<Comma>,
    pub rparen: RightParen,
}

/// Modifier expression that tells incat operation how many layers to search.
/// `depth(xx)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierDepth {
    span: Span,
    pub depth: Depth,
    pub lparen: LeftParen,
    pub val: LitIntOrInf,
    pub rparen: RightParen,
}

/// Modifier expression that tells backlinks operation to filter out redirects.
/// `noredir` or `noredir()`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierNoRedir {
    span: Span,
    pub noredir: NoRedir,
    pub lparen: Option<LeftParen>,
    pub rparen: Option<RightParen>,
}

/// Modifier expression that tells backlinks operation to show only redirects.
/// `onlyredir` or `onlyredir()`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierOnlyRedir {
    span: Span,
    pub onlyredir: OnlyRedir,
    pub lparen: Option<LeftParen>,
    pub rparen: Option<RightParen>,
}

/// Modifier expression that tells backlinks operation only to show direct backlinks.
/// `direct` or `direct()`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierDirect {
    span: Span,
    pub direct: Direct,
    pub lparen: Option<LeftParen>,
    pub rparen: Option<RightParen>,
}

expose_span!(ModifierLimit);
expose_span!(ModifierResolve);
expose_span!(ModifierNs);
expose_span!(ModifierDepth);
expose_span!(ModifierNoRedir);
expose_span!(ModifierOnlyRedir);
expose_span!(ModifierDirect);
