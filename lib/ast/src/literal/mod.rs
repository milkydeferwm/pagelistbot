//! Literal types.

use alloc::string::String;
use core::hash::{Hash, Hasher};
use crate::{IntOrInf, Span, expose_span};

#[cfg(feature = "parse")]
pub mod parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitString {
    span: Span,
    pub val: String,
}

impl Hash for LitString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitIntOrInf {
    span: Span,
    pub val: IntOrInf,
}

impl Hash for LitIntOrInf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitInt {
    span: Span,
    pub val: i32,
}

impl Hash for LitInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

expose_span!(LitString);
expose_span!(LitIntOrInf);
expose_span!(LitInt);
