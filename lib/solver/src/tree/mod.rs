//! Solver by converting the AST directly to nested futures.

use ast::Expression;
use core::fmt::{self, Debug, Display, Formatter};
use crate::{Answer, Solver, SolverError};
use futures::{StreamExt, TryStreamExt, channel::mpsc::unbounded};
use intorinf::IntOrInf;
use provider::{DataProvider, PageInfoError};
use std::{error::Error, collections::BTreeSet};

mod solverstream;
use solverstream::UniversalStream;

pub struct TreeSolver<P>
where
    P: DataProvider,
{
    provider: P,
    default_limit: IntOrInf,
}

impl<P> TreeSolver<P>
where
    P: DataProvider,
{
    pub fn new(provider: P, default_limit: IntOrInf) -> Self {
        Self {
            provider,
            default_limit,
        }
    }
}

impl<'p, P> Solver for TreeSolver<P>
where
    P: DataProvider + Clone,
{
    type Error = TreeSolverError<P>;

    async fn solve<'e>(&self, ast: &Expression<'e>) -> Result<Answer<'e, TreeSolver<P>>, SolverError<'e, TreeSolver<P>>> {
        let (send, recv) = unbounded::<SolverError<'e, TreeSolver<P>>>();
        let stream = UniversalStream::from_expr(ast, self.provider.clone(), self.default_limit, send)?;
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
    ResultLimitExceeded(IntOrInf),
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
    <P as DataProvider>::Error: Send,
{}
