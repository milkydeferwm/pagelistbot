//! Set operations.

use ast::Span;
use core::{mem, pin::Pin, task::{Context, Poll}};
use crate::SolverResult;
use futures::{Stream, stream::FusedStream};
use provider::{DataProvider, PageInfo};
use std::collections::BTreeSet;
use trio_result::TrioResult;

macro_rules! set {
    ($name:ident, $op:path) => {
        #[pin_project::pin_project]
        #[must_use = "streams do nothing unless you poll them"]
        pub struct $name<St1, St2, P>
        where
            St1: Stream<Item=SolverResult<P>>,
            St2: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            #[pin] source_1: Option<St1>,
            #[pin] source_2: Option<St2>,
            span: Span,

            // internal
            set_1: BTreeSet<PageInfo>,
            set_2: BTreeSet<PageInfo>,
            output: BTreeSet<PageInfo>,
        }

        impl<St1, St2, P> $name<St1, St2, P>
        where
            St1: Stream<Item=SolverResult<P>>,
            St2: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            pub fn new(stream_1: St1, stream_2: St2, span: Span) -> Self {
                Self {
                    source_1: Some(stream_1),
                    source_2: Some(stream_2),
                    span,

                    set_1: BTreeSet::new(),
                    set_2: BTreeSet::new(),
                    output: BTreeSet::new(),
                }
            }
        }

        impl<St1, St2, P> FusedStream for $name<St1, St2, P>
        where
            St1: Stream<Item=SolverResult<P>>,
            St2: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            fn is_terminated(&self) -> bool {
                return self.source_1.is_none() && self.source_2.is_none() && self.output.is_empty();
            }
        }

        impl<St1, St2, P> Stream for $name<St1, St2, P>
        where
            St1: Stream<Item=SolverResult<P>>,
            St2: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            type Item = SolverResult<P>;
        
            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                Poll::Ready(loop {
                    let mut all_ready = true;
                    // try poll stream 1
                    if let Some(s) = this.source_1.as_mut().as_pin_mut() {
                        if let Poll::Ready(item) = s.poll_next(cx) {
                            if let Some(item) = item {
                                match item {
                                    TrioResult::Ok(info) => { this.set_1.insert(info); },
                                    x => break Some(x),
                                }
                            } else {
                                this.source_1.set(None);
                            }
                        } else {
                            all_ready = false;
                        }
                    }
                    // try poll stream 2
                    if let Some(s) = this.source_2.as_mut().as_pin_mut() {
                        if let Poll::Ready(item) = s.poll_next(cx) {
                            if let Some(item) = item {
                                match item {
                                    TrioResult::Ok(info) => { this.set_2.insert(info); },
                                    x => break Some(x),
                                }
                            } else {
                                this.source_2.set(None);
                            }
                        } else {
                            all_ready = false;
                        }
                    }
                    if this.source_1.is_none() && this.source_2.is_none() {
                        if this.set_1.is_empty() && this.set_2.is_empty() {
                            // fetch output from output set.
                            if let Some(item) = this.output.pop_first() {
                                break Some(TrioResult::Ok(item));
                            } else {
                                break None;
                            }
                        } else {
                            // create output set.
                            let set_1 = mem::take(this.set_1);
                            let set_2 = mem::take(this.set_2);
                            let output = $op(&set_1, &set_2).cloned().collect::<BTreeSet<_>>();
                            *this.output = output;
                        }
                    } else if !all_ready {
                        return Poll::Pending;
                    }
                })
            }
        }
    }
}

set!(UnionStream, BTreeSet::union);
set!(IntersectionStream, BTreeSet::intersection);
set!(DifferenceStream, BTreeSet::difference);
set!(XorStream, BTreeSet::symmetric_difference);
