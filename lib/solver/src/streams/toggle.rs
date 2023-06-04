//! Toggle operation.

use ast::Span;
use core::{pin::Pin, task::{Context, Poll}};
use crate::{SolverResult, RuntimeError};
use futures::{Stream, ready};
use provider::{DataProvider};
use trio_result::TrioResult;

#[pin_project::pin_project]
pub struct ToggleStream<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    #[pin] stream: St,
    span: Span,
}

impl<St, P> ToggleStream<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    pub fn new(stream: St, span: Span) -> Self {
        Self {
            stream,
            span,
        }
    }
}

impl<St, P> Stream for ToggleStream<St, P>
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
                    TrioResult::Ok(mut info) => {
                        info.swap();

                        // No page's associated page lies in virtual namespaces.
                        // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
                        // In such case, we do not yield this item, the loop goes on.
                        let t = match info.get_title() {
                            Ok(t) => t,
                            Err(e) => break Some(TrioResult::Err(RuntimeError::PageInfo { span: *this.span, error: e })),
                        };
                        if t.namespace() >= 0 {
                            break Some(TrioResult::Ok(info));
                        }
                    },
                    x => break Some(x),
                }
            } else {
                break None;
            }
        })
    }
}
