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
    use crate::proof_primitive::hyperkzg::convert_g1_affine_from_halo2_to_ark;
    use halo2curves::group::prime::PrimeCurveAffine;
    use nova_snark::traits::{Engine, TranscriptEngineTrait, TranscriptReprTrait};

    struct TranscriptBytes(Vec<u8>);

    impl TranscriptReprTrait<<HyperKZGEngine as Engine>::GE> for TranscriptBytes {
        fn to_transcript_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
    }

    #[test]
    fn keccak_transcript_engine_labels_and_domain_separators_are_noops() {
        let mut first = <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(
            b"first label is ignored",
        );
        let mut second = <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(
            b"second label is ignored",
        );

        TranscriptEngineTrait::<HyperKZGEngine>::dom_sep(&mut first, b"ignored separator");

        assert_eq!(
            TranscriptEngineTrait::<HyperKZGEngine>::squeeze(&mut first, b"first")
                .expect("squeeze succeeds"),
            TranscriptEngineTrait::<HyperKZGEngine>::squeeze(&mut second, b"second")
                .expect("squeeze succeeds")
        );
    }

    #[test]
    fn keccak_transcript_engine_absorb_and_squeeze_update_state() {
        let mut empty = <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(b"");
        let mut absorbed = <Keccak256Transcript as TranscriptEngineTrait<HyperKZGEngine>>::new(b"");

        TranscriptEngineTrait::<HyperKZGEngine>::absorb(
            &mut absorbed,
            b"ignored label",
            &TranscriptBytes(vec![1, 2, 3, 4]),
        );

        let empty_challenge =
            TranscriptEngineTrait::<HyperKZGEngine>::squeeze(&mut empty, b"challenge")
                .expect("squeeze succeeds");
        let first_absorbed_challenge =
            TranscriptEngineTrait::<HyperKZGEngine>::squeeze(&mut absorbed, b"challenge")
                .expect("squeeze succeeds");
        let second_absorbed_challenge =
            TranscriptEngineTrait::<HyperKZGEngine>::squeeze(&mut absorbed, b"challenge")
                .expect("squeeze succeeds");

        assert_ne!(empty_challenge, first_absorbed_challenge);
        assert_ne!(first_absorbed_challenge, second_absorbed_challenge);
    }

    #[test]
    fn nova_commitment_key_conversion_preserves_ck_points() {
        let ck = vec![
            halo2curves::bn256::G1Affine::generator(),
            halo2curves::bn256::G1Affine::identity(),
        ];
        let setup = CommitmentKey::<HyperKZGEngine>::new(
            ck.clone(),
            halo2curves::bn256::G1Affine::generator(),
            halo2curves::bn256::G2Affine::generator(),
        );

        let public_setup = nova_commitment_key_to_hyperkzg_public_setup(&setup);

        assert_eq!(
            public_setup,
            ck.iter()
                .map(convert_g1_affine_from_halo2_to_ark)
                .collect::<Vec<_>>()
        );
    }
}
