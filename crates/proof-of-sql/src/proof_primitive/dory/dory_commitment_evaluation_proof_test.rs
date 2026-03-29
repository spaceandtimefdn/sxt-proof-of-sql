use super::{
    test_setup::{test_prover_setup, test_public_parameters, test_verifier_setup, TEST_SIGMA},
    DoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
};
use crate::base::commitment::CommitmentEvaluationProof;
use ark_std::UniformRand;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Create a small `PublicParameters` for tests that need a specific sigma.
fn make_pp(sigma: usize) -> PublicParameters {
    PublicParameters::test_rand(sigma, &mut ChaCha20Rng::seed_from_u64(0))
}

#[test]
fn test_simple_ipa() {
    // Re-use the cached setup (sigma=4 is large enough for length-1 vectors).
    let ps = test_prover_setup();
    let vs = test_verifier_setup();

    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let a: Vec<_> = (0..1)
        .map(|_| ark_bls12_381::Fr::rand(&mut rng))
        .collect();
    let b: Vec<_> = (0..1)
        .map(|_| ark_bls12_381::Fr::rand(&mut rng))
        .collect();
    let offset = 0usize;
    DoryEvaluationProof::verify_batched_proof(
        &DoryEvaluationProof::new_batched_proof(&a, &b, offset, ps, &mut rng)
            .expect("proof creation should succeed"),
        &a,
        &b,
        offset,
        vs,
    )
    .expect("proof verification should succeed");
}

#[test]
fn test_random_ipa_with_length_1() {
    let ps = test_prover_setup();
    let vs = test_verifier_setup();

    let mut rng = ChaCha20Rng::seed_from_u64(1);
    let a: Vec<_> = (0..1)
        .map(|_| ark_bls12_381::Fr::rand(&mut rng))
        .collect();
    let b: Vec<_> = (0..1)
        .map(|_| ark_bls12_381::Fr::rand(&mut rng))
        .collect();
    let offset = 0usize;
    DoryEvaluationProof::verify_batched_proof(
        &DoryEvaluationProof::new_batched_proof(&a, &b, offset, ps, &mut rng)
            .expect("proof creation should succeed"),
        &a,
        &b,
        offset,
        vs,
    )
    .expect("proof verification should succeed");
}

#[test]
fn test_random_ipa_with_various_lengths() {
    // Use cached setup for all sizes up to 2^TEST_SIGMA.
    let ps = test_prover_setup();
    let vs = test_verifier_setup();

    let mut rng = ChaCha20Rng::seed_from_u64(2);
    for length in [1usize, 2, 3, 4, 7, 8, 9, 15, 16] {
        if length > (1 << TEST_SIGMA) {
            continue; // skip lengths beyond the cached setup's capacity
        }
        let a: Vec<_> = (0..length)
            .map(|_| ark_bls12_381::Fr::rand(&mut rng))
            .collect();
        let b: Vec<_> = (0..length)
            .map(|_| ark_bls12_381::Fr::rand(&mut rng))
            .collect();
        let offset = 0usize;
        DoryEvaluationProof::verify_batched_proof(
            &DoryEvaluationProof::new_batched_proof(&a, &b, offset, ps, &mut rng)
                .expect("proof creation should succeed"),
            &a,
            &b,
            offset,
            vs,
        )
        .expect("proof verification should succeed for length {length}");
    }
}
