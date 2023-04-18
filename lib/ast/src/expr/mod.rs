//! Expressions.

use alloc::{
    sync::Arc,
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
pub enum Expression {
    And(ExpressionAnd),
    Add(ExpressionAdd),
    Sub(ExpressionSub),
    Xor(ExpressionXor),
    Paren(ExpressionParen),
    Page(ExpressionPage),
    Link(ExpressionLink),
    LinkTo(ExpressionLinkTo),
    Embed(ExpressionEmbed),
    InCat(ExpressionInCat),
    Prefix(ExpressionPrefix),
    Toggle(ExpressionToggle),
}

impl Expression {
    /// Get the span for this item.
    pub fn get_span(&self) -> &Span {
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
pub struct ExpressionAnd {
    span: Span,
    pub expr1: Arc<Expression>,
    pub and: And,
    pub expr2: Arc<Expression>,
}

/// Set operation add
/// `<expr> + <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionAdd {
    span: Span,
    pub expr1: Arc<Expression>,
    pub add: Add,
    pub expr2: Arc<Expression>,
}

/// Set operation sub
/// `<expr> - <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionSub {
    span: Span,
    pub expr1: Arc<Expression>,
    pub sub: Sub,
    pub expr2: Arc<Expression>,
}

/// Set operation xor
/// `<expr> ^ <expr>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionXor {
    span: Span,
    pub expr1: Arc<Expression>,
    pub xor: Caret,
    pub expr2: Arc<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionParen {
    span: Span,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
}

/// Primitive operation page info
/// `page("...","...")`
/// `"...","..."`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPage {
    span: Span,
    pub page: Option<Page>,
    pub lparen: Option<LeftParen>,
    pub vals: Vec<LitString>,
    pub commas: Vec<Comma>,
    pub rparen: Option<RightParen>,
}

/// Composite operation link
/// `link(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionLink {
    span: Span,
    pub link: Link,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
    pub attributes: Vec<Attribute>,
}

/// Composite operation linkto
/// `linkto(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionLinkTo {
    span: Span,
    pub linkto: LinkTo,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
    pub attributes: Vec<Attribute>,
}

/// Composite operation embed
/// `embed(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionEmbed {
    span: Span,
    pub embed: Embed,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
    pub attributes: Vec<Attribute>,
}

/// Composite operation incat
/// `incat(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionInCat {
    span: Span,
    pub incat: InCat,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
    pub attributes: Vec<Attribute>,
}

/// Composite operation prefix
/// `prefix(<expr>)<attributes>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPrefix {
    span: Span,
    pub prefix: Prefix,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
    pub attributes: Vec<Attribute>,
}

/// Composite operation toggle
/// `toggle(<expr>)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionToggle {
    span: Span,
    pub toggle: Toggle,
    pub lparen: LeftParen,
    pub expr: Arc<Expression>,
    pub rparen: RightParen,
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
