/// Contains the Dory proof system implementation.
mod dory_commitment_evaluation_proof;
mod dory_commitment_helper;
mod dory_compute_commitments_impl;
mod dory_field_serialization;
mod dory_inner_product_proof;
mod dory_messages;
mod dory_public_setup;
mod dory_reduce_helper;
mod dory_scalar;
mod dory_structure;
mod extended_dory_inner_product_proof;
mod extended_dory_reduce_helper;
mod gs_scalar;
mod pack_scalars;
mod public_parameters;
mod scalar_conversions;
mod setup;
mod state;
mod transpose;

pub use dory_commitment_evaluation_proof::DoryEvaluationProof;
pub use dory_public_setup::{DoryProverPublicSetup, DoryVerifierPublicSetup};
pub use dory_scalar::{DoryScalar, GT};
pub use public_parameters::PublicParameters;
pub use setup::{ProverSetup, VerifierSetup};

#[cfg(test)]
mod test_setup;

#[cfg(test)]
pub(super) use test_setup::{test_prover_setup, test_verifier_setup};
