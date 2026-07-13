//! Tests for StandardBasisHelper.

#[cfg(test)]
mod standard_basis_helper_test {
    #[test]
    fn test_standard_basis_helper_module_exists() {
        // Module should be accessible
        let type_name = std::any::type_name::<crate::proof_primitive::dynamic_matrix_utils::standard_basis_helper::StandardBasisHelper>();
        assert!(!type_name.is_empty());
    }
}
