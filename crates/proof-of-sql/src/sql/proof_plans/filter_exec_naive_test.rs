use super::test_utility::{filter, table_exec};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::{bigint, owned_table},
            table_utility::{borrowed_bigint, table},
            ColumnField, ColumnType, TableRef, TableTestAccessor, TestAccessor,
        },
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::test_utility::{aliased_plan, column, const_int128, equal},
    },
};
use bumpalo::Bump;

#[test]
fn we_can_filter_data_with_naive_evaluation_proof() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
    ]);
    let table_ref = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(table_ref.clone(), data, 0);

    let input = table_exec(
        table_ref.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );
    let plan = filter(
        vec![aliased_plan(column(&table_ref, "b", &accessor), "b")],
        input,
        equal(column(&table_ref, "a", &accessor), const_int128(5_i128)),
    );

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let result = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    assert_eq!(result, owned_table([bigint("b", [3_i64, 5])]));
}
