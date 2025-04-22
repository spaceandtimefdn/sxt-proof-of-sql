//! This module contains the main logic for Proof of SQL.

mod error;
pub mod parse;
/// [`AnalyzeError`] temporarily exists until we switch to using Datafusion Analyzer to handle type checking.
pub use error::{AnalyzeError, AnalyzeResult};
pub mod postprocessing;
pub mod proof;
pub mod proof_exprs;
pub mod proof_gadgets;
pub mod proof_plans;
mod scale;
pub use scale::scale_cast_binary_op;
