/// Tests for column type arithmetic operations
#[cfg(test)]
mod tests {
    use crate::base::database::{ColumnType, ColumnTypeOperation};

    // Helper: verify that a binary op on two column types returns the expected result type
    fn check_binary_op(
        op: impl Fn(ColumnType, ColumnType) -> Option<ColumnType>,
        lhs: ColumnType,
        rhs: ColumnType,
        expected: Option<ColumnType>,
    ) {
        assert_eq!(op(lhs, rhs), expected, "lhs={lhs:?} rhs={rhs:?}");
    }

    // -----------------------------------------------------------------------
    // Addition / Subtraction – integer widening
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_same_integer_type_returns_same() {
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::TinyInt,
            ColumnType::TinyInt,
            Some(ColumnType::TinyInt),
        );
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::Int,
            ColumnType::Int,
            Some(ColumnType::Int),
        );
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::BigInt,
            ColumnType::BigInt,
            Some(ColumnType::BigInt),
        );
    }

    #[test]
    fn test_add_integer_widening() {
        // TinyInt + BigInt => BigInt
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::TinyInt,
            ColumnType::BigInt,
            Some(ColumnType::BigInt),
        );
        // SmallInt + Int => Int
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::SmallInt,
            ColumnType::Int,
            Some(ColumnType::Int),
        );
    }

    #[test]
    fn test_add_incompatible_types_returns_none() {
        // VARCHAR + INT should not be valid for arithmetic
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::VarChar,
            ColumnType::BigInt,
            None,
        );
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::BigInt,
            ColumnType::VarChar,
            None,
        );
    }

    // -----------------------------------------------------------------------
    // Decimal / Scalar combinations
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_scalar_types() {
        check_binary_op(
            ColumnType::arithmetic_result_type,
            ColumnType::Scalar,
            ColumnType::Scalar,
            Some(ColumnType::Scalar),
        );
    }
}
