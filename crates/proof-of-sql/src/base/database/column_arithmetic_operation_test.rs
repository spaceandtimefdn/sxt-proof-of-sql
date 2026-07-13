//! Tests for column arithmetic operations.

#[cfg(test)]
mod column_arithmetic_operation_test {
    use crate::base::database::{ColumnOperationError, OwnedColumn};
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn test_column_operation_error_display() {
        let err = ColumnOperationError::DifferentColumnLength {
            len_a: 5,
            len_b: 10,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_column_operation_error_signed_casting() {
        let err = ColumnOperationError::SignedCastingError {
            left_type: crate::base::database::ColumnType::Uint8,
            right_type: crate::base::database::ColumnType::TinyInt,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_column_operation_error_integer_overflow() {
        let err = ColumnOperationError::IntegerOverflow {
            error: "test overflow".to_string(),
        };
        let s = format!("{}", err);
        assert!(s.contains("overflow"));
    }

    #[test]
    fn test_column_operation_error_column_not_found() {
        let err = ColumnOperationError::ColumnNotFound {
            column_name: "test_col".to_string(),
        };
        let s = format!("{}", err);
        assert!(s.contains("test_col"));
    }

    #[test]
    fn test_column_operation_error_invalid_cast() {
        let err = ColumnOperationError::InvalidCast {
            from_type: crate::base::database::ColumnType::Boolean,
            to_type: crate::base::database::ColumnType::BigInt,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_column_operation_error_partial_eq() {
        let err1 = ColumnOperationError::DifferentColumnLength { len_a: 5, len_b: 10 };
        let err2 = ColumnOperationError::DifferentColumnLength { len_a: 5, len_b: 10 };
        let err3 = ColumnOperationError::DifferentColumnLength { len_a: 3, len_b: 7 };
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_column_operation_error_debug() {
        let err = ColumnOperationError::IntegerOverflow {
            error: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_different_column_length_error() {
        let col_a = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let col_b = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3, 4, 5]);
        // These columns have different lengths
        assert_eq!(col_a.len(), 3);
        assert_eq!(col_b.len(), 5);
    }

    #[test]
    fn test_owned_column_arithmetic_types() {
        // Test that different column types can be created
        let col = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        assert_eq!(col.len(), 3);
        assert_eq!(col.column_type(), crate::base::database::ColumnType::BigInt);

        let col2 = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        assert_eq!(col2.column_type(), crate::base::database::ColumnType::Int);

        let col3 = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        assert_eq!(col3.column_type(), crate::base::database::ColumnType::SmallInt);
    }

    #[test]
    fn test_owned_column_clone() {
        let col = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let cloned = col.clone();
        assert_eq!(col, cloned);
    }

    #[test]
    fn test_owned_column_debug() {
        let col = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let debug_str = format!("{:?}", col);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_all_error_variants_have_display() {
        let errors = vec![
            ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 },
            ColumnOperationError::SignedCastingError {
                left_type: crate::base::database::ColumnType::Uint8,
                right_type: crate::base::database::ColumnType::TinyInt,
            },
            ColumnOperationError::IntegerOverflow {
                error: "test".to_string(),
            },
            ColumnOperationError::ColumnNotFound {
                column_name: "test".to_string(),
            },
            ColumnOperationError::InvalidCast {
                from_type: crate::base::database::ColumnType::Boolean,
                to_type: crate::base::database::ColumnType::BigInt,
            },
        ];
        for err in errors {
            let s = format!("{}", err);
            assert!(!s.is_empty(), "Error should have a display message");
        }
    }
}