//! Page List Bot solver.
//! 
//! This crate defines the traits that other solvers should comply to, 
//! also provides other crates a layer of abstract, so that other parts 
//! of the solver can easily switch between implementations.
//! 
//! A built-in solver, `RecursiveTreeSolver`, is available through the `recursive-tree` feature gate.

#![allow(incomplete_features)]
#![feature(async_fn_in_trait, is_some_and, type_alias_impl_trait)]

extern crate alloc;

pub mod core;
pub mod cvt_attr;
#[cfg(feature = "tree-future")]
pub mod tree;

// re-exports from core
pub use crate::core::{Solver, Answer, SolverError};
