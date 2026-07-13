//! Tests for DoryReduceHelper.

#[cfg(test)]
mod dory_reduce_helper_test {
    use crate::proof_primitive::dory::dory_reduce_helper;

    #[test]
    fn test_dory_reduce_helper_module_exists() {
        // Test that functions can be referenced
        let _ = dory_reduce_helper::dory_reduce_prove_compute_Ds;
        let _ = dory_reduce_helper::dory_reduce_prove_mutate_v_vecs;
        let _ = dory_reduce_helper::dory_reduce_prove_compute_Cs;
        let _ = dory_reduce_helper::dory_reduce_verify_update_C;
        let _ = dory_reduce_helper::dory_reduce_verify_update_Ds;
    }
}