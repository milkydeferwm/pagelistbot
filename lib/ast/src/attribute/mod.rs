//! Modifier attributes and filter attributes.
//! Currently only modifier attributes are implemented.

use core::hash::{Hash, Hasher};
use crate::{Span, expose_span};
use crate::token::Dot;
use crate::modifier::Modifier;

#[cfg(feature = "parse")]
pub mod parse;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    Modifier(AttributeModifier),
}

impl Attribute {
    pub fn get_span(&self) -> Span {
        match self {
            Self::Modifier(x) => x.get_span(),
        }
    }
}

/// Attribute for modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeModifier {
    span: Span,
    pub dot: Dot,
    pub modifier: Modifier,
}

impl Hash for AttributeModifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dot.hash(state);
        self.modifier.hash(state);
    }
}

expose_span!(AttributeModifier);
