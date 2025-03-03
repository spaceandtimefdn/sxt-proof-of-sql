//! This module contains the logical plan representation used in both proof generation and postprocessing.
mod error;
pub use error::LogicalPlanError;
mod expr;
pub use expr::{BinaryOperator, Expr};
mod plan;
pub use plan::LogicalPlan;
mod ast_to_plan;
