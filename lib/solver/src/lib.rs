//! Page List Bot solver.

// pub mod builder;
pub mod attr;
pub mod error;
// pub mod streams;
pub mod streamsv2;

// re-exports from core
// pub use crate::streams::SolverStream;
pub use crate::error::{RuntimeWarning, RuntimeError, SemanticError};
pub use crate::streamsv2::from_expr;

pub type SolverResult<P> = trio_result::TrioResult<provider::PageInfo, RuntimeWarning<P>, RuntimeError<P>>;
