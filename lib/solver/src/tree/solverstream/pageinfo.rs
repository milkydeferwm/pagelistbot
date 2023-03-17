//! Retrive information of a page.
use crate::SolverError;
use super::{TreeSolverError, TreeSolver, unique::TryUnique};

use ast::Span;
use futures::{Stream, TryStreamExt};
use provider::{PageInfo, DataProvider};

pub(crate) type PageInfoStream<'e, P>
where
    P: DataProvider + Clone,
= impl Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>;

pub(crate) fn make_pageinfo_stream<P>(
    titles: Vec<String>,
    provider: P,
    span: Span<'_>,
) -> PageInfoStream<'_, P>
where
    P: DataProvider + Clone,
{
    TryUnique::new(
        provider.get_page_info_from_raw(titles)
        .map_err(move |e| SolverError::from_solver_error(span, TreeSolverError::Provider(e)))
    )
}
