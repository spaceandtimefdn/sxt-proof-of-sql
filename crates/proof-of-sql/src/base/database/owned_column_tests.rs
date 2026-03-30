/// Tests for [`OwnedColumn`] construction, length, and type introspection.
///
/// These cover the match-arm paths that are not reached by higher-level
/// end-to-end query tests.
#[cfg(test)]
mod tests {
    use crate::base::database::{ColumnType, OwnedColumn};
    use crate::base::scalar::test_scalar::TestScalar;

    // Helper: build a BigInt column with known values.
    fn bigint_col(values: &[i64]) -> OwnedColumn<TestScalar> {
        OwnedColumn::BigInt(values.to_vec())
    }

    // Helper: build a Boolean column.
    fn boolean_col(values: &[bool]) -> OwnedColumn<TestScalar> {
        OwnedColumn::Boolean(values.to_vec())
    }

    // -----------------------------------------------------------------------
    // len / is_empty
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_bigint_column_is_empty() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![]);
        assert!(col.is_empty(), "Empty column should report is_empty() == true");
        assert_eq!(col.len(), 0);
    }

    #[test]
    fn test_non_empty_bigint_column_length() {
        let col = bigint_col(&[1, 2, 3]);
        assert!(!col.is_empty());
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_varchar_column_length() {
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::VarChar(vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_boolean_column_length() {
        let col = boolean_col(&[true, false, true, false]);
        assert_eq!(col.len(), 4);
    }

    #[test]
    fn test_int128_column_length() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![0i128, i128::MAX, i128::MIN]);
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_tinyint_column_length() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1i8, 2, 3]);
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_smallint_column_length() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10i16, 20]);
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_int_column_length() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![100i32]);
        assert_eq!(col.len(), 1);
    }

    // -----------------------------------------------------------------------
    // column_type
    // -----------------------------------------------------------------------

    #[test]
    fn test_bigint_column_type() {
        assert_eq!(bigint_col(&[]).column_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_boolean_column_type() {
        assert_eq!(boolean_col(&[]).column_type(), ColumnType::Boolean);
    }

    #[test]
    fn test_varchar_column_type() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec![]);
        assert_eq!(col.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn test_int128_column_type() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![]);
        assert_eq!(col.column_type(), ColumnType::Int128);
    }

    #[test]
    fn test_tinyint_column_type() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![]);
        assert_eq!(col.column_type(), ColumnType::TinyInt);
    }

    #[test]
    fn test_smallint_column_type() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![]);
        assert_eq!(col.column_type(), ColumnType::SmallInt);
    }

    #[test]
    fn test_int_column_type() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
        assert_eq!(col.column_type(), ColumnType::Int);
    }

    // -----------------------------------------------------------------------
    // PartialEq / Debug
    // -----------------------------------------------------------------------

    #[test]
    fn test_two_equal_bigint_columns_compare_equal() {
        let a = bigint_col(&[1, 2, 3]);
        let b = bigint_col(&[1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_two_different_bigint_columns_are_not_equal() {
        let a = bigint_col(&[1, 2, 3]);
        let b = bigint_col(&[1, 2, 4]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_debug_output_is_non_empty() {
        let col = bigint_col(&[42]);
        assert!(!format!("{:?}", col).is_empty());
    }

    // -----------------------------------------------------------------------
    // Scalar column
    // -----------------------------------------------------------------------

    #[test]
    fn test_scalar_column_length_and_type() {
        let scalars = vec![TestScalar::ZERO, TestScalar::ONE];
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(scalars);
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::Scalar);
    }
}
