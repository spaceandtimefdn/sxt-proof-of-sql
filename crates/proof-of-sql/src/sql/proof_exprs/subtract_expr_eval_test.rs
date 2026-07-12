use super::{DecimalProofExpr, DynProofExpr, ProofExpr, SubtractExpr};
use crate::{
    base::{
        database::{
            table_utility::{borrowed_int, borrowed_smallint, table},
            Column, ColumnRef, ColumnType, LiteralValue, TableRef,
        },
        map::{IndexMap, IndexSet},
        math::decimal::Precision,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
        AnalyzeError,
    },
};
use alloc::{boxed::Box, collections::VecDeque, string::ToString, vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;

fn column_expr(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
    DynProofExpr::new_column(ColumnRef::new(table_ref.clone(), name.into(), column_type))
}

fn test_subtract_expr(table_ref: &TableRef) -> SubtractExpr {
    SubtractExpr::try_new(
        Box::new(column_expr(table_ref, "lhs", ColumnType::Int)),
        Box::new(column_expr(table_ref, "rhs", ColumnType::SmallInt)),
    )
    .unwrap()
}

fn assert_decimal_column(
    column: Column<TestScalar>,
    precision: Precision,
    scale: i8,
    expected: &[TestScalar],
) {
    match column {
        Column::Decimal75(actual_precision, actual_scale, values) => {
            assert_eq!(actual_precision, precision);
            assert_eq!(actual_scale, scale);
            assert_eq!(values, expected);
        }
        other => panic!("expected Decimal75 column, got {other:?}"),
    }
}

#[test]
fn constructor_keeps_operands_and_rejects_non_numeric_types() {
    let lhs = DynProofExpr::new_literal(LiteralValue::Int(7));
    let rhs = DynProofExpr::new_literal(LiteralValue::BigInt(2));
    let expr = SubtractExpr::try_new(Box::new(lhs.clone()), Box::new(rhs)).unwrap();

    assert_eq!(expr.lhs().data_type(), ColumnType::Int);
    assert_eq!(expr.rhs().data_type(), ColumnType::BigInt);
    assert_eq!(
        expr.data_type(),
        ColumnType::Decimal75(Precision::new(20).unwrap(), 0)
    );
    assert_eq!(expr.precision(), Precision::new(20).unwrap());
    assert_eq!(expr.scale(), 0);

    let err = SubtractExpr::try_new(
        Box::new(lhs),
        Box::new(DynProofExpr::new_literal(LiteralValue::VarChar(
            "not numeric".to_string(),
        ))),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}

#[test]
fn first_round_evaluate_subtracts_columns_into_a_decimal_result() {
    let alloc = Bump::new();
    let data = table([
        borrowed_int("lhs", [10_i32, -5, 2], &alloc),
        borrowed_smallint("rhs", [3_i16, 7, -4], &alloc),
    ]);
    let table_ref = TableRef::new("sxt", "numbers");
    let expr = test_subtract_expr(&table_ref);
    let expected = [7, -12, 6].map(TestScalar::from);

    let result = expr.first_round_evaluate(&alloc, &data, &[]).unwrap();

    assert_decimal_column(result, expr.precision(), expr.scale(), &expected);
}

#[test]
fn final_round_evaluate_matches_first_round_without_extra_constraints() {
    let alloc = Bump::new();
    let data = table([
        borrowed_int("lhs", [20_i32, -8, 1, 0], &alloc),
        borrowed_smallint("rhs", [-2_i16, -8, 5, 12], &alloc),
    ]);
    let table_ref = TableRef::new("sxt", "numbers");
    let expr = test_subtract_expr(&table_ref);
    let expected = [22, 0, -4, -12].map(TestScalar::from);
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

    let result = expr
        .final_round_evaluate(&mut builder, &alloc, &data, &[])
        .unwrap();

    assert_decimal_column(result, expr.precision(), expr.scale(), &expected);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
    assert!(builder.pcs_proof_mles().is_empty());
}

#[test]
fn verifier_evaluate_subtracts_accessed_column_evaluations() {
    let table_ref = TableRef::new("sxt", "numbers");
    let lhs_ref = ColumnRef::new(table_ref.clone(), "lhs".into(), ColumnType::Int);
    let rhs_ref = ColumnRef::new(table_ref, "rhs".into(), ColumnType::SmallInt);
    let expr = SubtractExpr::try_new(
        Box::new(DynProofExpr::new_column(lhs_ref.clone())),
        Box::new(DynProofExpr::new_column(rhs_ref.clone())),
    )
    .unwrap();
    let mut accessor = IndexMap::default();
    accessor.insert(Ident::new("lhs"), TestScalar::from(81));
    accessor.insert(Ident::new("rhs"), TestScalar::from(13));
    let mut builder =
        MockVerificationBuilder::new(vec![], 1, vec![], vec![], vec![], vec![], vec![]);

    let value = expr
        .verifier_evaluate(&mut builder, &accessor, TestScalar::from(4), &[])
        .unwrap();

    assert_eq!(value, TestScalar::from(68));

    let mut references = IndexSet::default();
    expr.get_column_references(&mut references);
    assert_eq!(references.len(), 2);
    assert!(references.contains(&lhs_ref));
    assert!(references.contains(&rhs_ref));
}
