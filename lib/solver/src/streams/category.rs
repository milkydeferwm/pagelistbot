//! fetch the pages in a given category, optional depth hunting.

use ast::Span;
use core::{mem, pin::Pin, task::{Context, Poll}};
use std::{collections::BTreeSet, vec::IntoIter};
use crate::{SolverResult, RuntimeError, RuntimeWarning};


use futures::{stream::{Flatten, Iter}, Stream, ready};
use intorinf::IntOrInf;
use mwtitle::Title;
use provider::{DataProvider, CategoryMembersConfig};
use trio_result::TrioResult;

#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
struct CategoryMembersStreamSingle<P>
where
    P: DataProvider,
{
    provider: P,
    config: CategoryMembersConfig,
    span: Span,
    max_depth: IntOrInf,

    // internal
    #[pin] stream: Option<Flatten<Iter<IntoIter<<P as DataProvider>::CategoryMembersStream>>>>,
    visited: BTreeSet<Title>,
    queue: BTreeSet<Title>,
    curr_depth: IntOrInf,
}

impl<P> CategoryMembersStreamSingle<P>
where
    P: DataProvider,
{
    fn new(title: Title, provider: P, config: CategoryMembersConfig, max_depth: IntOrInf, span: Span) -> Self {
        Self {
            provider,
            config,
            span,
            max_depth,

            stream: None,
            visited: BTreeSet::from_iter([title.clone()]),
            queue: BTreeSet::from_iter([title]),
            curr_depth: IntOrInf::Int(0),
        }
    }
}

impl<P> Stream for CategoryMembersStreamSingle<P>
where
    P: DataProvider,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                // current stream is active.
                if let Some(item) = ready!(s.poll_next(cx)) {
                    // current stream is not exhausted.
                    match item {
                        TrioResult::Ok(info) => {
                            let t = match info.get_title() {
                                Ok(t) => t,
                                Err(e) => break Some(TrioResult::Err(RuntimeError::PageInfo { span: *this.span, error: e })),
                            };
                            // determine whether to add this to visit queue.
                            if t.is_category() && !this.visited.contains(t) && *this.curr_depth < *this.max_depth {
                                this.queue.insert(t.to_owned());
                                this.visited.insert(t.to_owned());
                            }
                            // determine whether to yield this item.
                            if !this.config.namespace.as_ref().is_some_and(|ns| !ns.contains(&t.namespace())) {
                                break Some(TrioResult::Ok(info));
                            }
                        },
                        TrioResult::Warn(w) => break Some(TrioResult::Warn(RuntimeWarning::Provider { span: *this.span, warn: w })),
                        TrioResult::Err(e) => break Some(TrioResult::Err(RuntimeError::Provider { span: *this.span, error: e })),
                    }
                } else {
                    // current stream is exhausted.
                    this.stream.set(None);
                    *this.curr_depth += 1;
                }
            } else if !this.queue.is_empty() {
                // continue next.
                let mut config = this.config.to_owned();
                if *this.curr_depth <= *this.max_depth {
                    if let Some(ns) = &mut config.namespace {
                        ns.insert(14);
                    }
                }
                let queue = mem::take(this.queue);
                let stream = this.provider.get_category_members_multi(queue, &config);
                this.stream.set(Some(stream));
            } else {
                // no next
                break None;
            }
        })
    }
}

#[pin_project::pin_project]
#[must_use = "streams do nothing unless you poll them"]
pub struct CategoryMembersStream<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    #[pin] source: St,
    provider: P,
    config: CategoryMembersConfig,
    span: Span,
    max_depth: IntOrInf,

    #[pin] stream: Option<CategoryMembersStreamSingle<P>>,
}

impl<St, P> CategoryMembersStream<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    pub fn new(source: St, provider: P, config: CategoryMembersConfig, max_depth: IntOrInf, span: Span) -> Self {
        Self {
            source,
            provider,
            config,
            span,
            max_depth,

            stream: None,
        }
    }
}

impl<St, P> Stream for CategoryMembersStream<St, P>
where
    St: Stream<Item=SolverResult<P>>,
    P: DataProvider + Clone,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                // current stream is active.
                if let x @ Some(_) = ready!(s.poll_next(cx)) {
                    // current stream is not exhausted.
                    break x;
                } else {
                    // current stream is exhausted.
                    this.stream.set(None);
                }
            } else if let Some(s) = ready!(this.source.as_mut().poll_next(cx)) {
                // next item is available.
                if let TrioResult::Ok(s) = s {  // getting next item successful.
                    let t = match s.get_title() {
                        Ok(t) => t,
                        Err(w) => break Some(TrioResult::Err(RuntimeError::PageInfo { span: *this.span, error: w })),
                    };
                    let next = CategoryMembersStreamSingle::new(t.to_owned(), this.provider.to_owned(), this.config.to_owned(), *this.max_depth, *this.span);
                    this.stream.set(Some(next));
                } else {    // getting next item failure or warning.
                    break Some(s);
                }
            } else {
                break None;
            }
        })
    }
}
