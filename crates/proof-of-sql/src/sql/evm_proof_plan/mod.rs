mod error;
pub(crate) use error::{EVMProofPlanError, EVMProofPlanResult};
mod exprs;
pub(crate) use exprs::EVMDynProofExpr;
mod plans;
mod proof_plan;
#[cfg(test)]
mod tests;

/// Companion tests for [`EVMProofPlan`] — see `proof_plan_test.rs` for
/// coverage of `into_inner` and the delegated `ProofPlan` / `ProverEvaluate`
/// trait methods (the sibling `tests.rs` covers serialize/deserialize).
#[cfg(test)]
mod proof_plan_test;

pub use proof_plan::EVMProofPlan;
