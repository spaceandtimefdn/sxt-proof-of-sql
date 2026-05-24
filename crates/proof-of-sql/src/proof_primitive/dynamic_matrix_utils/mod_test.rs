#[cfg(test)]
mod dynamic_matrix_utils_mod_test {
    use crate::proof_primitive::dynamic_matrix_utils::{matrix_structure, standard_basis_helper};
    #[test]
    fn test_module_exports() {
        let _ = matrix_structure::DynamicMatrixStructure::default();
        let _ = standard_basis_helper::StandardBasisHelper::default();
    }
}
