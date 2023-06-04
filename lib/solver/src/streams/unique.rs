//! Make the output unique.

use ast::Span;
use core::{fmt, pin::Pin, task::{Context, Poll}};
use crate::{SolverResult, RuntimeError};
use futures::{Stream, ready, stream::FusedStream};
use mwtitle::Title;
use pin_project::pin_project;
use provider::DataProvider;
use std::collections::BTreeSet;
use trio_result::TrioResult;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Unique<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    #[pin] stream: St,
    span: Span,
    yielded: BTreeSet<Title>,
}

impl<St, P> Unique<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    pub fn new(stream: St, span: Span) -> Self {
        Self {
            stream,
            span,
            yielded: BTreeSet::new(),
        }
    }
}

impl<St, P> fmt::Debug for Unique<St, P>
where
    St: Stream<Item=SolverResult<P>> + fmt::Debug,
    P: DataProvider,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unique")
            .field("stream", &self.stream)
            .field("span", &self.span)
            .field("yielded", &self.yielded)
            .finish()
    }
}

impl<St, P> FusedStream for Unique<St, P>
where
    Self: Stream,
    St: Stream<Item=SolverResult<P>> + FusedStream,
    P: DataProvider,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St, P> Stream for Unique<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                match item {
                    TrioResult::Ok(info) => {
                        let t = match info.get_title() {
                            Ok(x) => x,
                            Err(e) => break Some(TrioResult::Err(RuntimeError::PageInfo { span: *this.span, error: e })),
                        };
                        if !this.yielded.contains(t) {
                            this.yielded.insert(t.to_owned());
                            break Some(TrioResult::Ok(info));
                        }
                    },
                    x =>  break Some(x),
                }
            } else {
                break None;
            }
        })
    }
}
