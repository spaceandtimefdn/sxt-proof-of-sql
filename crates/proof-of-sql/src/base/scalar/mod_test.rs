#[cfg(test)]
mod scalar_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::scalar::{Scalar, MontScalar, ScalarConversionError};
        let _ = ScalarConversionError::default();
    }
}
