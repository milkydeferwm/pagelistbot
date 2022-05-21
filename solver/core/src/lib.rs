//! Core of Page List Bot solver family.
//! 
//! This crate defines the traits that other solvers should comply to, 
//! also provides other crates a layer of abstract, so that other parts 
//! of the solver can easily switch between implementations.

use std::collections::HashSet;
use mwtitle::Title;
use pagelistbot_parser::ast::Expr;

/// Trait for all solver implementations.
#[async_trait::async_trait]
pub trait Solver<'a> {
    async fn solve(ast: &'a Expr) -> Result<(HashSet<Title>, Vec<Error<'a>>), Error<'a>>;
}

#[derive(Debug)]
pub struct Error<'a> {
    pub node: &'a Expr,
    pub content: Box<dyn std::error::Error>,
}
