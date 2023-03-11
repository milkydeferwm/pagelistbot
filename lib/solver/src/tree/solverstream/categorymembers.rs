//! fetch the pages in a given category, optional depth hunting.
use core::{pin::Pin, task::{Context, Poll}};
use std::collections::BTreeSet;
use crate::SolverError;
use super::{TreeSolverError, TreeSolver};

use ast::Span;
use futures::{stream::{TryStream, Empty, self}, Stream, channel::mpsc::UnboundedSender, future::Either};
use intorinf::IntOrInf;
use mwtitle::Title;
use provider::{Pair, PageInfo, DataProvider, CategoryMembersConfig};


#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub(crate) struct CategoryMembersStream<'e, F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
    P: DataProvider + Clone,
{
    #[pin] source: F,
    #[pin] workload: Either<Empty<<<P as DataProvider>::CategoryMembersStream as Stream>::Item>, CategoryMembersStreamOne<P>>,
    provider: P,
    config: CategoryMembersConfig,
    span: Span<'e>,
    result_limit: IntOrInf,
    produced_pages: BTreeSet<Title>,
    count: IntOrInf,
    max_depth: IntOrInf,
    warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
}

impl<'e, F, P> CategoryMembersStream<'e, F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
    P: DataProvider + Clone,
{
    pub fn new(
        stream: F,
        provider: P,
        config: CategoryMembersConfig,
        span: Span<'e>,
        result_limit: IntOrInf,
        max_depth: IntOrInf,
        warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>
    ) -> Self
    {
        Self {
            source: stream,
            workload: Either::Left(stream::empty()),
            provider,
            config,
            span,
            result_limit,
            count: IntOrInf::Int(0),
            warning_sender,

            produced_pages: BTreeSet::new(),
            max_depth,
        }
    }
}

impl<'e, F, P> Stream for CategoryMembersStream<'e, F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
    P: DataProvider + Clone,
{
    type Item = Result<Pair<PageInfo>, SolverError<'e, TreeSolver<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        match *this.workload {
            Either::Left(_) => {
                // poll source stream.
                let source_poll_result = this.source.as_mut().try_poll_next(cx)?;
                match source_poll_result {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Ready(Some(pair)) => {
                        let title = pair.0.get_title().unwrap().to_owned();    // SAFETY: infalliable.
                        let st = CategoryMembersStreamOne::new(title, this.provider.clone(), this.config.clone(), *this.max_depth);
                        this.workload.set(Either::Right(st));
                        Poll::Pending
                    },
                }
            },
            Either::Right(_) => {
                // poll data stream.
                let data_poll_result = this.workload.as_mut().try_poll_next(cx);
                match data_poll_result {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(None) => {
                        this.workload.set(Either::Left(stream::empty()));
                        Poll::Pending
                    },
                    Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(SolverError::from_solver_error(*this.span, TreeSolverError::Provider(e))))),
                    Poll::Ready(Some(Ok(pair))) => {
                        let title = pair.0.get_title().unwrap().to_owned();    // SAFETY: infalliable.
                        *this.count += 1;
                        if *this.count >= *this.result_limit {
                            // send warning.
                            let warning = SolverError::from_solver_error(*this.span, TreeSolverError::ResultLimitExceeded(*this.result_limit));
                            this.warning_sender.unbounded_send(warning).expect("cannot send warning");
                            Poll::Ready(None)
                        } else if this.produced_pages.contains(&title) {
                            Poll::Pending
                        } else {
                            this.produced_pages.insert(title);
                            Poll::Ready(Some(Ok(pair)))
                        }
                    }
                }
            }
        }
    }
}

unsafe impl<'e, F, P> Send for CategoryMembersStream<'e, F, P>
where
    F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>> + Send,
    P: DataProvider + Clone + Send,
    <P as DataProvider>::CategoryMembersStream: Send,
{}

/// Stream object for CategoryMembers query of one item.
#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
struct CategoryMembersStreamOne<P>
where
    P: DataProvider,
{
    /// configuration object.
    config: CategoryMembersConfig,
    /// provider object that is used to spawn new workloads.
    provider: P,
    /// maximum tree search depth.
    max_depth: IntOrInf,

    // internal states.
    #[pin] workload: <P as DataProvider>::CategoryMembersStream,
    visited_categories: BTreeSet<Title>,
    category_backlog: BTreeSet<Title>,
    current_depth: IntOrInf,
}

impl<P> CategoryMembersStreamOne<P>
where
    P: DataProvider,
{
    fn new(title: Title, provider: P, config: CategoryMembersConfig, max_depth: IntOrInf) -> Self {
        // kick off the stream.
        let visited_categories = {
            let title = title.clone();
            BTreeSet::from([title])
        };
        let workload = provider.get_category_members(vec![title], &config);
        Self {
            config,
            provider,
            max_depth,
            workload,
            visited_categories,
            category_backlog: BTreeSet::new(),
            current_depth: IntOrInf::Int(0),
        }
    }
}

impl<P> Stream for CategoryMembersStreamOne<P>
where
    P: DataProvider,
{
    type Item = Result<Pair<PageInfo>, <P as DataProvider>::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let poll_result = this.workload.as_mut().try_poll_next(cx)?;
        match poll_result {
            Poll::Pending => {  // we haven't got an item from workload.
                Poll::Pending
            },
            Poll::Ready(None) => {
                if this.category_backlog.is_empty() {
                    Poll::Ready(None)   // we are totally done with this.
                } else {    // continue into next layer.
                    *this.current_depth += 1;
                    let mut config = this.config.clone();
                    if *this.current_depth <= *this.max_depth {
                        if let Some(ns) = &mut config.namespace {
                            ns.insert(14);
                        }
                    }
                    let to_visit = this.category_backlog.clone();
                    this.visited_categories.extend(to_visit);
                    let to_visit = this.category_backlog.clone();
                    this.category_backlog.clear();
                    let workload = this.provider.get_category_members(to_visit, &config);
                    this.workload.set(workload);
                    Poll::Pending
                }
            },
            Poll::Ready(Some(page_pair)) => {
                let title = page_pair.0.get_title().unwrap();   // SAFETY: infalliable
                // do we need to add this to backlog?
                if title.is_category()
                    && *this.current_depth < *this.max_depth
                    && !this.visited_categories.contains(title)
                {
                    this.category_backlog.insert(title.clone());
                }
                // do we need to yield this item?
                if this.config.namespace.is_none()
                    || this.config.namespace.as_ref().is_some_and(|ns| ns.contains(&title.namespace())) {
                    Poll::Ready(Some(Ok(page_pair)))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

unsafe impl<P> Send for CategoryMembersStreamOne<P>
where
    P: DataProvider + Send,
    <P as DataProvider>::CategoryMembersStream: Send,
{}
