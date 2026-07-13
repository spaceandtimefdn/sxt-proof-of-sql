//! Tests for ExtendedDoryReduce.

#[cfg(test)]
mod extended_dory_reduce_test {
    use crate::proof_primitive::dory::extended_dory_reduce;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_extended_dory_reduce_module_exists() {
        // Test that functions can be referenced
        let _ = extended_dory_reduce::extended_dory_reduce_prove::<TestScalar>;
        let _ = extended_dory_reduce::extended_dory_reduce_verify::<TestScalar>;
    }
}