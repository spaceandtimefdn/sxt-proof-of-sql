use super::{AddExpr, DecimalProofExpr, DynProofExpr, MultiplyExpr, ProofExpr};
use crate::base::{
    database::{Column, ColumnType, LiteralValue, Table, TableOptions},
    map::IndexMap,
    math::decimal::Precision,
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

fn decimal_literal(precision: u8, scale: i8, value: i32) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        value.into(),
    ))
}

fn empty_table_with_rows<'a>(row_count: usize) -> Table<'a, TestScalar> {
    Table::try_new_with_options(IndexMap::default(), TableOptions::new(Some(row_count))).unwrap()
}

#[test]
fn add_expr_decimal_helpers_use_result_type() {
    let expr = AddExpr::try_new(
        Box::new(decimal_literal(20, 3, 10)),
        Box::new(decimal_literal(10, 3, 200)),
    )
    .unwrap();

    assert_eq!(
        expr.data_type(),
        ColumnType::Decimal75(Precision::new(21).unwrap(), 3)
    );
    assert_eq!(expr.precision(), Precision::new(21).unwrap());
    assert_eq!(expr.scale(), 3);
    assert_eq!(
        expr.lhs().data_type(),
        ColumnType::Decimal75(Precision::new(20).unwrap(), 3)
    );
    assert_eq!(
        expr.rhs().data_type(),
        ColumnType::Decimal75(Precision::new(10).unwrap(), 3)
    );
}

#[test]
fn add_expr_first_round_evaluate_repeats_decimal_result_for_empty_table() {
    let alloc = Bump::new();
    let table = empty_table_with_rows(3);
    let expr = AddExpr::try_new(
        Box::new(decimal_literal(20, 3, 10)),
        Box::new(decimal_literal(10, 3, 200)),
    )
    .unwrap();

    let result = expr
        .first_round_evaluate::<TestScalar>(&alloc, &table, &[])
        .unwrap();

    assert_eq!(
        result,
        Column::Decimal75(
            Precision::new(21).unwrap(),
            3,
            &[
                TestScalar::from(210),
                TestScalar::from(210),
                TestScalar::from(210)
            ]
        )
    );
}

#[test]
fn multiply_expr_decimal_helpers_use_result_type() {
    let expr = MultiplyExpr::try_new(
        Box::new(decimal_literal(20, 3, 20)),
        Box::new(decimal_literal(10, 2, 30)),
    )
    .unwrap();

    assert_eq!(
        expr.data_type(),
        ColumnType::Decimal75(Precision::new(31).unwrap(), 5)
    );
    assert_eq!(expr.precision(), Precision::new(31).unwrap());
    assert_eq!(expr.scale(), 5);
    assert_eq!(
        expr.lhs().data_type(),
        ColumnType::Decimal75(Precision::new(20).unwrap(), 3)
    );
    assert_eq!(
        expr.rhs().data_type(),
        ColumnType::Decimal75(Precision::new(10).unwrap(), 2)
    );
}

#[test]
fn multiply_expr_first_round_evaluate_repeats_decimal_result_for_empty_table() {
    let alloc = Bump::new();
    let table = empty_table_with_rows(2);
    let expr = MultiplyExpr::try_new(
        Box::new(decimal_literal(20, 3, 20)),
        Box::new(decimal_literal(10, 2, 30)),
    )
    .unwrap();

    let result = expr
        .first_round_evaluate::<TestScalar>(&alloc, &table, &[])
        .unwrap();

    assert_eq!(
        result,
        Column::Decimal75(
            Precision::new(31).unwrap(),
            5,
            &[TestScalar::from(600), TestScalar::from(600)]
        )
    );
}
