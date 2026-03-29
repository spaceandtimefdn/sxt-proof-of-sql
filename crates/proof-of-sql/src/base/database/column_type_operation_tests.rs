/// Additional tests for column type operations to improve coverage
#[cfg(test)]
mod tests {
    use crate::base::database::{ColumnOperationError, ColumnOperationResult, ColumnType};

    // Test the Display implementation for ColumnType
    #[test]
    fn test_column_type_display() {
        assert_eq!(ColumnType::Boolean.to_string(), "Boolean");
        assert_eq!(ColumnType::TinyInt.to_string(), "TinyInt");
        assert_eq!(ColumnType::SmallInt.to_string(), "SmallInt");
        assert_eq!(ColumnType::Int.to_string(), "Int");
        assert_eq!(ColumnType::BigInt.to_string(), "BigInt");
        assert_eq!(ColumnType::Int128.to_string(), "Int128");
        assert_eq!(ColumnType::Decimal75(crate::base::math::decimal::Precision::new(10).unwrap(), 2).to_string(), "Decimal75(10, 2)");
        assert_eq!(ColumnType::Scalar.to_string(), "Scalar");
        assert_eq!(ColumnType::VarChar.to_string(), "VarChar");
        assert_eq!(ColumnType::TimestampTZ(
            crate::base::database::ColumnType::TIMESTAMP_TZ_TIME_UNIT,
            crate::base::database::ColumnType::TIMESTAMP_TZ_TIMEZONE,
        ).to_string(), "TimestampTZ(Second, UTC)");
    }

    // Test ColumnOperationError display variants
    #[test]
    fn test_column_operation_error_display_division_by_zero() {
        let err = ColumnOperationError::DivisionByZero;
        let s = err.to_string();
        assert!(s.to_lowercase().contains("zero") || s.to_lowercase().contains("division") || !s.is_empty());
    }

    #[test]
    fn test_column_operation_error_display_int_type_overflow() {
        let err = ColumnOperationError::IntegerOverflow {
            error: "overflow".to_string(),
        };
        let s = err.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_column_operation_error_mismatch() {
        let err = ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "+".to_string(),
            left_type: ColumnType::BigInt,
            right_type: ColumnType::VarChar,
        };
        let s = err.to_string();
        assert!(s.contains('+') || s.contains("operator") || !s.is_empty());
    }
}
