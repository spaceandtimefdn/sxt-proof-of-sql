//! Tests for DoryCommitmentHelper.

#[cfg(test)]
mod dory_commitment_helper_test {
    use crate::proof_primitive::dory::dory_commitment_helper_cpu;
    use crate::proof_primitive::dory::dory_commitment_helper_gpu;

    #[test]
    fn test_dory_commitment_helper_cpu_exists() {
        let _ = dory_commitment_helper_cpu::dory_commitment_helper_cpu_prove;
        let _ = dory_commitment_helper_cpu::dory_commitment_helper_cpu_verify;
    }
}