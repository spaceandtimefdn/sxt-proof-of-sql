#[cfg(test)]
mod math_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::math::permutation::{Permutation, PermutationError};
        let _ = PermutationError::default();
    }
}
