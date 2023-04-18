//! fetch the pages in a given category, optional depth hunting.
use core::{pin::Pin, task::{Context, Poll}};
use std::{collections::BTreeSet, vec::IntoIter};
use crate::SolverError;
use super::{TreeSolverError, TreeSolver, unique::TryUnique, counted::Counted};

use ast::Span;
use futures::{stream::{TryStream, Flatten, Iter}, Stream, channel::mpsc::UnboundedSender, ready, TryStreamExt};
use intorinf::IntOrInf;
use mwtitle::Title;
use provider::{PageInfo, DataProvider, CategoryMembersConfig};

pub(crate) type CategoryMembersStream<St, P>
where
    P: DataProvider + Clone,
    St: Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>,
= impl Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>;

pub(crate) fn make_categorymembers_stream<St, P>(
    stream: St,
    provider: P,
    config: CategoryMembersConfig,
    span: Span,
    limit: IntOrInf,
    max_depth: IntOrInf,
    warning_sender: UnboundedSender<SolverError<TreeSolver<P>>>,
) -> CategoryMembersStream<St, P>
where
    P: DataProvider + Clone,
    St: Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>,
{
    let span_2 = span.clone();
    let stream = stream.map_ok(move |x| {
        let span = span_2.clone();
        CategoryMembersStreamOne::new(x.get_title().unwrap().to_owned(), provider.clone(), config.clone(), max_depth)
        .map_err(move |e| {
            let span = span.clone();
            SolverError::from_solver_error(span, TreeSolverError::Provider(e))
        })
    }).try_flatten();

    TryUnique::new(Counted::new(stream, span, limit, warning_sender))
}

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
    #[pin] workload: Flatten<Iter<IntoIter<<P as DataProvider>::CategoryMembersStream>>>,
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
        let workload = provider.get_category_members_multi([title], &config);
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
    type Item = Result<PageInfo, <P as DataProvider>::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(item) = ready!(this.workload.as_mut().try_poll_next(cx)?) {
                let title = item.get_title().unwrap();   // SAFETY: infalliable
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
                    break Some(Ok(item))
                }
            } else {
                // polling for this layer is complete. ready for next layer.
                if this.category_backlog.is_empty() {
                    // does not have next layer.
                    break None;
                } else {
                    // prepare next layer.
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
                    let workload = this.provider.get_category_members_multi(to_visit, &config);
                    this.workload.set(workload);
                }
            }
        })
    }
}

unsafe impl<P> Send for CategoryMembersStreamOne<P>
where
    P: DataProvider + Send,
    <P as DataProvider>::CategoryMembersStream: Send,
{}
