//! Solver by converting the AST directly to nested futures.

use ast::Expression;
use crate::{Answer, Solver, SolverError};
use futures::{StreamExt, TryStreamExt, channel::mpsc::unbounded};
use intorinf::IntOrInf;
use provider::{DataProvider, PageInfoError};
use std::{error::Error, collections::BTreeSet};
use thiserror::Error as ThisError;

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
    type Error = TreeSolverError<<P as DataProvider>::Error>;

    async fn solve(&self, ast: &Expression) -> Result<Answer<TreeSolver<P>>, SolverError<TreeSolver<P>>> {
        let (send, recv) = unbounded::<SolverError<TreeSolver<P>>>();
        let stream = UniversalStream::from_expr(ast, self.provider.clone(), self.default_limit, send)?;
        // collect!
        let result_titles = {
            let result_pagepairs = stream.try_collect::<BTreeSet<_>>().await?;
            result_pagepairs.into_iter().map(|this| this.get_title().unwrap().to_owned()).collect::<BTreeSet<_>>()
        };
        let warnings = recv.collect::<Vec<_>>().await;
        Ok(Answer {
            titles: result_titles,
            warnings,
        })
    }
}

#[derive(Debug, ThisError)]
pub enum TreeSolverError<E>
where
    E: Error + 'static,
{
    #[error("data provider error: {0}")]
    Provider(#[source] E),
    #[error("cannot extract page information: {0}")]
    PageInfo(#[from] PageInfoError),
    #[error("result output limit `{0}` exceeded, result may be incomplete")]
    ResultLimitExceeded(IntOrInf),
}

impl<E> Clone for TreeSolverError<E>
where
    E: Error + Clone + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Provider(e) => Self::Provider(e.clone()),
            Self::PageInfo(e) => Self::PageInfo(*e),
            Self::ResultLimitExceeded(lim) => Self::ResultLimitExceeded(*lim),
        }
    }
}

unsafe impl<E> Send for TreeSolverError<E>
where
    E: Error + Send + 'static,
{}
