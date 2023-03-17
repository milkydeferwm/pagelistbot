//! Abstract syntax tree for Page List Bot expressions.

#![no_std]
extern crate alloc;

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;

pub mod attribute;
pub mod expr;
pub mod literal;
pub mod modifier;
pub mod token;
#[cfg(feature = "parse")]
mod parse_util;

pub use attribute::{Attribute, AttributeModifier};
pub use expr::{
    Expression,
    ExpressionAnd, ExpressionAdd, ExpressionSub, ExpressionXor,
    ExpressionParen,
    ExpressionPage, ExpressionLink, ExpressionLinkTo, ExpressionEmbed, ExpressionInCat, ExpressionPrefix, ExpressionToggle,
};
pub use intorinf::IntOrInf;
pub use literal::{LitString, LitIntOrInf};
pub use modifier::{
    Modifier,
    ModifierLimit, ModifierResolve,
    ModifierNs,
    ModifierDepth,
    ModifierNoRedir, ModifierOnlyRedir, ModifierDirect,
};
pub use token::{
    Dot, Comma, LeftParen, RightParen, And, Add, Sub, Caret,
    Page, Link, LinkTo, Embed, InCat, Prefix, Toggle,
    Limit, Resolve, Ns, Depth, NoRedir, OnlyRedir, Direct,
};

pub(crate) use macros::expose_span;

mod macros {
    macro_rules! expose_span {
        ($class:ident) => {
            /// Get the span for this item.
            impl<'a> $class<'a> {
                pub fn get_span(&self) -> crate::Span<'a> {
                    self.span
                }
            }
        }
    }

    pub(crate) use expose_span;
}
