//! Set operations.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use std::collections::BTreeSet;
use super::TreeSolver;

use ast::Span;
use futures::{stream::TryStream, Stream, channel::mpsc::UnboundedSender};
use provider::{PageInfo, Pair, DataProvider};

macro_rules! set_operation {
    ($vis:vis, $name:ident, $op:path, $from_node:ident) => {
        #[pin_project::pin_project]
        #[must_use = "streams do nothing unless you poll them"]
        #[derive(Debug)]
        $vis struct $name<'e, F1, F2, P>
        where
            P: DataProvider + Clone,
            F1: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            F2: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
        {
            #[pin] st1: F1,
            #[pin] st2: F2,
            st1_complete: bool,
            st2_complete: bool,
            set_1: BTreeSet<Pair<PageInfo>>,
            set_2: BTreeSet<Pair<PageInfo>>,
            output_set: Option<BTreeSet<Pair<PageInfo>>>,
            span: Span<'e>,
            warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
        }

        impl<'e, F1, F2, P> $name<'e, F1, F2, P>
        where
            P: DataProvider + Clone,
            F1: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            F2: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
        {
            pub fn new(stream1: F1, stream2: F2, span: Span<'e>, warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>) -> Self {
                Self {
                    st1: stream1,
                    st2: stream2,
                    st1_complete: false,
                    st2_complete: false,
                    set_1: BTreeSet::new(),
                    set_2: BTreeSet::new(),
                    output_set: None,
                    span,
                    warning_sender,
                }
            }
        }

        impl<'e, F1, F2, P> Stream for $name<'e, F1, F2, P>
        where
            P: DataProvider + Clone,
            F1: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            F2: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
        {
            type Item = Result<Pair<PageInfo>, SolverError<'e, TreeSolver<P>>>;
        
            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                // we already have results, return one-by-one
                if let Some(output_set) = this.output_set.as_mut() {
                    // we already have results, return one-by-one
                    if let Some(item) = output_set.pop_first() {
                        Poll::Ready(Some(Ok(item)))
                    } else {
                        Poll::Ready(None)
                    }
                } else {    // we don't already have results, poll them
                    if !*(this.st1_complete) {
                        // try poll stream 1
                        if let Poll::Ready(item) = this.st1.as_mut().try_poll_next(cx)? {
                            if item.is_none() {
                                *(this.st1_complete) = true;
                            }
                            this.set_1.extend(item);
                        }
                    }
                    if !*(this.st2_complete) {
                        // try poll stream 2
                        if let Poll::Ready(item) = this.st2.as_mut().try_poll_next(cx)? {
                            if item.is_none() {
                                *(this.st2_complete) = true;
                            }
                            this.set_2.extend(item);
                        }
                    }
                    // are the two streams all complete?
                    if *(this.st1_complete) && *(this.st2_complete) {
                        let mut combined = $op(&this.set_1, &this.set_2).cloned().collect::<BTreeSet<_>>();
                        // clear two sets
                        this.set_1.clear();
                        this.set_2.clear();
                        // get the first result of the combined set
                        let first_returned = combined.pop_first();
                        // put remaining to output_set
                        *(this.output_set) = Some(combined);
                        if let Some(item) = first_returned {
                            Poll::Ready(Some(Ok(item)))
                        } else {
                            Poll::Ready(None)
                        }
                    } else {
                        Poll::Pending
                    }
                }
            }
        }

        unsafe impl<'e, F1, F2, P> Send for $name<'e, F1, F2, P>
        where
            P: DataProvider + Clone,
            <P as DataProvider>::Error: Send,
            F1: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>> + Send,
            F2: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>> + Send,
        {}
    };
}

set_operation!(pub(crate), UnionStream, BTreeSet::union, union_from_node);
set_operation!(pub(crate), IntersectionStream, BTreeSet::intersection, intersection_from_node);
set_operation!(pub(crate), DifferenceStream, BTreeSet::difference, difference_from_node);
set_operation!(pub(crate), XorStream, BTreeSet::symmetric_difference, xor_from_node);
