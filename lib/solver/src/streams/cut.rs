//! Make the output cut on error.

use core::{fmt, pin::Pin, task::{Context, Poll}};
use crate::SolverResult;
use futures::{Stream, ready, stream::FusedStream};
use pin_project::pin_project;
use provider::DataProvider;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct CutError<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    #[pin] stream: St,
    terminated: bool,
}

impl<St, P> CutError<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            terminated: false,
        }
    }
}

impl<St, P> fmt::Debug for CutError<St, P>
where
    St: Stream<Item=SolverResult<P>> + fmt::Debug,
    P: DataProvider,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CutError")
            .field("stream", &self.stream)
            .field("terminated", &self.terminated)
            .finish()
    }
}

impl<St, P> FusedStream for CutError<St, P>
where
    Self: Stream,
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

impl<St, P> Stream for CutError<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready({
            if *this.terminated {
                None
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                if item.is_err() {
                    *this.terminated = true;
                }
                Some(item)
            } else {
                *this.terminated = true;
                None
            }
        })
    }
}
