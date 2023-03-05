//! Expressions.

use alloc::{
    boxed::Box,
    vec::Vec,
};
use crate::{Span, expose_span};
use crate::attribute::Attribute;
use crate::literal::LitString;
use crate::token::{
    And, Add, Sub, Caret, LeftParen, RightParen, Comma,
    Page, Link, LinkTo, Embed, InCat, Prefix, Toggle,
};

#[cfg(feature = "parse")]
pub mod parse;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression<'a> {
    And(ExpressionAnd<'a>),
    Add(ExpressionAdd<'a>),
    Sub(ExpressionSub<'a>),
    Xor(ExpressionXor<'a>),
    Paren(ExpressionParen<'a>),
    Page(ExpressionPage<'a>),
    Link(ExpressionLink<'a>),
    LinkTo(ExpressionLinkTo<'a>),
    Embed(ExpressionEmbed<'a>),
    InCat(ExpressionInCat<'a>),
    Prefix(ExpressionPrefix<'a>),
    Toggle(ExpressionToggle<'a>),
}

impl<'a> Expression<'a> {
    /// Get the span for this item.
    pub fn get_span(&self) -> Span<'a> {
        match self {
            Self::And(expr) => expr.get_span(),
            Self::Add(expr) => expr.get_span(),
            Self::Sub(expr) => expr.get_span(),
            Self::Xor(expr) => expr.get_span(),
            Self::Paren(expr) => expr.get_span(),
            Self::Page(expr) => expr.get_span(),
            Self::Link(expr) => expr.get_span(),
            Self::LinkTo(expr) => expr.get_span(),
            Self::Embed(expr) => expr.get_span(),
            Self::InCat(expr) => expr.get_span(),
            Self::Prefix(expr) => expr.get_span(),
            Self::Toggle(expr) => expr.get_span(),
        }
    }
}

/// Set operation and
/// `<expr> & <expr>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionAnd<'a> {
    span: Span<'a>,
    pub expr1: Box<Expression<'a>>,
    pub and: And<'a>,
    pub expr2: Box<Expression<'a>>,
}

/// Set operation add
/// `<expr> + <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionAdd<'a> {
    span: Span<'a>,
    pub expr1: Box<Expression<'a>>,
    pub add: Add<'a>,
    pub expr2: Box<Expression<'a>>,
}

/// Set operation sub
/// `<expr> - <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionSub<'a> {
    span: Span<'a>,
    pub expr1: Box<Expression<'a>>,
    pub sub: Sub<'a>,
    pub expr2: Box<Expression<'a>>,
}

/// Set operation xor
/// `<expr> ^ <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionXor<'a> {
    span: Span<'a>,
    pub expr1: Box<Expression<'a>>,
    pub xor: Caret<'a>,
    pub expr2: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionParen<'a> {
    span: Span<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
}

/// Primitive operation page info
/// `page("...","...")`
/// `"...","..."`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPage<'a> {
    span: Span<'a>,
    pub page: Option<Page<'a>>,
    pub lparen: Option<LeftParen<'a>>,
    pub vals: Vec<LitString<'a>>,
    pub commas: Vec<Comma<'a>>,
    pub rparen: Option<RightParen<'a>>,
}

/// Composite operation link
/// `link(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionLink<'a> {
    span: Span<'a>,
    pub link: Link<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

/// Composite operation linkto
/// `linkto(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionLinkTo<'a> {
    span: Span<'a>,
    pub linkto: LinkTo<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

/// Composite operation embed
/// `embed(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionEmbed<'a> {
    span: Span<'a>,
    pub embed: Embed<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

/// Composite operation incat
/// `incat(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionInCat<'a> {
    span: Span<'a>,
    pub incat: InCat<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

/// Composite operation prefix
/// `prefix(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPrefix<'a> {
    span: Span<'a>,
    pub prefix: Prefix<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
    pub attributes: Vec<Attribute<'a>>,
}

/// Composite operation toggle
/// `toggle(<expr>)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionToggle<'a> {
    span: Span<'a>,
    pub toggle: Toggle<'a>,
    pub lparen: LeftParen<'a>,
    pub expr: Box<Expression<'a>>,
    pub rparen: RightParen<'a>,
}

expose_span!(ExpressionAdd);
expose_span!(ExpressionAnd);
expose_span!(ExpressionSub);
expose_span!(ExpressionXor);
expose_span!(ExpressionParen);
expose_span!(ExpressionPage);
expose_span!(ExpressionLink);
expose_span!(ExpressionLinkTo);
expose_span!(ExpressionEmbed);
expose_span!(ExpressionInCat);
expose_span!(ExpressionPrefix);
expose_span!(ExpressionToggle);
