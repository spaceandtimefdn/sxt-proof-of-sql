extern crate alloc;

mod error;
pub use error::{EVMProofPlanError, EVMProofPlanResult};
mod exprs;
pub use exprs::EVMDynProofExpr;
mod plans;
mod proof_plan;
#[cfg(test)]
mod tests;

pub use proof_plan::EVMProofPlan;

#[cfg(test)]
mod evm_tests;