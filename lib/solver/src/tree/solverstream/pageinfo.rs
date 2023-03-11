//! Retrive information of a page.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::{TreeSolverError, TreeSolver};

use ast::Span;
use futures::{Stream, channel::mpsc::UnboundedSender};
use provider::{PageInfo, Pair, DataProvider};

#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub(crate) struct PageInfoStream<'e, P>
where
    P: DataProvider + Clone,
{
    #[pin] st: <P as DataProvider>::PageInfoRawStream,
    span: Span<'e>,
    warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
}

impl<'e, P> PageInfoStream<'e, P>
where
    P: DataProvider + Clone,
{
    pub fn new<T>(provider: P, titles: T, span: Span<'e>, warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        let st = provider.get_page_info_from_raw(titles);
        Self {
            st,
            span,
            warning_sender,
        }
    }
}

impl<'e, P> Stream for PageInfoStream<'e, P>
where
    P: DataProvider + Clone,
{
    type Item = Result<Pair<PageInfo>, SolverError<'e, TreeSolver<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let poll_result = this.st.as_mut().poll_next(cx);
        if let Poll::Ready(poll_result) = poll_result {
            if let Some(poll_result) = poll_result {
                match poll_result {
                    Ok(item) => Poll::Ready(Some(Ok(item))),
                    Err(e) => Poll::Ready(Some(Err(SolverError::from_solver_error(*this.span, TreeSolverError::Provider(e))))),
                }
            } else {
                Poll::Ready(None)
            }
        } else {
            Poll::Pending
        }
    }
}

unsafe impl<'e, P> Send for PageInfoStream<'e, P>
where
    P: DataProvider + Clone,
    <P as DataProvider>::PageInfoRawStream: Send,
    <P as DataProvider>::Error: Send,
{}
