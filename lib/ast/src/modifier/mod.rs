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
pub enum Modifier<'a> {
    Limit(ModifierLimit<'a>),
    Resolve(ModifierResolve<'a>),
    Ns(ModifierNs<'a>),
    Depth(ModifierDepth<'a>),
    NoRedir(ModifierNoRedir<'a>),
    OnlyRedir(ModifierOnlyRedir<'a>),
    Direct(ModifierDirect<'a>),
}

impl<'a> Modifier<'a> {
    pub fn get_span(&self) -> Span<'a> {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierLimit<'a> {
    span: Span<'a>,
    pub limit: Limit<'a>,
    pub lparen: LeftParen<'a>,
    pub val: LitIntOrInf<'a>,
    pub rparen: RightParen<'a>,
}

/// Modifier expression that defines whether to resolve redirects.
/// `resolve` or `resolve()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierResolve<'a> {
    span: Span<'a>,
    pub resolve: Resolve<'a>,
    pub lparen: Option<LeftParen<'a>>,
    pub rparen: Option<RightParen<'a>>,
}

/// Modifier expression that contrains the results inside certain namespaces.
/// `ns(xx,xx)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModifierNs<'a> {
    span: Span<'a>,
    pub ns: Ns<'a>,
    pub lparen: LeftParen<'a>,
    pub vals: Vec<LitInt<'a>>,
    pub commas: Vec<Comma<'a>>,
    pub rparen: RightParen<'a>,
}

/// Modifier expression that tells incat operation how many layers to search.
/// `depth(xx)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierDepth<'a> {
    span: Span<'a>,
    pub depth: Depth<'a>,
    pub lparen: LeftParen<'a>,
    pub val: LitIntOrInf<'a>,
    pub rparen: RightParen<'a>,
}

/// Modifier expression that tells backlinks operation to filter out redirects.
/// `noredir` or `noredir()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierNoRedir<'a> {
    span: Span<'a>,
    pub noredir: NoRedir<'a>,
    pub lparen: Option<LeftParen<'a>>,
    pub rparen: Option<RightParen<'a>>,
}

/// Modifier expression that tells backlinks operation to show only redirects.
/// `onlyredir` or `onlyredir()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierOnlyRedir<'a> {
    span: Span<'a>,
    pub onlyredir: OnlyRedir<'a>,
    pub lparen: Option<LeftParen<'a>>,
    pub rparen: Option<RightParen<'a>>,
}

/// Modifier expression that tells backlinks operation only to show direct backlinks.
/// `direct` or `direct()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierDirect<'a> {
    span: Span<'a>,
    pub direct: Direct<'a>,
    pub lparen: Option<LeftParen<'a>>,
    pub rparen: Option<RightParen<'a>>,
}

expose_span!(ModifierLimit);
expose_span!(ModifierResolve);
expose_span!(ModifierNs);
expose_span!(ModifierDepth);
expose_span!(ModifierNoRedir);
expose_span!(ModifierOnlyRedir);
expose_span!(ModifierDirect);
