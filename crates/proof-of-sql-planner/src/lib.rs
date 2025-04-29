//! This crate converts a `DataFusion` `LogicalPlan` to a `ProofPlan` and `Postprocessing`
#![cfg_attr(test, expect(clippy::missing_panics_doc))]
extern crate alloc;
mod aggregate;
pub(crate) use aggregate::{aggregate_function_to_proof_expr, AggregateFunc};
mod context;
pub use context::PoSqlContextProvider;
#[cfg(test)]
pub(crate) use context::PoSqlTableSource;
mod conversion;
pub use conversion::{
    get_table_refs_from_statement, sql_to_proof_plans, sql_to_proof_plans_with_postprocessing,
    wasm_friendly_sql_to_proof_plans, optimizer
};
#[cfg(test)]
mod df_util;
mod expr;
pub use expr::expr_to_proof_expr;
mod error;
pub use error::{PlannerError, PlannerResult};
mod plan;
/// Proof of SQL Postprocessing. Used when the last step of the logical plan is an unprovable projection.
pub mod postprocessing;
pub use plan::logical_plan_to_proof_plan;
mod proof_plan_with_postprocessing;
pub use proof_plan_with_postprocessing::{
    logical_plan_to_proof_plan_with_postprocessing, ProofPlanWithPostprocessing,
};
mod util;
pub use util::column_fields_to_schema;
pub(crate) use util::{
    column_to_column_ref, placeholder_to_placeholder_expr, scalar_value_to_literal_value,
    schema_to_column_fields, table_reference_to_table_ref,
};
