//! Core Page List Bot solver traits and types.

use interface::types::ast::Node;
use provider::DataProvider;

/// Universal result interface.
pub struct Answer<'a, E>
where
    E: std::error::Error + Send + Sync,
{
    pub titles: std::collections::BTreeSet<mwtitle::Title>,
    pub warnings: Vec<Error<'a, E>>,
}

/// Trait for all solver implementations.
#[async_trait::async_trait]
pub trait Solver<'solver, 'query, P>
where
    P: DataProvider,
{
    type InnerError: std::error::Error + Send + Sync;

    async fn solve(&'solver self, ast: &'query Node) -> Result<Answer<'query, Self::InnerError>, Error<'query, Self::InnerError>>;
}

/// Common error type for all solver implementations.
#[derive(Debug)]
pub struct Error<'a, E>
where
    E: std::error::Error + Send + Sync,
{
    pub node: &'a Node,
    pub content: E
}

unsafe impl<'a, E> Send for Error<'a, E>
where
    E: std::error::Error + Send + Sync,
{}
