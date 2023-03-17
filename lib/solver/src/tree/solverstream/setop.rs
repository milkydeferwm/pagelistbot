//! Set operations.
use core::{fmt::{self, Debug, Formatter}, pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use std::collections::BTreeSet;
use super::TreeSolver;

use futures::{TryStreamExt, TryFuture, Stream, ready, stream::TryCollect, future::{TryJoin, try_join}};
use provider::{PageInfo, DataProvider};

macro_rules! set_operation {
    ($vis:vis, $name:ident, $op:path) => {
        #[pin_project::pin_project]
        #[must_use = "streams do nothing unless you poll them"]
        $vis struct $name<'e, St1, St2, P>
        where
            P: DataProvider + Clone,
            St1: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
            St2: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
        {
            #[pin] sets: TryJoin<TryCollect<St1, BTreeSet<PageInfo>>, TryCollect<St2, BTreeSet<PageInfo>>>,
            output_set: Option<BTreeSet<PageInfo>>,
        }

        impl<'e, St1, St2, P> $name<'e, St1, St2, P>
        where
            P: DataProvider + Clone,
            St1: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
            St2: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
        {
            pub fn new(stream1: St1, stream2: St2) -> Self {
                let sets = try_join(stream1.try_collect(), stream2.try_collect());
                Self {
                    sets,
                    output_set: None,
                }
            }
        }

        impl<'e, St1, St2, P> Debug for $name<'e, St1, St2, P>
        where
            P: DataProvider + Clone,
            <P as DataProvider>::Error: Debug,
            St1: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>> + Debug,
            St2: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>> + Debug,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_struct(&stringify!($name))
                    .field("sets", &self.sets)
                    .field("output_set", &self.output_set)
                    .finish()
            }
        }

        impl<'e, St1, St2, P> Stream for $name<'e, St1, St2, P>
        where
            P: DataProvider + Clone,
            St1: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
            St2: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
        {
            type Item = Result<PageInfo, SolverError<'e, TreeSolver<P>>>;
        
            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                Poll::Ready(loop {
                    if let Some(output_set) = this.output_set.as_mut() {
                        // we already have results, return one-by-one
                        if let Some(item) = output_set.pop_first() {
                            break Some(Ok(item));
                        } else {
                            break None;
                        }
                    } else {
                        // poll the collect future
                        let (set1, set2) = ready!(this.sets.as_mut().try_poll(cx)?);
                        let set = $op(&set1, &set2).cloned().collect::<BTreeSet<_>>();
                        *this.output_set = Some(set);
                    }
                })
            }
        }
    }
}

set_operation!(pub(crate), UnionStream, BTreeSet::union);
set_operation!(pub(crate), IntersectionStream, BTreeSet::intersection);
set_operation!(pub(crate), DifferenceStream, BTreeSet::difference);
set_operation!(pub(crate), XorStream, BTreeSet::symmetric_difference);
