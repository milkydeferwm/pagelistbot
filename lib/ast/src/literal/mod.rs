//! Literal types.

use alloc::string::String;
use crate::{IntOrInf, Span, expose_span};

#[cfg(feature = "parse")]
pub mod parse;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LitString {
    span: Span,
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LitIntOrInf {
    span: Span,
    pub val: IntOrInf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LitInt {
    span: Span,
    pub val: i32,
}

expose_span!(LitString);
expose_span!(LitIntOrInf);
expose_span!(LitInt);
