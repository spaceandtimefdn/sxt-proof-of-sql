//! Tests for column comparison operations.

#[cfg(test)]
mod column_comparison_operation_test {
    use crate::base::database::ColumnOperationError;

    #[test]
    fn test_comparison_error_types() {
        let errors = vec![
            ColumnOperationError::IntegerOverflow {
                error: "test overflow".to_string(),
            },
            ColumnOperationError::ColumnNotFound {
                column_name: "test_col".to_string(),
            },
        ];
        for err in errors {
            let s = format!("{}", err);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_comparison_error_debug() {
        let err = ColumnOperationError::IntegerOverflow {
            error: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_comparison_error_partial_eq() {
        let err1 = ColumnOperationError::IntegerOverflow {
            error: "test".to_string(),
        };
        let err2 = ColumnOperationError::IntegerOverflow {
            error: "test".to_string(),
        };
        let err3 = ColumnOperationError::IntegerOverflow {
            error: "different".to_string(),
        };
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_signed_casting_error() {
        let err = ColumnOperationError::SignedCastingError {
            left_type: crate::base::database::ColumnType::Uint8,
            right_type: crate::base::database::ColumnType::TinyInt,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_different_column_length() {
        let err = ColumnOperationError::DifferentColumnLength { len_a: 5, len_b: 10 };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_invalid_cast() {
        let err = ColumnOperationError::InvalidCast {
            from_type: crate::base::database::ColumnType::Boolean,
            to_type: crate::base::database::ColumnType::BigInt,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }
}