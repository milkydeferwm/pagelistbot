//! Core Page List Bot solver traits and types.

use core::fmt;
use interface::types::ast::{Node, Span};
use std::{collections::BTreeSet, error::Error};

/// Universal result interface.
pub struct Answer<S>
where
    S: Solver,
{
    pub titles: BTreeSet<mwtitle::Title>,
    pub warnings: Vec<SolverError<S>>,
}

impl<S> fmt::Debug for Answer<S>
where
    S: Solver,
    <S as Solver>::InnerError: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer").field("titles", &self.titles).field("warnings", &self.warnings).finish()
    }
}

impl<S> Clone for Answer<S>
where
    S: Solver,
    <S as Solver>::InnerError: Clone,
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
    type InnerError;

    async fn solve<'q>(&self, ast: &'q Node) -> Result<Answer<Self>, SolverError<Self>>;
}

/// Common error type for all solver implementations.
pub struct SolverError<S>
where
    S: Solver,
{
    pub span: Span,
    pub content: <S as Solver>::InnerError,
}

impl<S> fmt::Debug for SolverError<S>
where
    S: Solver,
    <S as Solver>::InnerError: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolverError").field("span", &self.span).field("content", &self.content).finish()
    }
}

impl<S> fmt::Display for SolverError<S>
where
    S: Solver,
    <S as Solver>::InnerError: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Ln {}, Col {}] ‐ [Ln {}, Col {}]: {:}", self.span.begin_line, self.span.begin_col, self.span.end_line, self.span.end_col, self.content)
    }
}

impl<S> Error for SolverError<S>
where
    S: Solver + fmt::Debug,
    <S as Solver>::InnerError: Error,
{}

unsafe impl<S> Send for SolverError<S>
where
    S: Solver,
    <S as Solver>::InnerError: Send,
{}

impl<S> Clone for SolverError<S>
where
    S: Solver,
    <S as Solver>::InnerError: Clone,
{
    fn clone(&self) -> Self {
        Self {
            span: self.span,
            content: self.content.clone(),
        }
    }
}

impl<S> Copy for SolverError<S>
where
    S: Solver,
    <S as Solver>::InnerError: Copy,
{}
