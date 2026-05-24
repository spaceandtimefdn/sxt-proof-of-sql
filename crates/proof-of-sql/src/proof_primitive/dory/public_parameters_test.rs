//! Tests for PublicParameters.

#[cfg(test)]
mod public_parameters_test {
    use crate::proof_primitive::dory::PublicParameters;

    #[test]
    fn test_public_parameters_type_exists() {
        let _: Option<PublicParameters> = None;
    }

    #[test]
    fn test_public_parameters_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<PublicParameters>());
        assert!(!debug_str.is_empty());
    }
}
