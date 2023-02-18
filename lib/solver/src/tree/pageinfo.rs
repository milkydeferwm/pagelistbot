//! Retrive information of a page.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::{TreeSolverError, TreeSolver};

use futures::{Stream, channel::mpsc::UnboundedSender};
use interface::types::ast::Span;
use provider::{PageInfo, Pair, DataProvider};

#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub(super) struct PageInfoStream<'p, P>
where
    P: DataProvider + 'p,
{
    #[pin] st: <P as DataProvider>::PageInfoRawStream,
    span: Span,
    warning_sender: UnboundedSender<SolverError<TreeSolver<'p, P>>>,
}

impl<'p, P> PageInfoStream<'p, P>
where
    P: DataProvider,
{
    pub fn new<T: IntoIterator<Item = String>>(provider: &'p P, titles: T, span: Span, warning_sender: UnboundedSender<SolverError<TreeSolver<'p, P>>>) -> Self {
        let st = provider.get_page_info_from_raw(titles);
        Self {
            st,
            span,
            warning_sender,
        }
    }
}

impl<'p, P> Stream for PageInfoStream<'p, P>
where
    P: DataProvider,
{
    type Item = Result<Pair<PageInfo>, SolverError<TreeSolver<'p, P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let poll_result = this.st.as_mut().poll_next(cx);
        if let Poll::Ready(poll_result) = poll_result {
            if let Some(poll_result) = poll_result {
                match poll_result {
                    Ok(item) => Poll::Ready(Some(Ok(item))),
                    Err(e) => {
                        Poll::Ready(Some(Err(SolverError {
                            span: *(this.span),
                            content: TreeSolverError::Provider(e),
                        })))
                    },
                }
            } else {
                Poll::Ready(None)
            }
        } else {
            Poll::Pending
        }
    }
}

unsafe impl<'p, P> Send for PageInfoStream<'p, P>
where
    P: DataProvider,
    <P as DataProvider>::PageInfoRawStream: Send,
    <P as DataProvider>::Error: Send,
{}
