//! Tests for ProvableResultColumn.

#[cfg(test)]
mod provable_result_column_test {
    use crate::sql::proof::ProvableResultColumn;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_provable_result_column_exists() {
        // Test that the type can be referenced
        let _: Option<ProvableResultColumn<TestScalar>> = None;
    }

    #[test]
    fn test_provable_result_column_debug() {
        // Create a simple result column for testing
        use crate::base::database::OwnedColumn;
        let col = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let result: ProvableResultColumn<TestScalar> = col.into();
        let debug_str = format!("{:?}", result);
        assert!(!debug_str.is_empty());
    }
}