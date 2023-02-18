//! Solver by converting the AST directly to nested futures.

use core::{fmt::{self, Debug, Display, Formatter}, pin::Pin, task::{Context, Poll}};
use crate::{Answer, Solver, SolverError};
use futures::{StreamExt, TryStreamExt, channel::mpsc::{unbounded, UnboundedSender}, Stream};
use interface::types::ast::{Node, Expr, NumberOrInf};
use provider::{DataProvider, Pair, PageInfo, PageInfoError};
use std::{error::Error, collections::BTreeSet};

mod setop;
mod toggle;
mod pageinfo;
mod simplequery;
mod categorymembers;

type PinnedUniversalStream<'p, P> = Pin<Box<UniversalStream<'p, P>>>;

#[pin_project::pin_project(project = UniversalStreamProj)]
#[non_exhaustive]
enum UniversalStream<'p, P>
where
    P: DataProvider,
{
    /// Stream by pageinfo
    PageInfo(#[pin] pageinfo::PageInfoStream<'p, P>),
    // set operations
    /// Stream by intersection operation
    Intersection(#[pin] setop::IntersectionStream<'p, PinnedUniversalStream<'p, P>, PinnedUniversalStream<'p, P>, P>),
    /// Stream by union operation
    Union(#[pin] setop::UnionStream<'p, PinnedUniversalStream<'p, P>, PinnedUniversalStream<'p, P>, P>),
    /// Stream by difference operation
    Difference(#[pin] setop::DifferenceStream<'p, PinnedUniversalStream<'p, P>, PinnedUniversalStream<'p, P>, P>),
    /// Stream by xor operation
    Xor(#[pin] setop::XorStream<'p, PinnedUniversalStream<'p, P>, PinnedUniversalStream<'p, P>, P>),
    // unary operations
    /// Stream by links operation
    Links(#[pin]simplequery::LinksStream<'p, PinnedUniversalStream<'p, P>, P>),
    /// Stream by backlinks operation
    Backlinks(#[pin] simplequery::BacklinksStream<'p, PinnedUniversalStream<'p, P>, P>),
    /// Stream by embeds operation
    Embeds(#[pin] simplequery::EmbedsStream<'p, PinnedUniversalStream<'p, P>, P>),
    /// Stream by perfix operation
    Prefix(#[pin] simplequery::PrefixStream<'p, PinnedUniversalStream<'p, P>, P>),
    /// Stream by incat operation
    CategoryMembers(#[pin] categorymembers::CategoryMembersStream<'p, PinnedUniversalStream<'p, P>, P>),
    /// Stream by toggle operation
    Toggle(#[pin] toggle::ToggleStream<'p, PinnedUniversalStream<'p, P>, P>),
}

impl<'p, P> Stream for UniversalStream<'p, P>
where
    P: DataProvider,
{
    type Item = Result<Pair<PageInfo>, SolverError<TreeSolver<'p, P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            UniversalStreamProj::PageInfo(s) => s.poll_next(cx),
            UniversalStreamProj::Intersection(s) => s.poll_next(cx),
            UniversalStreamProj::Union(s) => s.poll_next(cx),
            UniversalStreamProj::Difference(s) => s.poll_next(cx),
            UniversalStreamProj::Xor(s) => s.poll_next(cx),
            UniversalStreamProj::Links(s) => s.poll_next(cx),
            UniversalStreamProj::Backlinks(s) => s.poll_next(cx),
            UniversalStreamProj::Embeds(s) => s.poll_next(cx),
            UniversalStreamProj::Prefix(s) => s.poll_next(cx),
            UniversalStreamProj::CategoryMembers(s) => s.poll_next(cx),
            UniversalStreamProj::Toggle(s) => s.poll_next(cx),
        }
    }
}

pub struct TreeSolver<'p, P>
where
    P: DataProvider,
{
    provider: &'p P,
    default_limit: NumberOrInf<usize>,
}

impl<'p, P> TreeSolver<'p, P>
where
    P: DataProvider,
{
    pub fn new(provider: &'p P, default_limit: NumberOrInf<usize>) -> Self {
        Self {
            provider,
            default_limit,
        }
    }

    fn future_from_node<'q>(&self, node: &'q Node, warning_sender: UnboundedSender<SolverError<TreeSolver<'p, P>>>) -> UniversalStream<'p, P> {
        const PROCESS_LIMIT: usize = 1;
        match node.get_expr() {
            Expr::Page { titles } => {
                let st = pageinfo::PageInfoStream::new(self.provider, titles.to_owned(), node.get_span(), warning_sender);
                UniversalStream::PageInfo(st)
            },
            Expr::Intersection { set1, set2 } => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set1, warning_sender)
                };
                let st2 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set2, warning_sender)
                };
                let st = setop::IntersectionStream::new(Box::pin(st1), Box::pin(st2), node.get_span(), warning_sender);
                UniversalStream::Intersection(st)
            },
            Expr::Union { set1, set2 } => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set1, warning_sender)
                };
                let st2 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set2, warning_sender)
                };
                let st = setop::UnionStream::new(Box::pin(st1), Box::pin(st2), node.get_span(), warning_sender);
                UniversalStream::Union(st)
            },
            Expr::Difference { set1, set2 } => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set1, warning_sender)
                };
                let st2 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set2, warning_sender)
                };
                let st = setop::DifferenceStream::new(Box::pin(st1), Box::pin(st2), node.get_span(), warning_sender);
                UniversalStream::Difference(st)
            },
            Expr::Xor { set1, set2 } => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set1, warning_sender)
                };
                let st2 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(set2, warning_sender)
                };
                let st = setop::XorStream::new(Box::pin(st1), Box::pin(st2), node.get_span(), warning_sender);
                UniversalStream::Xor(st)
            },
            Expr::Link { target, modifier } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = simplequery::LinksStream::new(
                    Box::pin(st0),
                    self.provider,
                    modifier.to_owned(),
                    node.get_span(),
                    modifier.result_limit.unwrap_or(self.default_limit),
                    PROCESS_LIMIT,
                    warning_sender,
                );
                UniversalStream::Links(st)
            },
            Expr::BackLink { target, modifier } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = simplequery::BacklinksStream::new(
                    Box::pin(st0),
                    self.provider,
                    modifier.to_owned(),
                    node.get_span(),
                    modifier.result_limit.unwrap_or(self.default_limit),
                    PROCESS_LIMIT,
                    warning_sender,
                );
                UniversalStream::Backlinks(st)
            },
            Expr::Embed { target, modifier } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = simplequery::EmbedsStream::new(
                    Box::pin(st0),
                    self.provider,
                    modifier.to_owned(),
                    node.get_span(),
                    modifier.result_limit.unwrap_or(self.default_limit),
                    PROCESS_LIMIT,
                    warning_sender,
                );
                UniversalStream::Embeds(st)
            },
            Expr::InCategory { target, modifier } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = categorymembers::CategoryMembersStream::new(
                    Box::pin(st0),
                    self.provider,
                    modifier.to_owned(),
                    node.get_span(),
                    modifier.result_limit.unwrap_or(self.default_limit),
                    PROCESS_LIMIT,
                    warning_sender,
                );
                UniversalStream::CategoryMembers(st)
            },
            Expr::Prefix { target, modifier } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = simplequery::PrefixStream::new(
                    Box::pin(st0),
                    self.provider,
                    modifier.to_owned(),
                    node.get_span(),
                    modifier.result_limit.unwrap_or(self.default_limit),
                    PROCESS_LIMIT,
                    warning_sender,
                );
                UniversalStream::Prefix(st)
            },
            Expr::Toggle { target } => {
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    self.future_from_node(target, warning_sender)
                };
                let st = toggle::ToggleStream::new(
                    Box::pin(st0),
                    node.get_span(),
                    warning_sender,
                );
                UniversalStream::Toggle(st)
            },
        }
    }
}

