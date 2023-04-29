//! Core Page List Bot solver traits and types.

use ast::{Expression, Span};
use core::fmt::{self, Debug, Display, Formatter};
use std::{collections::BTreeSet, error::Error};
use super::cvt_attr::AttributeError;

/// Universal result interface.
pub struct Answer<S>
where
    S: Solver,
{
    pub titles: BTreeSet<mwtitle::Title>,
    pub warnings: Vec<SolverError<S>>,
}

impl<S> Debug for Answer<S>
where
    S: Solver,
    <S as Solver>::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer").field("titles", &self.titles).field("warnings", &self.warnings).finish()
    }
}

impl<S> Clone for Answer<S>
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
    type Error: Error + 'static;

    async fn solve(&self, ast: &Expression) -> Result<Answer<Self>, SolverError<Self>>;
}

/// Common error type for all solver implementations.
pub struct SolverError<S>
where
    S: Solver,
{
    pub span: Span,
    pub inner: SolverErrorInner<S>,
}

impl<S> Clone for SolverError<S>
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

impl<S> Debug for SolverError<S>
where
    S: Solver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolverError")
            .field("span", &self.span)
            .field("inner", &self.inner)
            .finish()
    }
}

/*
impl<S> Display for SolverError<S>
where
    S: Solver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]: {}", self.span.location_line(), self.span.get_utf8_column(), self.inner)
    }
}
*/

/*
impl<'e, S> Error for SolverError<'e, S>
where
    S: Solver,
{}
*/
/*
unsafe impl<S> Send for SolverError<S>
where
    S: Solver,
    <S as Solver>::Error: Send,
{}
*/

impl<S> SolverError<S>
where
    S: Solver,
{
    pub fn from_solver_error(span: Span, content: <S as Solver>::Error) -> Self {
        Self {
            span,
            inner: SolverErrorInner::Solver(content),
        }
    }

    pub fn from_attribute_error(span: Span, content: AttributeError) -> Self {
        Self {
            span,
            inner: SolverErrorInner::Attribute(content),
        }
    }
}

#[non_exhaustive]
pub enum SolverErrorInner<S>
where
    S: Solver,
{
    /// An error happens when converting AST `Attribute` to configurations.
    Attribute(AttributeError),
    /// An error happens with solver solving.
    Solver(<S as Solver>::Error),
}

impl<S> Debug for SolverErrorInner<S>
where
    S: Solver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attribute(arg0) => f.debug_tuple("Attribute").field(arg0).finish(),
            Self::Solver(arg0) => f.debug_tuple("Solver").field(arg0).finish(),
        }
    }
}

impl<S> Display for SolverErrorInner<S>
where
    S: Solver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attribute(content) => Display::fmt(&content, f),
            Self::Solver(content) => Display::fmt(&content, f),
        }
    }
}

impl<S> Error for SolverErrorInner<S>
where
    S: Solver,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Attribute(content) => Some(content),
            Self::Solver(content) => Some(content),
        }
    }
}

/*
unsafe impl<'e, S> Send for SolverErrorInner<'e, S>
where
    S: Solver,
    <S as Solver>::Error: Send,
{}
*/

impl<S> Clone for SolverErrorInner<S>
where
    S: Solver,
    <S as Solver>::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Attribute(content) => Self::Attribute(content.clone()),
            Self::Solver(content) => Self::Solver(content.clone()),
        }
    }
}
