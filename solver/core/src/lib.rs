//! Core of Page List Bot solver family.
//! 
//! This crate defines the traits that other solvers should comply to, 
//! also provides other crates a layer of abstract, so that other parts 
//! of the solver can easily switch between implementations.

use std::collections::BTreeSet;
use mwtitle::Title;
use pagelistbot_parser::ast::Node;

/// Trait for all solver implementations.
#[async_trait::async_trait]
pub trait Solver<'a> {
    async fn solve(&'a self, ast: &'a Node) -> Result<(BTreeSet<Title>, Vec<Error<'a>>), Error<'a>>;
}

/// Common error type for all solver implementations.
#[derive(Debug)]
pub struct Error<'a> {
    pub node: &'a Node,
    pub content: Box<dyn std::error::Error + Send + Sync>,
}

unsafe impl<'a> Send for Error<'a> {}
