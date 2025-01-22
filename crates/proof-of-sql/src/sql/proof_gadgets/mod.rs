//! This module contains shared proof logic for multiple `ProofExpr` / `ProofPlan` implementations.
mod membership_check;
mod shift;
#[allow(unused_imports, dead_code)]
use membership_check::{
    final_round_evaluate_membership_check, first_round_evaluate_membership_check,
    verify_membership_check,
};
#[cfg(test)]
mod membership_check_test;
#[allow(unused_imports, dead_code)]
use shift::{final_round_evaluate_shift, first_round_evaluate_shift, verify_shift};
#[cfg(test)]
mod shift_test;
mod sign_expr;
pub(crate) use sign_expr::{prover_evaluate_sign, result_evaluate_sign, verifier_evaluate_sign};
#[cfg(all(test, feature = "blitzar"))]
pub(crate) mod range_check;
#[cfg(all(test, feature = "blitzar"))]
pub(crate) mod range_check_test;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;
