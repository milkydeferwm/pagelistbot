//! Toggle operation.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::{TreeSolverError, DynamicFalliablePageInfoPairStream};

use futures::{stream::TryStream, Stream, channel::mpsc::UnboundedSender};
use interface::types::ast::{Node, Span, NumberOrInf};
use pin_project::pin_project;
use provider::{Pair, PageInfo, DataProvider};

#[pin_project]
#[must_use = "streams do nothing unless you poll them"]
#[derive(Debug)]
pub(super) struct ToggleStream<F, P>
where
    P: DataProvider,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
{
    #[pin] st: F,
    span: Span,
    warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>,
}

impl<F, P> ToggleStream<F, P>
where
    P: DataProvider,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
{
    pub fn new(stream: F, span: Span, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> Self {
        Self {
            st: stream,
            span,
            warning_sender,
        }
    }
}

impl<F, P> Stream for ToggleStream<F, P>
where
    P: DataProvider,
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
{
    type Item = Result<Pair<PageInfo>, SolverError<TreeSolverError<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            match this.st.as_mut().try_poll_next(cx)? {
                Poll::Ready(r) => {
                    if let Some((this_page, associated_page)) = r {
                        match associated_page.get_title() {
                            Ok(associated_title) => {
                                if associated_title.namespace() >= 0 {
                                    return Poll::Ready(Some(Ok((associated_page, this_page))));
                                }
                                // No page's associated page lies in virtual namespaces.
                                // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
                                // In such cases, we should not yield an item, instead continue the loop.
                            },
                            Err(e) => {
                                return Poll::Ready(Some(Err(SolverError {
                                    span: *(this.span),
                                    content: TreeSolverError::PageInfo(e),
                                })));
                            },
                        }
                    } else {
                        return Poll::Ready(None);
                    }
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

unsafe impl<F, P> Send for ToggleStream<F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>> + Send,
    P: DataProvider,
    <P as DataProvider>::Error: Send,
{}

pub(super) fn toggle_from_node<'p, P>(provider: P, node: &Node, default_limit: NumberOrInf<usize>, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> ToggleStream<DynamicFalliablePageInfoPairStream<'p, P>, P>
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
    let stream = super::dispatch_node(provider, node.get_child(), default_limit, warning_sender.clone());
    ToggleStream::new(stream, node.get_span(), warning_sender)
}
