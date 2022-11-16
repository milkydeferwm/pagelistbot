//! Page List Bot solver.
//! 
//! This crate defines the traits that other solvers should comply to, 
//! also provides other crates a layer of abstract, so that other parts 
//! of the solver can easily switch between implementations.
//! 
//! A built-in solver, `RecursiveTreeSolver`, is available through the `recursive-tree` feature gate.

pub mod core;
#[cfg(feature="recursive-tree")]
pub mod recursive_tree;

// re-exports from core
pub use crate::core::{Solver, Answer, Error};
