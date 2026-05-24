//! Tests for DynamicDoryCommitmentHelperCpu.

#[cfg(all(test, not(feature = "blitzar")))]
mod dynamic_dory_commitment_helper_cpu_test {
    use crate::proof_primitive::dory::dynamic_dory_commitment_helper_cpu;

    #[test]
    fn test_dynamic_dory_commitment_helper_cpu_exists() {
        let _ = dynamic_dory_commitment_helper_cpu::dynamic_dory_commitment_helper_cpu_prove;
        let _ = dynamic_dory_commitment_helper_cpu::dynamic_dory_commitment_helper_cpu_verify;
        let _ = dynamic_dory_commitment_helper_cpu::compute_dynamic_dory_commitments;
    }
}