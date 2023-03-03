//! Modifier attributes and filter attributes.
//! Currently only modifier attributes are implemented.

use crate::{Span, expose_span};
use crate::token::Dot;
use crate::modifier::Modifier;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute<'a> {
    Modifier(AttributeModifier<'a>),
}

impl<'a> Attribute<'a> {
    pub fn get_span(&self) -> Span<'a> {
        match self {
            Self::Modifier(x) => x.get_span(),
        }
    }
}

/// Attribute for modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeModifier<'a> {
    span: Span<'a>,
    pub dot: Dot<'a>,
    pub modifier: Modifier<'a>,
}

expose_span!(AttributeModifier);
