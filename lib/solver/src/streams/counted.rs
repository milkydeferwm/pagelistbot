//! Make the output counted.

use ast::Span;
use core::{fmt, pin::Pin, task::{Context, Poll}};
use crate::{SolverResult, RuntimeWarning};
use futures::{Stream, ready, stream::FusedStream};
use pin_project::pin_project;
use provider::DataProvider;
use trio_result::TrioResult;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Counted<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    #[pin] stream: St,
    span: Span,
    count: usize,
    limit: usize,
    terminated: bool,
}

impl<St, P> Counted<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    pub fn new(stream: St, limit: usize, span: Span) -> Self {
        Self {
            stream,
            span,
            count: 0,
            limit,
            terminated: false,
        }
    }
}

impl<St, P> fmt::Debug for Counted<St, P>
where
    St: Stream<Item=SolverResult<P>> + fmt::Debug,
    P: DataProvider,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Counted")
            .field("stream", &self.stream)
            .field("span", &self.span)
            .field("count", &self.count)
            .field("limit", &self.limit)
            .field("terminated", &self.terminated)
            .finish()
    }
}

impl<St, P> FusedStream for Counted<St, P>
where
    Self: Stream,
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

impl<St, P> Stream for Counted<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider + Clone,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready({
            if *this.terminated {
                None
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                match item {
                    x @ TrioResult::Ok(_) => {
                        *this.count += 1;
                        if *this.count <= *this.limit {
                            Some(x)
                        } else {
                            *this.terminated = true;
                            Some(TrioResult::Warn(RuntimeWarning::ResultLimitExceeded { span: *this.span, limit: *this.limit }))
                        }
                    },
                    x => Some(x),
                }
            } else {
                *this.terminated = true;
                None
            }
        })
    }
}
