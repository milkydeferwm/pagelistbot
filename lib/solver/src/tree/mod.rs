//! Solver by converting the AST directly to nested futures.

use core::fmt::{self, Debug, Display, Formatter};
use crate::{Answer, Solver, SolverError};
use futures::{StreamExt, TryStreamExt, stream::BoxStream, channel::mpsc::{unbounded, UnboundedSender}};
use interface::types::ast::{Node, Expr, NumberOrInf};
use provider::{DataProvider, Pair, PageInfo, PageInfoError};
use std::{error::Error, collections::BTreeSet};

mod setop;
mod toggle;
mod pageinfo;
mod simplequery;
mod categorymembers;

pub struct TreeSolver<P>
where
    P: DataProvider + Clone,
{
    provider: P,
    default_limit: NumberOrInf<usize>,
}

type DynamicFalliablePageInfoPairStream<'a, P> = BoxStream<'a, Result<Pair<PageInfo>, SolverError<TreeSolverError<P>>>>;
const PROCESS_LIMIT: usize = 1;

/// dynamic dispatcher for node-to-stream
fn dispatch_node<'p, P>(provider: P, node: &Node, default_limit: NumberOrInf<usize>, warning_sender: UnboundedSender<SolverError<TreeSolverError<P>>>) -> DynamicFalliablePageInfoPairStream<'p, P>
where
    P: DataProvider + Clone + Send + 'p,
    <P as DataProvider>::Error: Send + 'p,
    <P as DataProvider>::PageInfoRawStream: Send,
    <P as DataProvider>::LinksStream: Send,
    <P as DataProvider>::BacklinksStream: Send,
    <P as DataProvider>::EmbedsStream: Send,
    <P as DataProvider>::CategoryMembersStream: Send,
    <P as DataProvider>::PrefixStream: Send,
{
    match node.get_expr() {
        Expr::Page { .. } => Box::pin(pageinfo::page_info_from_node(provider, node, default_limit, warning_sender)),
        Expr::Intersection { .. } => Box::pin(setop::intersection_from_node(provider, node, default_limit, warning_sender)),
        Expr::Union { .. } => Box::pin(setop::union_from_node(provider, node, default_limit, warning_sender)),
        Expr::Difference { .. } => Box::pin(setop::difference_from_node(provider, node, default_limit, warning_sender)),
        Expr::Xor { .. } => Box::pin(setop::xor_from_node(provider, node, default_limit, warning_sender)),
        Expr::Link { .. } => Box::pin(simplequery::links_from_node(provider, node, default_limit, PROCESS_LIMIT, warning_sender)),
        Expr::BackLink { .. } => Box::pin(simplequery::backlinks_from_node(provider, node, default_limit, PROCESS_LIMIT, warning_sender)),
        Expr::Embed { .. } => Box::pin(simplequery::embeds_from_node(provider, node, default_limit, PROCESS_LIMIT, warning_sender)),
        Expr::InCategory { .. } => Box::pin(categorymembers::category_members_from_node(provider, node, default_limit, PROCESS_LIMIT, warning_sender)),
        Expr::Prefix { .. } => Box::pin(simplequery::prefix_from_node(provider, node, default_limit, PROCESS_LIMIT, warning_sender)),
        Expr::Toggle { .. } => Box::pin(toggle::toggle_from_node(provider, node, default_limit, warning_sender)),
    }
}

impl<P> TreeSolver<P>
where
    P: DataProvider + Clone,
{
    pub fn new(provider: P, default_limit: NumberOrInf<usize>) -> Self {
        Self {
            provider,
            default_limit,
        }
    }
}

impl<'s, 'p, P> Solver<'s, P> for TreeSolver<P>
where
    P: DataProvider + Clone + Debug + Send + 'p,
    <P as DataProvider>::Error: Send + 'p,
    <P as DataProvider>::PageInfoRawStream: Send + 'p,
    <P as DataProvider>::LinksStream: Send + 'p,
    <P as DataProvider>::BacklinksStream: Send + 'p,
    <P as DataProvider>::EmbedsStream: Send + 'p,
    <P as DataProvider>::CategoryMembersStream: Send + 'p,
    <P as DataProvider>::PrefixStream: Send + 'p,
{
    type InnerError = TreeSolverError<P>;

    async fn solve<'q>(&'s self, ast: &'q Node) -> Result<Answer<TreeSolverError<P>>, SolverError<TreeSolverError<P>>> {
        let (send, recv) = unbounded::<SolverError<TreeSolverError<_>>>();
        let stream = dispatch_node(self.provider.clone(), ast, self.default_limit, send);
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

#[derive(Debug)]
pub enum TreeSolverError<P>
where
    P: DataProvider,
{
    Provider(<P as DataProvider>::Error),
    PageInfo(PageInfoError),
    ResultLimitExceeded(NumberOrInf<usize>),
    ProcessLimitExceeded(usize),
}

impl<P> Display for TreeSolverError<P>
where
    P: DataProvider,
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
    P: DataProvider + Debug,
{}

unsafe impl<P> Send for TreeSolverError<P>
where
    P: DataProvider,
    <P as DataProvider>::Error: Send
{}
