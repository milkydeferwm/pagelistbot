//! Core Page List Bot solver traits and types.

use core::fmt;
use interface::types::ast::{Node, Span};
use provider::DataProvider;
use std::{collections::BTreeSet, error::Error};

/// Universal result interface.
pub struct Answer<E>
where
    E: Error,// + Send + Sync,
{
    pub titles: BTreeSet<mwtitle::Title>,
    pub warnings: Vec<SolverError<E>>,
}

/// Trait for all solver implementations.
// #[async_trait::async_trait]
pub trait Solver<'s, P>
where
    P: DataProvider,    // data provider has its own lifetime bound
{
    type InnerError: Error/* + Send + Sync*/;

    async fn solve<'q>(&'s self, ast: &'q Node) -> Result<Answer<Self::InnerError>, SolverError<Self::InnerError>>;
}

/// Common error type for all solver implementations.
#[derive(Debug)]
pub struct SolverError<E> {
    pub span: Span,
    pub content: E
}
/*
unsafe impl<'a, E> Send for SolverError<'a, E>
where
    E: Send,
{}
*/
impl<E> fmt::Display for SolverError<E>
where
    E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Ln {}, Col {}] ‐ [Ln {}, Col {}]: {:}", self.span.begin_line, self.span.begin_col, self.span.end_line, self.span.end_col, self.content)
    }
}

impl <E> Error for SolverError<E>
where
    E: Error
{}
