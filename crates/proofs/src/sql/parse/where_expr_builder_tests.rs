#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType},
            math::decimal::Precision,
        },
        record_batch,
        sql::{
            ast::BoolExprPlan,
            parse::{query_expr_tests::record_batch_to_accessor, QueryExpr, WhereExprBuilder},
        },
    };
    use curve25519_dalek::RistrettoPoint;
    use proofs_sql::{
        decimal_unknown::DecimalUnknown,
        intermediate_ast::{BinaryOperator, Expression, Literal},
        Identifier, SelectStatement,
    };
    use std::{collections::HashMap, str::FromStr};

    fn run_test_case(column_mapping: &HashMap<Identifier, ColumnRef>, expr: Expression) {
        let builder = WhereExprBuilder::new(column_mapping);
        let result = builder.build::<RistrettoPoint>(Some(Box::new(expr)));
        assert!(result.is_ok(), "Test case should succeed without panic.");
    }

    fn get_column_mappings_for_testing() -> HashMap<Identifier, ColumnRef> {
        let mut column_mapping = HashMap::new();
        // Setup column mapping
        column_mapping.insert(
            Identifier::try_new("decimal_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("decimal_column").unwrap(),
                ColumnType::Decimal75(Precision::new(6).unwrap(), 2),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("int128_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("int128_column").unwrap(),
                ColumnType::Int128,
            ),
        );
        column_mapping.insert(
            Identifier::try_new("bigint_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ),
        );

        column_mapping.insert(
            Identifier::try_new("varchar_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("varchar_column").unwrap(),
                ColumnType::VarChar,
            ),
        );
        column_mapping
    }

    #[test]
    fn we_cannot_round_decimals_down_to_match() {
        let mut column_mapping = HashMap::new();
        column_mapping.insert(
            Identifier::try_new("test_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("c").unwrap(),
                ColumnType::Decimal75(Precision::new(6).unwrap(), 1),
            ),
        );

        let builder = WhereExprBuilder::new(&column_mapping);
        let left_expr = Expression::Column(Identifier::try_new("test_column").unwrap());
        let right_expr = Expression::Literal(Literal::Decimal(DecimalUnknown::new("123.456")));

        let expr = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        };

        // Error because we cannot round a decimal down
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr)))
            .is_err());
    }

    #[test]
    fn we_can_directly_check_whether_integer_columns_eq_integer() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int128_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_whether_bigint_columns_ge_int128() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::GreaterThanOrEqual,
            left: Box::new(Expression::Column(
                Identifier::try_new("bigint_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(-12345))),
        };
        let actual = builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .unwrap()
            .unwrap();
        let expected = BoolExprPlan::new_inequality(
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ),
            (-12345).into(),
            false,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_directly_check_whether_bigint_columns_le_int128() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::LessThanOrEqual,
            left: Box::new(Expression::Column(
                Identifier::try_new("bigint_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(-12345))),
        };
        let actual = builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .unwrap()
            .unwrap();
        let expected = BoolExprPlan::new_inequality(
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ),
            (-12345).into(),
            true,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_directly_check_whether_varchar_columns_eq_varchar() {
        let column_mapping = get_column_mappings_for_testing();
        // VarChar column with VarChar literal
        let expr_varchar_to_varchar = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("varchar_column").unwrap(),
            )), // Ensure this column is defined in column_mapping
            right: Box::new(Expression::Literal(Literal::VarChar(
                "test_string".to_string(),
            ))),
        };

        run_test_case(&column_mapping, expr_varchar_to_varchar);
    }

    #[test]
    fn we_can_check_non_decimal_columns_eq_integer_literals() {
        let column_mapping = get_column_mappings_for_testing();

        // Non-decimal column with integer literal
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int128_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        run_test_case(&column_mapping, expr_integer_to_integer);
    }

    #[test]
    fn we_can_check_scaled_integers_eq_correctly() {
        let column_mapping = get_column_mappings_for_testing();

        // Decimal column with integer literal that can be appropriately scaled
        let expr_integer_to_decimal = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("decimal_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        run_test_case(&column_mapping, expr_integer_to_decimal);
    }

    #[test]
    fn we_can_check_exact_scale_and_precision_eq() {
        let column_mapping = get_column_mappings_for_testing();

        // Decimal column with matching scale decimal literal
        let expr_decimal = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("decimal_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Decimal(DecimalUnknown::new(
                "123.45",
            )))),
        };
        run_test_case(&column_mapping, expr_decimal);
    }

    #[test]
    #[should_panic(expected = "The parser must ensure that the expression is a boolean expression")]
    fn unexpected_expression_panics() {
        let column_mapping = HashMap::new();

        let builder = WhereExprBuilder::new(&column_mapping);
        // Creating an unexpected expression type
        let expr_unexpected = Expression::Literal(Literal::Int128(123));
        builder
            .build::<RistrettoPoint>(Some(Box::new(expr_unexpected)))
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "The parser must ensure that the left side is a column")]
    fn left_side_not_column_panics() {
        let column_mapping = HashMap::new();

        let builder = WhereExprBuilder::new(&column_mapping);
        // Intentionally setting the left expression to a non-column type to trigger a panic
        let left_expr = Expression::Literal(Literal::Int128(123));
        let right_expr = Expression::Literal(Literal::Int128(456));

        let expr = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        };

        // This should trigger a panic due to the left side not being a column
        let _ = builder.build::<RistrettoPoint>(Some(Box::new(expr)));
    }

    #[test]
    fn we_expect_an_error_while_trying_to_check_varchar_column_eq_decimal() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => ["abc"],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_expect_an_error_while_trying_to_check_varchar_column_ge_decimal() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => ["abc"],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b >= 123").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_expect_an_error_while_trying_to_check_int128_column_eq_decimal() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i128],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.456").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_do_not_expect_an_error_while_trying_to_check_int128_column_eq_decimal_with_zero_scale() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i128],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_ok());
    }

    #[test]
    fn we_do_not_expect_an_error_while_trying_to_check_bigint_column_eq_decimal_with_zero_scale() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i64],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_ok());
    }

    #[test]
    fn we_do_expect_an_error_while_trying_to_check_bigint_column_eq_decimal_with_nonzero_scale() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i64],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.456").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }
}
