//! Implementation of `HyperKZG` PCS for usage with proof-of-sql.
//!
//! The prover side of this implementation simply wraps around nova's hyper-kzg implementation.
//!
//! While the `Commitment` for this commitment scheme is always available, the corresponding
//! `CommitmentEvaluationProof` is gated behind the `hyperkzg_proof` feature flag.
//! This is done to preserve `no_std` compatibility for `no_std` commitment generation apps.

mod scalar;
pub use scalar::BNScalar;

mod public_setup;
#[cfg(feature = "std")]
pub use public_setup::deserialize_flat_compressed_hyperkzg_public_setup_from_reader;
#[cfg(feature = "hyperkzg_proof")]
pub use public_setup::load_small_setup_for_testing;
pub use public_setup::{
    deserialize_flat_compressed_hyperkzg_public_setup_from_slice, HyperKZGPublicSetup,
    HyperKZGPublicSetupOwned,
};

mod commitment;
pub use commitment::HyperKZGCommitment;

#[cfg(feature = "hyperkzg_proof")]
mod arkworks_halo2_interop;
#[cfg(feature = "hyperkzg_proof")]
pub(crate) use arkworks_halo2_interop::{
    convert_to_ark_bn254_g1_affine, convert_to_halo2_bn256_g1_affine,
};

#[cfg(feature = "hyperkzg_proof")]
mod nova_commitment;

#[cfg(feature = "hyperkzg_proof")]
mod nova_engine;
#[cfg(feature = "hyperkzg_proof")]
pub use nova_engine::{nova_commitment_key_to_hyperkzg_public_setup, HyperKZGEngine};

#[cfg(feature = "hyperkzg_proof")]
mod commitment_evaluation_proof;
#[cfg(feature = "hyperkzg_proof")]
pub use commitment_evaluation_proof::HyperKZGCommitmentEvaluationProof;

#[cfg(feature = "hyperkzg_proof")]
mod halo2_conversions;

#[cfg(all(test, feature = "hyperkzg_proof"))]
mod evm_tests;
