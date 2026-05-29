use super::{DynProofExpr, ProofExpr, ScalingCastExpr};
use crate::{
    base::{
        database::{Column, ColumnType, LiteralValue, Table, TableOptions},
        map::{indexmap, IndexMap},
        math::decimal::Precision,
        scalar::test_scalar::TestScalar,
    },
    sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use bumpalo::Bump;

fn decimal_type(precision: u8, scale: i8) -> ColumnType {
    ColumnType::Decimal75(
        Precision::new(precision).expect("test precision should be valid"),
        scale,
    )
}

fn int_to_decimal_expr() -> ScalingCastExpr {
    ScalingCastExpr::try_new(
        Box::new(DynProofExpr::new_literal(LiteralValue::Int(7))),
        decimal_type(11, 1),
    )
    .expect("int to decimal scaling cast should be valid")
}

fn empty_table_with_rows<'a>(row_count: usize) -> Table<'a, TestScalar> {
    Table::try_new_with_options(IndexMap::default(), TableOptions::new(Some(row_count)))
        .expect("empty test table with fixed row count should be valid")
}

fn assert_decimal_column(column: Column<TestScalar>, expected: &[TestScalar]) {
    assert_eq!(column.column_type(), decimal_type(11, 1));
    assert_eq!(
        column.as_decimal75().expect("column should be decimal75"),
        expected
    );
}

fn new_mock_verification_builder() -> MockVerificationBuilder<TestScalar> {
    MockVerificationBuilder::new(
        Vec::new(),
        0,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

#[test]
fn we_can_evaluate_scaling_cast_expr_in_first_and_final_rounds() {
    let expr = int_to_decimal_expr();
    let alloc = Bump::new();
    let table = empty_table_with_rows(3);
    let expected = [
        TestScalar::from(70),
        TestScalar::from(70),
        TestScalar::from(70),
    ];

    let first_round_column = expr
        .first_round_evaluate(&alloc, &table, &[])
        .expect("first-round scaling cast should evaluate");
    assert_decimal_column(first_round_column, &expected);

    let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
    let final_round_column = expr
        .final_round_evaluate(&mut builder, &alloc, &table, &[])
        .expect("final-round scaling cast should evaluate");
    assert_decimal_column(final_round_column, &expected);
}

#[test]
fn we_can_verify_scaling_cast_expr() {
    let expr = int_to_decimal_expr();
    let mut builder = new_mock_verification_builder();
    let verified = expr
        .verifier_evaluate(&mut builder, &indexmap! {}, TestScalar::from(3), &[])
        .expect("scaling cast verifier evaluation should succeed");

    assert_eq!(verified, TestScalar::from(210));
}
