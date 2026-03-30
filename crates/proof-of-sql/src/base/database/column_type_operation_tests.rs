/// Tests for column type arithmetic and comparison operations.
///
/// These tests cover the type-coercion and error paths that are not exercised
/// by the main integration suite.
#[cfg(test)]
mod tests {
    use crate::base::database::ColumnType;

    // -----------------------------------------------------------------------
    // try_add / try_sub
    // -----------------------------------------------------------------------

    #[test]
    fn test_try_add_integer_types_are_commutative() {
        // Adding two integer columns of the same type gives the same type back.
        let pairs = [
            (ColumnType::TinyInt, ColumnType::TinyInt),
            (ColumnType::SmallInt, ColumnType::SmallInt),
            (ColumnType::Int, ColumnType::Int),
            (ColumnType::BigInt, ColumnType::BigInt),
            (ColumnType::Int128, ColumnType::Int128),
        ];
        for (a, b) in pairs {
            let result_ab = a.try_add(b);
            let result_ba = b.try_add(a);
            assert_eq!(
                result_ab, result_ba,
                "try_add should be commutative for {:?} and {:?}",
                a, b
            );
            assert!(
                result_ab.is_ok(),
                "Same-type integer addition should succeed for {:?}",
                a
            );
        }
    }

    #[test]
    fn test_try_add_integer_widens_to_larger_type() {
        // TinyInt + BigInt should succeed and yield BigInt (or wider).
        let result = ColumnType::TinyInt.try_add(ColumnType::BigInt);
        assert!(
            result.is_ok(),
            "Adding TinyInt and BigInt should succeed, got {:?}",
            result
        );
        // The result type must be at least as wide as BigInt.
        let result_type = result.unwrap();
        assert!(
            matches!(
                result_type,
                ColumnType::BigInt | ColumnType::Int128
            ),
            "Expected BigInt or wider, got {:?}",
            result_type
        );
    }

    #[test]
    fn test_try_add_decimal_types_succeed() {
        use crate::base::math::decimal::Precision;
        let dec38 = ColumnType::Decimal75(Precision::new(38).unwrap(), 6);
        let dec10 = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        // Adding two Decimal75 columns should succeed.
        assert!(
            dec38.try_add(dec10).is_ok(),
            "Decimal75 + Decimal75 should succeed"
        );
    }

    #[test]
    fn test_try_add_incompatible_types_return_error() {
        // Boolean columns cannot participate in arithmetic.
        let result = ColumnType::Boolean.try_add(ColumnType::BigInt);
        assert!(
            result.is_err(),
            "Boolean + BigInt should fail, got {:?}",
            result
        );

        let result2 = ColumnType::BigInt.try_add(ColumnType::Boolean);
        assert!(
            result2.is_err(),
            "BigInt + Boolean should fail, got {:?}",
            result2
        );
    }

    #[test]
    fn test_try_sub_mirrors_try_add_for_same_types() {
        let types = [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ];
        for t in types {
            assert_eq!(
                t.try_add(t),
                t.try_sub(t),
                "try_add and try_sub should agree on the output type for {:?}",
                t
            );
        }
    }

    // -----------------------------------------------------------------------
    // try_multiply
    // -----------------------------------------------------------------------

    #[test]
    fn test_try_multiply_integer_types() {
        let result = ColumnType::Int.try_multiply(ColumnType::Int);
        assert!(result.is_ok(), "Int * Int should succeed");
    }

    #[test]
    fn test_try_multiply_incompatible_returns_error() {
        let result = ColumnType::Boolean.try_multiply(ColumnType::Int);
        assert!(
            result.is_err(),
            "Boolean * Int should fail, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // try_divide
    // -----------------------------------------------------------------------

    #[test]
    fn test_try_divide_integer_types() {
        let result = ColumnType::BigInt.try_divide(ColumnType::BigInt);
        assert!(result.is_ok(), "BigInt / BigInt should succeed");
    }

    #[test]
    fn test_try_divide_incompatible_returns_error() {
        let result = ColumnType::Boolean.try_divide(ColumnType::Int);
        assert!(
            result.is_err(),
            "Boolean / Int should fail, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // is_numeric / is_integer
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_numeric_returns_true_for_numeric_types() {
        let numeric_types = [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ];
        for t in numeric_types {
            assert!(t.is_numeric(), "{:?} should be numeric", t);
        }
    }

    #[test]
    fn test_is_numeric_returns_false_for_non_numeric_types() {
        let non_numeric = [ColumnType::Boolean, ColumnType::VarChar];
        for t in non_numeric {
            assert!(!t.is_numeric(), "{:?} should NOT be numeric", t);
        }
    }

    #[test]
    fn test_is_integer_returns_true_for_integer_types() {
        let integer_types = [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ];
        for t in integer_types {
            assert!(t.is_integer(), "{:?} should be integer", t);
        }
    }

    #[test]
    fn test_is_integer_returns_false_for_decimal_and_boolean() {
        use crate::base::math::decimal::Precision;
        let non_integer = [
            ColumnType::Boolean,
            ColumnType::VarChar,
            ColumnType::Decimal75(Precision::new(10).unwrap(), 2),
        ];
        for t in non_integer {
            assert!(!t.is_integer(), "{:?} should NOT be integer", t);
        }
    }
}
