use super::equals_expr::{
    final_round_evaluate_equals_zero, first_round_evaluate_equals_zero,
    verifier_evaluate_equals_zero,
};
use super::{test_utility::*, EqualsExpr, ProofExpr};
use crate::{
    base::{
        database::ColumnType,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::{
        proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
        AnalyzeError,
    },
};
use alloc::{collections::VecDeque, vec, vec::Vec};
use bumpalo::Bump;

#[test]
fn we_can_create_equals_expr_and_access_children_without_blitzar() {
    let expr = EqualsExpr::try_new(Box::new(const_bigint(1)), Box::new(const_bigint(2))).unwrap();

    assert_eq!(expr.data_type(), ColumnType::Boolean);
    assert_eq!(expr.lhs().data_type(), ColumnType::BigInt);
    assert_eq!(expr.rhs().data_type(), ColumnType::BigInt);
}

#[test]
fn equals_expr_rejects_mismatched_types_without_blitzar() {
    let err =
        EqualsExpr::try_new(Box::new(const_bigint(1)), Box::new(const_varchar("x"))).unwrap_err();

    assert!(matches!(err, AnalyzeError::DataTypeMismatch { .. }));
}

#[test]
fn we_can_evaluate_equals_zero_in_the_first_round_without_blitzar() {
    let alloc = Bump::new();
    let lhs = [
        TestScalar::ZERO,
        TestScalar::from(5),
        TestScalar::from(-3),
        TestScalar::ZERO,
    ];

    let selection = first_round_evaluate_equals_zero(lhs.len(), &alloc, &lhs);

    assert_eq!(selection, &[true, false, false, true]);
}

#[test]
fn we_can_evaluate_equals_zero_in_the_final_round_without_blitzar() {
    let alloc = Bump::new();
    let lhs = [
        TestScalar::ZERO,
        TestScalar::from(5),
        TestScalar::from(-3),
        TestScalar::ZERO,
    ];
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

    let selection = final_round_evaluate_equals_zero(lhs.len(), &mut builder, &alloc, &lhs);

    assert_eq!(selection, &[true, false, false, true]);
    assert_eq!(builder.pcs_proof_mles().len(), 2);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 2);
}

#[test]
fn verifier_equals_zero_records_identity_constraints_without_blitzar() {
    let mut builder = MockVerificationBuilder::new(
        Vec::new(),
        3,
        Vec::new(),
        vec![vec![TestScalar::ZERO, TestScalar::ONE]],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    let selection_eval =
        verifier_evaluate_equals_zero(&mut builder, TestScalar::ZERO, TestScalar::ONE).unwrap();

    assert_eq!(selection_eval, TestScalar::ONE);
    assert_eq!(builder.get_identity_results(), vec![vec![true, true]]);
}
