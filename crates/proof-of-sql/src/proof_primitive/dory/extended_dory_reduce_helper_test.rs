//! Tests for ExtendedDoryReduceHelper.

#[cfg(test)]
mod extended_dory_reduce_helper_test {
    use crate::proof_primitive::dory::extended_dory_reduce_helper;

    #[test]
    fn test_extended_dory_reduce_helper_module_exists() {
        // Test that functions can be referenced
        let _ = extended_dory_reduce_helper::extended_dory_reduce_prove_compute_E_betas;
        let _ = extended_dory_reduce_helper::extended_dory_reduce_prove_compute_signed_Es;
        let _ = extended_dory_reduce_helper::extended_dory_reduce_prove_fold_s_vecs;
        let _ = extended_dory_reduce_helper::extended_dory_reduce_verify_update_Es;
    }
}