/// Tests for the [`Column`] enum — focusing on `column_type()`, `len()`,
/// `is_empty()`, and the conversion helpers that are lightly covered by other
/// test modules.
#[cfg(test)]
mod tests {
    use crate::base::database::{Column, ColumnType};
    use crate::base::scalar::test_scalar::TestScalar;

    // -----------------------------------------------------------------------
    // column_type()
    // -----------------------------------------------------------------------

    #[test]
    fn test_boolean_column_type() {
        let data: &[bool] = &[true, false];
        let col: Column<TestScalar> = Column::Boolean(data);
        assert_eq!(col.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn test_tinyint_column_type() {
        let data: &[i8] = &[1, -1];
        let col: Column<TestScalar> = Column::TinyInt(data);
        assert_eq!(col.column_type(), ColumnType::TinyInt);
    }

    #[test]
    fn test_smallint_column_type() {
        let data: &[i16] = &[100, 200];
        let col: Column<TestScalar> = Column::SmallInt(data);
        assert_eq!(col.column_type(), ColumnType::SmallInt);
    }

    #[test]
    fn test_int_column_type() {
        let data: &[i32] = &[1_000];
        let col: Column<TestScalar> = Column::Int(data);
        assert_eq!(col.column_type(), ColumnType::Int);
    }

    #[test]
    fn test_bigint_column_type() {
        let data: &[i64] = &[i64::MAX, 0];
        let col: Column<TestScalar> = Column::BigInt(data);
        assert_eq!(col.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_int128_column_type() {
        let data: &[i128] = &[i128::MIN, i128::MAX];
        let col: Column<TestScalar> = Column::Int128(data);
        assert_eq!(col.column_type(), ColumnType::Int128);
    }

    // -----------------------------------------------------------------------
    // len() / is_empty()
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_column_is_empty() {
        let data: &[i64] = &[];
        let col: Column<TestScalar> = Column::BigInt(data);
        assert!(col.is_empty());
        assert_eq!(col.len(), 0);
    }

    #[test]
    fn test_non_empty_column_length() {
        let data: &[i64] = &[1, 2, 3, 4, 5];
        let col: Column<TestScalar> = Column::BigInt(data);
        assert!(!col.is_empty());
        assert_eq!(col.len(), 5);
    }

    #[test]
    fn test_boolean_column_length() {
        let data: &[bool] = &[true; 10];
        let col: Column<TestScalar> = Column::Boolean(data);
        assert_eq!(col.len(), 10);
    }

    #[test]
    fn test_varchar_column_length_and_type() {
        let strs = vec!["alpha", "beta", "gamma"];
        let scalars: Vec<TestScalar> = strs
            .iter()
            .map(|_| TestScalar::ZERO)
            .collect();
        let col: Column<TestScalar> = Column::VarChar((&strs, &scalars));
        assert_eq!(col.column_type(), ColumnType::VarChar);
        assert_eq!(col.len(), 3);
    }

    // -----------------------------------------------------------------------
    // Scalar column
    // -----------------------------------------------------------------------

    #[test]
    fn test_scalar_column_type_and_length() {
        let data = vec![TestScalar::ZERO, TestScalar::ONE];
        let col: Column<TestScalar> = Column::Scalar(&data);
        assert_eq!(col.column_type(), ColumnType::Scalar);
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_scalar_column_is_not_empty() {
        let data = vec![TestScalar::ONE];
        let col: Column<TestScalar> = Column::Scalar(&data);
        assert!(!col.is_empty());
    }
}
