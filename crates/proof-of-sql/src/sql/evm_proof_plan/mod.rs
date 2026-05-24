mod error;
pub(crate) use error::{EVMProofPlanError, EVMProofPlanResult};
#[cfg(test)]
mod error_test;
mod exprs;
pub(crate) use exprs::EVMDynProofExpr;
mod plans;
#[cfg(test)]
mod plans_test;
mod proof_plan;
#[cfg(test)]
mod proof_plan_test;
#[cfg(test)]
mod tests;

pub use proof_plan::EVMProofPlan;
