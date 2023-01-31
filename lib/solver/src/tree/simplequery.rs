//! Link operation.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::{TreeSolverError, DynamicFalliablePageInfoPairStream};

use futures::{stream::{TryStream, Empty, self}, Stream, channel::mpsc::UnboundedSender, future::Either};
use interface::types::ast::{Node, Span, NumberOrInf, Modifier};
use pin_project::pin_project;
use provider::{Pair, PageInfo, DataProvider, core::{PageInfoProvider, LinksProvider, BackLinksProvider, EmbedsProvider, CategoryMembersProvider, PrefixProvider}};

macro_rules! simple_query {
    ($vis:vis, $name:ident, $trait:ident, $trait_method:ident, $from_node:ident) => {
        #[pin_project]
        #[must_use = "streams do nothing unless you poll them"]
        $vis struct $name<F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<<P as $trait>::Error>>>,
            P: $trait,
        {
            #[pin] source: F,
            #[pin] workload: Either<Empty<<<P as $trait>::OutputStream as Stream>::Item>, <P as $trait>::OutputStream>,
            provider: P,
            modifier: Modifier,
            span: Span,
            result_limit: NumberOrInf<usize>,
            process_limit: usize,
            has_work: bool,
            processed: usize,
            ready: usize,
            warning_sender: UnboundedSender<SolverError<TreeSolverError<<P as $trait>::Error>>>,
        }

        impl<F, P> $name<F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<<P as $trait>::Error>>>,
            P: $trait,
        {
            pub fn new(stream: F, provider: P, modifier: Modifier, span: Span, result_limit: NumberOrInf<usize>, process_limit: usize, warning_sender: UnboundedSender<SolverError<TreeSolverError<<P as $trait>::Error>>>) -> Self {
                Self {
                    source: stream,
                    workload: Either::Left(stream::empty()),
                    provider,
                    modifier,
                    span,
                    result_limit,
                    process_limit,
                    has_work: false,
                    processed: 0,
                    ready: 0,
                    warning_sender,
                }
            }
        }

        impl<F, P> Stream for $name<F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<<P as $trait>::Error>>>,
            P: $trait + Clone,
        {
            type Item = Result<Pair<PageInfo>, SolverError<TreeSolverError<<P as $trait>::Error>>>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                loop {
                    if *(this.has_work) {
                        // do work
                        let poll_result = this.workload.as_mut().try_poll_next(cx);
                        match poll_result {
                            Poll::Pending => {
                                return Poll::Pending
                            },
                            Poll::Ready(None) => {
                                // this workload is over
                                this.workload.set(Either::Left(stream::empty()));
                                *(this.has_work) = false;
                            },
                            Poll::Ready(Some(Err(e))) => {
                                // poll failed
                                return Poll::Ready(Some(Err(SolverError {
                                    span: *(this.span),
                                    content: TreeSolverError::Provider(e),
                                })));
                            },
                            Poll::Ready(Some(Ok(page_pair))) => {
                                *(this.ready) += 1;
                                if NumberOrInf::Finite(*(this.ready)) > *(this.result_limit) {
                                    // result output limit exceeded. send a warning.
                                    let warning = SolverError {
                                        span: *(this.span),
                                        content: TreeSolverError::ResultLimitExceeded(*(this.result_limit)),
                                    };
                                    this.warning_sender.unbounded_send(warning).expect("cannot send warning");
                                    return Poll::Ready(None);
                                } else {
                                    return Poll::Ready(Some(Ok(page_pair)));
                                }
                            }
                        }
                    } else {
                        // no work, waiting for source.
                        let source_result = this.source.as_mut().try_poll_next(cx);
                        match source_result {
                            Poll::Pending => {
                                return Poll::Pending;
                            },
                            Poll::Ready(None) => {
                                // no more source, the stream is over
                                return Poll::Ready(None);
                            },
                            Poll::Ready(Some(Err(e))) => {
                                // source poll failed
                                return Poll::Ready(Some(Err(e)));
                            },
                            Poll::Ready(Some(Ok(item))) => {
                                // have source title. to process it.
                                *(this.processed) += 1;
                                if *(this.processed) > *(this.process_limit) {
                                    // process count limit lit. send a warning.
                                    let warning = SolverError {
                                        span: *(this.span),
                                        content: TreeSolverError::ProcessLimitExceeded(*(this.process_limit)),
                                    };
                                    this.warning_sender.unbounded_send(warning).expect("cannot send warning");
                                    return Poll::Ready(None);
                                } else {
                                    // set up new stream.
                                    let title = item.0.get_title();
                                    match title {
                                        Err(e) => {
                                            // this is in fact infalliable.
                                            return Poll::Ready(Some(Err(SolverError {
                                                span: *(this.span),
                                                content: TreeSolverError::PageInfo(e),
                                            })));
                                        },
                                        Ok(title) => {
                                            let title = title.to_owned();
                                            let provider = (this.provider).to_owned();
                                            // set workload
                                            this.workload.set(Either::Right(provider.$trait_method([title], this.modifier)));
                                            *(this.has_work) = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        unsafe impl<F, P> Send for $name<F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<<P as $trait>::Error>>> + Send,
            P: $trait + Send,
            <P as $trait>::OutputStream: Send,
            <P as $trait>::Error: Send,
        {}

        $vis fn $from_node<'p, P>(provider: P, node: &Node, default_limit: NumberOrInf<usize>, process_limit: usize, warning_sender: UnboundedSender<SolverError<TreeSolverError<<P as DataProvider>::Error>>>) -> $name<DynamicFalliablePageInfoPairStream<'p, <P as DataProvider>::Error>, P>
        where
            P: DataProvider + Clone + Send + 'p,
            <P as DataProvider>::Error: Send + 'p,
            <P as PageInfoProvider>::OutputStream: Send + 'p,
            <P as LinksProvider>::OutputStream: Send + 'p,
            <P as BackLinksProvider>::OutputStream: Send + 'p,
            <P as EmbedsProvider>::OutputStream: Send + 'p,
            <P as CategoryMembersProvider>::OutputStream: Send + 'p,
            <P as PrefixProvider>::OutputStream: Send + 'p,
        {
            let stream = super::dispatch_node(provider.clone(), node.get_child(), default_limit, warning_sender.clone());
            let limit = node.get_modifier().result_limit.unwrap_or(default_limit);
            $name::new(stream, provider, node.get_modifier().to_owned(), node.get_span(), limit, process_limit, warning_sender)
        }
    }
}

simple_query!(pub(super), LinksStream, LinksProvider, get_links, links_from_node);
simple_query!(pub(super), BacklinksStream, BackLinksProvider, get_backlinks, backlinks_from_node);
simple_query!(pub(super), EmbedsStream, EmbedsProvider, get_embeds, embeds_from_node);
simple_query!(pub(super), PrefixStream, PrefixProvider, get_prefix, prefix_from_node);
