//! Core Page List Bot solver traits and types.

use ast::{Expression, Span};
use core::fmt::{self, Debug, Display, Formatter};
use std::{collections::BTreeSet, error::Error};
use super::cvt_attr::AttributeError;

/// Universal result interface.
pub struct Answer<'e, S>
where
    S: Solver,
{
    pub titles: BTreeSet<mwtitle::Title>,
    pub warnings: Vec<SolverError<'e, S>>,
}

impl<'e, S> Debug for Answer<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer").field("titles", &self.titles).field("warnings", &self.warnings).finish()
    }
}

impl<'e, S> Clone for Answer<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            titles: self.titles.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

/// Trait for all solver implementations.
pub trait Solver: Sized {
    type Error;

    async fn solve<'e>(&self, ast: &Expression<'e>) -> Result<Answer<'e, Self>, SolverError<'e, Self>>;
}

/// Common error type for all solver implementations.
#[non_exhaustive]
pub struct SolverError<'e, S>
where
    S: Solver,
{
    pub span: Span<'e>,
    pub inner: SolverErrorInner<'e, S>,
}

impl<'e, S> Clone for SolverError<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            span: self.span,
            inner: self.inner.clone(),
        }
    }
}

impl<'e, S> Copy for SolverError<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Copy,
{}

impl<'e, S> Debug for SolverError<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolverError").field("span", &self.span).field("inner", &self.inner).finish()
    }
}

impl<'e, S> Display for SolverError<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]: {}", self.span.location_line(), self.span.get_utf8_column(), self.inner)
    }
}

impl<'e, S> Error for SolverError<'e, S>
where
    S: Solver + Debug,
    <S as Solver>::Error: Error,
{}

unsafe impl<'e, S> Send for SolverError<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Send,
{}

impl<'e, S> SolverError<'e, S>
where
    S: Solver,
{
    pub fn from_solver_error(span: Span<'e>, content: <S as Solver>::Error) -> Self {
        Self {
            span,
            inner: SolverErrorInner::Solver(content),
        }
    }

    pub fn from_attribute_error(span: Span<'e>, content: AttributeError<'e>) -> Self {
        Self {
            span,
            inner: SolverErrorInner::Attribute(content),
        }
    }
}

pub enum SolverErrorInner<'e, S>
where
    S: Solver,
{
    /// An error happens when converting AST `Attribute` to configurations.
    Attribute(AttributeError<'e>),
    /// An error happens with solver solving.
    Solver(<S as Solver>::Error),
}

impl<'e, S> Debug for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attribute(arg0) => f.debug_tuple("Attribute").field(arg0).finish(),
            Self::Solver(arg0) => f.debug_tuple("Solver").field(arg0).finish(),
        }
    }
}

impl<'e, S> Display for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attribute(content) => Display::fmt(&content, f),//write!(f, "[{}:{}]: {:}", attr.get_span().location_line(), attr.get_span().get_utf8_column(), content),
            Self::Solver(content) => content.fmt(f), //write!(f, "[{}:{}]: {:}", expr.get_span().location_line(), expr.get_span().get_utf8_column(), content),
        }
    }
}

impl<'e, S> Error for SolverErrorInner<'e, S>
where
    S: Solver + Debug,
    <S as Solver>::Error: Error,
{}

unsafe impl<'e, S> Send for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Send,
{}

impl<'e, S> Clone for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Attribute(content) => Self::Attribute(*content),
            Self::Solver(content) => Self::Solver(content.clone()),
        }
    }
}

impl<'e, S> Copy for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Copy,
{}
