#[cfg(test)]
mod sql_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::sql::{AnalyzeError, EVMProofPlan, scale_cast_binary_op};
        let _ = AnalyzeError::default();
        let _ = scale_cast_binary_op;
    }
}
