// This file re-exports the public API of the dory module and also exposes
// test-only helpers when the `test` cfg flag is active.

mod dory_commitment;
mod dory_commitment_evaluation_proof;
mod dory_field_deserialization;
mod dory_inner_product_proofs;
mod dory_messages;
mod dory_public_setup;
mod dory_reduction;
mod dory_reduction_helper;
mod dory_scalar;
mod dory_structure;
mod dory_vmv_helper;
mod dynamic_dory_commitment;
mod dynamic_dory_commitment_evaluation_proof;
mod dynamic_dory_helper;
mod dynamic_dory_structure;
mod extended_state;
mod f_hat_commitment;
mod f_hat_scalar;
mod g1_conversions;
mod g2_conversions;
mod gt_element;
mod pairings;
mod prover_state;
mod scalar_product_proof;
mod setup;
mod state;
mod verifier_state;
mod vmv_state;

#[cfg(test)]
pub(crate) mod test_setup;

#[cfg(test)]
mod dory_commitment_evaluation_proof_test;
#[cfg(test)]
mod dory_commitment_test;
#[cfg(test)]
mod dynamic_dory_commitment_evaluation_proof_test;
#[cfg(test)]
mod dynamic_dory_commitment_test;
#[cfg(test)]
mod setup_test;

pub use dory_commitment::DoryCommitment;
pub use dory_commitment_evaluation_proof::DoryEvaluationProof;
pub use dory_scalar::DoryScalar;
pub use dynamic_dory_commitment::DynamicDoryCommitment;
pub use dynamic_dory_commitment_evaluation_proof::DynamicDoryEvaluationProof;
pub use gt_element::GtElement;
pub use setup::{ProverSetup, PublicParameters, VerifierSetup};
