//! fetch the pages in a given category, optional depth hunting.
use core::{pin::Pin, task::{Context, Poll}};
use std::collections::BTreeSet;
use crate::SolverError;
use super::{TreeSolverError, DynamicFalliablePageInfoPairStream};

use futures::{stream::{TryStream, Empty, self}, Stream, channel::mpsc::UnboundedSender, future::Either};
use interface::types::ast::{Node, Span, NumberOrInf, Modifier};
use mwtitle::Title;
use pin_project::pin_project;
use provider::{Pair, PageInfo, DataProvider};

#[pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub(super) struct CategoryMembersStream<F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
    P: DataProvider,
{
    #[pin] source: F,
    #[pin] workload: Either<Empty<<<P as DataProvider>::CategoryMembersStream as Stream>::Item>, <P as DataProvider>::CategoryMembersStream>,
    provider: P,
    modifier: Modifier,
    span: Span,
    result_limit: NumberOrInf<usize>,
    process_limit: usize,
    has_work: bool,
    processed: usize,
    ready: usize,
    visited_categories: BTreeSet<Title>,
    produced_pages: BTreeSet<Pair<PageInfo>>,
    category_backlog: BTreeSet<Title>,
    current_depth: usize,
    warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>,
}

macro_rules! reset {
    ($proj:ident) => {
        *($proj.current_depth) = 0;
        $proj.visited_categories.clear();
        $proj.category_backlog.clear();
    };
}

macro_rules! prepare_new_workload {
    ($proj:ident, $source:expr) => {
        {
            // prepare to mangle the modifier, if needed.
            let mut modifier = $proj.modifier.clone();
            if NumberOrInf::Finite(*($proj.current_depth)) < modifier.categorymembers_recursion_depth {
                if let Some(ns) = &mut modifier.namespace {
                    ns.insert(14);
                }
            }
            // update some internal states
            *($proj.has_work) = true;
            *($proj.current_depth) += 1;
            let to_visit = $source.clone();
            $proj.visited_categories.extend(to_visit);
            // produce workload
            let to_visit = $source.clone();
            $source.clear();
            let provider = $proj.provider.to_owned();
            provider.get_category_members(to_visit, &modifier)
        }
    };
}

impl<F, P> CategoryMembersStream<F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
    P: DataProvider,
{
    pub fn new(stream: F, provider: P, modifier: Modifier, span: Span, result_limit: NumberOrInf<usize>, process_limit: usize, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> Self {
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

            visited_categories: BTreeSet::new(),
            produced_pages: BTreeSet::new(),
            category_backlog: BTreeSet::new(),
            current_depth: 0,
        }
    }
}

impl<F, P> Stream for CategoryMembersStream<F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>>,
    P: DataProvider + Clone,
{
    type Item = Result<Pair<PageInfo>, SolverError<TreeSolverError<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if *(this.has_work) {
                // poll workload
                let poll_result = this.workload.as_mut().try_poll_next(cx);
                match poll_result {
                    Poll::Pending => {  // we havn't got an item from workload.
                        return Poll::Pending;
                    },
                    Poll::Ready(None) => {
                        // this workload is over. do we have another category in backlog?
                        if !this.category_backlog.is_empty() {
                            let work = prepare_new_workload!(this, this.category_backlog);
                            this.workload.set(Either::Right(work));
                        } else {
                            // the workload for this source title is finished.
                            this.workload.set(Either::Left(stream::empty()));
                            *(this.has_work) = false;
                            reset!(this);
                        }
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
                            // try to inspect the returned page pair.
                            let title = page_pair.0.get_title();
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
                                    // do we need to add this to backlog?
                                    if title.is_category()  // this is a category.
                                       && NumberOrInf::Finite(*(this.current_depth) - 1) < this.modifier.categorymembers_recursion_depth   // current depth value is added by 1 when creating workload. we need to substract it back.
                                       && !this.visited_categories.contains(&title)    // we have not visited this category yet.
                                    {
                                        // add to backlog, backlog
                                        this.category_backlog.insert(title.clone());
                                    }
                                    // do we need to yield this item?
                                    if !this.produced_pages.contains(&page_pair) {
                                        // we have not yielded this before.
                                        // since the workload could use a mangled modifier to include subcats, we need to filter them out.
                                        // no namespace is specified => all namespaces allowed => yield.
                                        // namespace specified, page namespace included in specified namespaces => allowed to yield.
                                        // otherwise => mangled modifier => not yield.
                                        if this.modifier.namespace.is_none() || this.modifier.namespace.as_ref().is_some_and(|ns| ns.contains(&title.namespace())) {
                                            this.produced_pages.insert(page_pair.clone());
                                            return Poll::Ready(Some(Ok(page_pair)));
                                        }
                                    }
                                }
                            }
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
                                    let mut title = vec![title];
                                    let workload = prepare_new_workload!(this, title);
                                    // set workload
                                    this.workload.set(Either::Right(workload));
                                    reset!(this);
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

unsafe impl<F, P> Send for CategoryMembersStream<F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<TreeSolverError<P>>> + Send,
    P: DataProvider + Clone + Send,
    <P as DataProvider>::CategoryMembersStream: Send,
    P: Send,
{}

pub(super) fn category_members_from_node<'p, P>(provider: P, node: &Node, default_limit: NumberOrInf<usize>, process_limit: usize, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> CategoryMembersStream<DynamicFalliablePageInfoPairStream<'p, P>, P>
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
    let stream = super::dispatch_node(provider.clone(), node.get_child(), default_limit, warning_sender.clone());
    let limit = node.get_modifier().result_limit.unwrap_or(default_limit);
    CategoryMembersStream::new(stream, provider, node.get_modifier().to_owned(), node.get_span(), limit, process_limit, warning_sender)
}