impl<'p, P> Solver for TreeSolver<'p, P>
where
    P: DataProvider,
{
    type InnerError = TreeSolverError<P>;

    async fn solve<'q>(&self, ast: &'q Node) -> Result<Answer<TreeSolver<'p, P>>, SolverError<TreeSolver<'p, P>>> {
        let (send, recv) = unbounded::<SolverError<TreeSolver<'p, P>>>();
        let stream = self.future_from_node(ast, send);
        // collect!
        let result_titles = {
            let result_pagepairs = stream.try_collect::<BTreeSet<_>>().await?;
            result_pagepairs.into_iter().map(|(this, _)| this.get_title().unwrap().to_owned()).collect::<BTreeSet<_>>()
        };
        let warnings = recv.collect::<Vec<_>>().await;
        Ok(Answer {
            titles: result_titles,
            warnings,
        })
    }
}

pub enum TreeSolverError<P>
where
    P: DataProvider,
{
    Provider(<P as DataProvider>::Error),
    PageInfo(PageInfoError),
    ResultLimitExceeded(NumberOrInf<usize>),
    ProcessLimitExceeded(usize),
}

impl<P> Debug for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(arg0) => f.debug_tuple("Provider").field(arg0).finish(),
            Self::PageInfo(arg0) => f.debug_tuple("PageInfo").field(arg0).finish(),
            Self::ResultLimitExceeded(arg0) => f.debug_tuple("ResultLimitExceeded").field(arg0).finish(),
            Self::ProcessLimitExceeded(arg0) => f.debug_tuple("ProcessLimitExceeded").field(arg0).finish(),
        }
    }
}

impl<P> Display for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(e) => write!(f, "data provider error: {e}"),
            Self::PageInfo(e) => write!(f, "cannot extract page information: {e}"),
            Self::ResultLimitExceeded(lim) => write!(f, "result output limit `{lim}` exceeded, output is truncated"),
            Self::ProcessLimitExceeded(lim) => write!(f, "more than `{lim}` items are sent to this operation, output is truncated")
        }
    }
}

impl<P> Clone for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Provider(e) => Self::Provider(e.clone()),
            Self::PageInfo(e) => Self::PageInfo(*e),
            Self::ResultLimitExceeded(lim) => Self::ResultLimitExceeded(*lim),
            Self::ProcessLimitExceeded(lim) => Self::ProcessLimitExceeded(*lim),
        }
    }
}

impl<P> Error for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Error,
{}

unsafe impl<P> Send for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Send
{}
