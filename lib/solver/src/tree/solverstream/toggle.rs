//! Toggle operation.
use crate::SolverError;
use super::TreeSolver;

use futures::{Stream, TryStreamExt};
use provider::{PageInfo, DataProvider};

pub(super) type ToggleStream<'e, St, P>
where
    P: DataProvider + Clone,
    St: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
= impl Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>;

pub(super) fn make_toggle_stream<'e, St, P>(
    stream: St,
) -> ToggleStream<'e, St, P>
where
    P: DataProvider + Clone,
    St: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
{
    stream.try_filter_map(|mut x| async move {
        x.swap();

        // No page's associated page lies in virtual namespaces.
        // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
        if x.get_title().unwrap().namespace() >= 0 {
            Ok(Some(x))
        } else { Ok(None) }
    })
}
