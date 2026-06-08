use crate::proof_primitive::dory::{PublicParameters, ProverSetup, VerifierSetup};
#[cfg(test)]
use crate::tests::common::dory_setup_cache::{get_dory_pp, get_dory_prover, get_dory_verifier};

#[test]
fn we_can_create_prover_setups_with_various_sizes() {
    let pp = get_dory_pp();
    let prover = get_dory_prover();
    let verifier = get_dory_verifier();
    // ... rest of test logic ...
}
