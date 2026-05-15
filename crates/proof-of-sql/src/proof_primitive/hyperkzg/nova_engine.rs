use super::{BNScalar, HyperKZGPublicSetupOwned};
use crate::{
    base::{
        proof::{Keccak256Transcript, Transcript},
        slice_ops,
    },
    proof_primitive::hyperkzg::convert_g1_affine_from_halo2_to_ark,
};
use nova_snark::{
    errors::NovaError,
    provider::{bn256_grumpkin::bn256::Scalar as NovaScalar, hyperkzg::CommitmentKey},
    traits::{Engine, TranscriptEngineTrait, TranscriptReprTrait},
};
use serde::{Deserialize, Serialize};

/// The `HyperKZG` engine that implements nova's `Engine` trait.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HyperKZGEngine;

impl Engine for HyperKZGEngine {
    type Base = nova_snark::provider::bn256_grumpkin::bn256::Base;
    type Scalar = NovaScalar;
    type GE = nova_snark::provider::bn256_grumpkin::bn256::Point;
    type RO = nova_snark::provider::poseidon::PoseidonRO<Self::Base>;
    type ROCircuit = nova_snark::provider::poseidon::PoseidonROCircuit<Self::Base>;
    type RO2 = nova_snark::provider::poseidon::PoseidonRO<Self::Scalar>;
    type RO2Circuit = nova_snark::provider::poseidon::PoseidonROCircuit<Self::Scalar>;
    type TE = Keccak256Transcript;
    type CE = nova_snark::provider::hyperkzg::CommitmentEngine<Self>;
}

impl TranscriptEngineTrait<HyperKZGEngine> for Keccak256Transcript {
    fn new(_label: &'static [u8]) -> Self {
        Transcript::new()
    }

    fn squeeze(&mut self, _label: &'static [u8]) -> Result<NovaScalar, NovaError> {
        let res = Transcript::scalar_challenge_as_be::<BNScalar>(self).into();
        Transcript::challenge_as_le(self);
        Ok(res)
    }

    fn absorb<T: TranscriptReprTrait<<HyperKZGEngine as Engine>::GE>>(
        &mut self,
        _label: &'static [u8],
        o: &T,
    ) {
        Transcript::extend_as_le_from_refs(self, &o.to_transcript_bytes());
    }

    fn dom_sep(&mut self, _bytes: &'static [u8]) {}
}

/// Utility converting a nova `CommitmentKey` to a [`HyperKZGPublicSetupOwned`].
pub fn nova_commitment_key_to_hyperkzg_public_setup(
    setup: &CommitmentKey<HyperKZGEngine>,
) -> HyperKZGPublicSetupOwned {
    slice_ops::slice_cast_with(setup.ck(), convert_g1_affine_from_halo2_to_ark)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::hyperkzg::convert_g1_affine_from_ark_to_halo2;
    use ff::Field;
    use nova_snark::{
        provider::{
            bn256_grumpkin::bn256::Scalar as NovaScalar,
            hyperkzg::{Commitment as NovaCommitment, CommitmentEngine},
        },
        traits::{commitment::CommitmentEngineTrait, TranscriptEngineTrait},
    };

    #[test]
    fn commitment_key_conversion_preserves_nova_points() {
        let commitment_key: CommitmentKey<HyperKZGEngine> =
            CommitmentEngine::setup(b"nova-engine-test", 4);

        let public_setup = nova_commitment_key_to_hyperkzg_public_setup(&commitment_key);

        assert_eq!(public_setup.len(), commitment_key.ck().len());
        for (ark_point, halo2_point) in public_setup.iter().zip(commitment_key.ck()) {
            assert_eq!(convert_g1_affine_from_ark_to_halo2(ark_point), *halo2_point);
        }
    }

    #[test]
    fn transcript_engine_absorbs_commitments_and_squeezes_scalars() {
        let commitment_key: CommitmentKey<HyperKZGEngine> =
            CommitmentEngine::setup(b"nova-transcript-test", 1);
        let zero_commitment = NovaCommitment::<HyperKZGEngine>::default();
        let nonzero_commitment =
            CommitmentEngine::commit(&commitment_key, &[NovaScalar::ONE], &NovaScalar::ZERO);

        let mut zero_transcript =
            <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(b"ignored-label");
        zero_transcript.dom_sep(b"ignored-domain-separator");
        zero_transcript.absorb(b"ignored-absorb-label", &zero_commitment);

        let mut matching_transcript =
            <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(b"different-label");
        matching_transcript.absorb(b"different-absorb-label", &zero_commitment);

        let mut nonzero_transcript =
            <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(b"ignored-label");
        nonzero_transcript.absorb(b"ignored-absorb-label", &nonzero_commitment);

        let zero_challenge = zero_transcript.squeeze(b"ignored-squeeze-label").unwrap();
        let matching_challenge = matching_transcript
            .squeeze(b"different-squeeze-label")
            .unwrap();
        let nonzero_challenge = nonzero_transcript
            .squeeze(b"ignored-squeeze-label")
            .unwrap();

        assert_eq!(zero_challenge, matching_challenge);
        assert_ne!(zero_challenge, nonzero_challenge);
    }
}
