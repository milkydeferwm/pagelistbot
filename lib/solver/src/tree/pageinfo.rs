//! Retrive information of a page.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::TreeSolverError;

use futures::{Stream, channel::mpsc::UnboundedSender};
use interface::types::ast::{Node, Span, NumberOrInf};
use pin_project::pin_project;
use provider::{PageInfo, Pair, DataProvider};

#[pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub(super) struct PageInfoStream<P>
where
    P: DataProvider,
{
    #[pin] st: <P as DataProvider>::PageInfoRawStream,
    span: Span,
    warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>,
}

impl<P> PageInfoStream<P>
where
    P: DataProvider,
{
    pub fn new<T: IntoIterator<Item = String>>(provider: P, titles: T, span: Span, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> Self {
        let st = provider.get_page_info_from_raw(titles);
        Self {
            st,
            span,
            warning_sender,
        }
    }
}

impl<P> Stream for PageInfoStream<P>
where
    P: DataProvider,
{
    type Item = Result<Pair<PageInfo>, SolverError<TreeSolverError<P>>>;

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

unsafe impl<P> Send for PageInfoStream<P>
where
    P: DataProvider,
    <P as DataProvider>::PageInfoRawStream: Send,
    <P as DataProvider>::Error: Send,
{}

pub(super) fn page_info_from_node<'p, P>(provider: P, node: &Node, _default_limit: NumberOrInf<usize>, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> PageInfoStream<P>
where
    P: DataProvider + Clone + Send + 'p,
    <P as DataProvider>::Error: Send + 'p,
    <P as DataProvider>::PageInfoRawStream: Send + 'p,
    <P as DataProvider>::LinksStream: Send + 'p,
    <P as DataProvider>::BacklinksStream: Send + 'p,
    <P as DataProvider>::EmbedsStream: Send + 'p,
    <P as DataProvider>::CategoryMembersStream: Send + 'p,
    <P as DataProvider>::PrefixStream: Send + 'p,
{
    let titles_raw = node.get_titles().to_owned();
    PageInfoStream::new(provider, titles_raw, node.get_span(), warning_sender)
}
