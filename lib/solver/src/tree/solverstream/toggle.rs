//! Toggle operation.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::TreeSolver;

use ast::Span;
use futures::{stream::TryStream, Stream, channel::mpsc::UnboundedSender};
use provider::{Pair, PageInfo, DataProvider};

#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
#[derive(Debug)]
pub(crate) struct ToggleStream<'e, F, P>
where
    P: DataProvider + Clone,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
{
    #[pin] st: F,
    span: Span<'e>,
    warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
}

impl<'e, F, P> ToggleStream<'e, F, P>
where
    P: DataProvider + Clone,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
{
    pub fn new(stream: F, span: Span<'e>, warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>) -> Self {
        Self {
            st: stream,
            span,
            warning_sender,
        }
    }
}

impl<'e, F, P> Stream for ToggleStream<'e, F, P>
where
    P: DataProvider + Clone,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
{
    type Item = Result<Pair<PageInfo>, SolverError<'e, TreeSolver<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        match this.st.as_mut().try_poll_next(cx)? {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some((this_page, associated_page))) => {
                let associated_title = associated_page.get_title().unwrap();
                if associated_title.namespace() >= 0 {
                    Poll::Ready(Some(Ok((associated_page, this_page))))
                } else {
                    // No page's associated page lies in virtual namespaces.
                    // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
                    Poll::Pending
                }
            }
        }
    }
}

unsafe impl<'e, F, P> Send for ToggleStream<'e, F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>> + Send,
    P: DataProvider + Clone,
    <P as DataProvider>::Error: Send,
{}
