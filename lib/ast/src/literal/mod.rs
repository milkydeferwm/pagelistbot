//! Literal types.

use alloc::string::String;
use crate::{IntOrInf, Span, expose_span};

#[cfg(feature = "parse")]
pub mod parse;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LitString<'a> {
    span: Span<'a>,
    pub val: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LitIntOrInf<'a> {
    span: Span<'a>,
    pub val: IntOrInf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LitInt<'a> {
    span: Span<'a>,
    pub val: i32,
}

expose_span!(LitString);
expose_span!(LitIntOrInf);
expose_span!(LitInt);
