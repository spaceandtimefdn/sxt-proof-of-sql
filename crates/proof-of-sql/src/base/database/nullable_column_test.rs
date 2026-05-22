use super::{
    owned_table_utility::*, ColumnField, ColumnType, NullableOwnedColumn, OwnedColumn,
    OwnedTableTestAccessor, TableRef, ValidityError,
};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof, scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::test_utility::{add, aliased_plan, column},
        proof_plans::test_utility::{filter, table_exec},
    },
};
use alloc::vec;

#[test]
fn we_can_construct_nullable_bigint_with_canonical_nulls() {
    let nullable = NullableOwnedColumn::<TestScalar>::try_new_canonicalized(
        OwnedColumn::BigInt(vec![10, 999, 20, 777]),
        vec![true, false, true, false],
    )
    .unwrap();

    assert_eq!(nullable.values(), &OwnedColumn::BigInt(vec![10, 0, 20, 0]));
    assert_eq!(nullable.validity(), &[true, false, true, false]);
    assert_eq!(nullable.valid_len(), 2);
}

#[test]
fn we_reject_nullable_columns_with_noncanonical_nulls() {
    let result = NullableOwnedColumn::<TestScalar>::try_new(
        OwnedColumn::BigInt(vec![10, 999, 20]),
        vec![true, false, true],
    );

    assert!(matches!(
        result,
        Err(super::NullableColumnError::Validity {
            source: ValidityError::NonCanonicalNull { index: 1 }
        })
    ));
}

#[test]
fn we_can_add_nullable_bigint_to_nonnullable_bigint() {
    let nullable = NullableOwnedColumn::<TestScalar>::try_new_canonicalized(
        OwnedColumn::BigInt(vec![10, 999, 20, 777]),
        vec![true, false, true, false],
    )
    .unwrap();
    let rhs = OwnedColumn::BigInt(vec![1, 2, 3, 4]);

    let sum = nullable.try_element_wise_add_nonnullable(&rhs).unwrap();

    assert_eq!(sum.validity(), &[true, false, true, false]);
    assert_eq!(sum.values(), &OwnedColumn::BigInt(vec![11, 0, 23, 0]));
}

#[test]
fn we_can_add_two_nullable_bigints() {
    let left = NullableOwnedColumn::<TestScalar>::try_new_canonicalized(
        OwnedColumn::BigInt(vec![10, 999, 20, 777]),
        vec![true, false, true, false],
    )
    .unwrap();
    let right = NullableOwnedColumn::<TestScalar>::try_new_canonicalized(
        OwnedColumn::BigInt(vec![1, 2, 333, 4]),
        vec![true, true, false, false],
    )
    .unwrap();

    let sum = left.try_element_wise_add_nullable(&right).unwrap();

    assert_eq!(sum.validity(), &[true, false, false, false]);
    assert_eq!(sum.values(), &OwnedColumn::BigInt(vec![11, 0, 0, 0]));
}

#[test]
fn we_can_prove_a_query_over_nullable_bigint_with_validity_filter() {
    let nullable_score = NullableOwnedColumn::<TestScalar>::try_new_canonicalized(
        OwnedColumn::BigInt(vec![10, 999, 16, 777, 22]),
        vec![true, false, true, false, true],
    )
    .unwrap();
    let score_plus_bonus = nullable_score
        .try_element_wise_add_nonnullable(&OwnedColumn::BigInt(vec![1, 2, 3, 4, 5]))
        .unwrap();
    assert_eq!(
        score_plus_bonus.valid_values().unwrap(),
        OwnedColumn::BigInt(vec![11, 19, 27])
    );

    let table = owned_table([
        ("score".into(), nullable_score.values().clone()),
        ("score_valid".into(), nullable_score.validity_column()),
        ("bonus".into(), OwnedColumn::BigInt(vec![1, 2, 3, 4, 5])),
    ]);
    let table_ref = TableRef::new("sxt", "nullable_scores");
    let accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        table_ref.clone(),
        table,
        0,
        (),
    );
    let plan = filter(
        vec![aliased_plan(
            add(
                column(&table_ref, "score", &accessor),
                column(&table_ref, "bonus", &accessor),
            ),
            "score_plus_bonus",
        )],
        table_exec(
            table_ref.clone(),
            vec![
                ColumnField::new_nullable("score".into(), ColumnType::BigInt),
                ColumnField::new("score_valid".into(), ColumnType::Boolean),
                ColumnField::new("bonus".into(), ColumnType::BigInt),
            ],
        ),
        column(&table_ref, "score_valid", &accessor),
    );

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let result = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    assert_eq!(
        result,
        owned_table([decimal75("score_plus_bonus", 20, 0, [11_i64, 19, 27])])
    );
}

#[test]
fn nullable_schema_fields_preserve_nullability() {
    let field = ColumnField::new_nullable("score".into(), ColumnType::BigInt);

    assert_eq!(field.name().value, "score");
    assert_eq!(field.data_type(), ColumnType::BigInt);
    assert!(field.is_nullable());

    let field = field.with_nullable(false);
    assert!(!field.is_nullable());
}

#[test]
fn nonnullable_schema_fields_remain_the_default() {
    let field = ColumnField::new("score".into(), ColumnType::BigInt);

    assert!(!field.is_nullable());
}
