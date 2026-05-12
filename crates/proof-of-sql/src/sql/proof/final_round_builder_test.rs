use super::FinalRoundBuilder;
#[cfg(feature = "arrow")]
use super::ProvableQueryResult;
#[cfg(feature = "arrow")]
use crate::base::database::{Column, ColumnField, ColumnType};
use crate::base::{
    commitment::{naive_commitment::NaiveCommitment, Commitment, CommittableColumn},
    scalar::test_scalar::TestScalar,
};
use alloc::collections::VecDeque;
#[cfg(feature = "arrow")]
use alloc::sync::Arc;
#[cfg(feature = "arrow")]
use arrow::{
    array::Int64Array,
    datatypes::{Field, Schema},
    record_batch::RecordBatch,
};

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<TestScalar>::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 0_usize;
    let commitments: Vec<NaiveCommitment> =
        builder.commit_intermediate_mles(offset_generators, &());
    let expected_commitments =
        NaiveCommitment::compute_commitments(&[CommittableColumn::from(&mle2[..])], 0, &());
    assert_eq!(commitments, expected_commitments);
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<TestScalar>::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 123_usize;
    let commitments: Vec<NaiveCommitment> =
        builder.commit_intermediate_mles(offset_generators, &());
    let expected_commitments = NaiveCommitment::compute_commitments(
        &[CommittableColumn::from(&mle2[..])],
        offset_generators,
        &(),
    );
    assert_eq!(commitments, expected_commitments);
}

#[test]
fn we_can_evaluate_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let evaluation_vec = [TestScalar::from(100u64), TestScalar::from(10u64)];
    let evals = builder.evaluate_pcs_proof_mles(&evaluation_vec);
    let expected_evals = [TestScalar::from(120u64), TestScalar::from(1200u64)];
    assert_eq!(evals, expected_evals);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_form_the_provable_query_result() {
    let col1: Column<TestScalar> = Column::BigInt(&[11_i64, 12]);
    let col2: Column<TestScalar> = Column::BigInt(&[-3_i64, -4]);
    let res = ProvableQueryResult::new(2, &[col1, col2]);

    let column_fields = vec![
        ColumnField::new("a".into(), ColumnType::BigInt),
        ColumnField::new("b".into(), ColumnType::BigInt),
    ];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));

    let expected_res = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![11, 12])),
            Arc::new(Int64Array::from(vec![-3, -4])),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_consume_post_result_challenges_in_proof_builder() {
    let mut builder = FinalRoundBuilder::new(
        0,
        [
            TestScalar::from(123),
            TestScalar::from(456),
            TestScalar::from(789),
        ]
        .into(),
    );
    assert_eq!(
        TestScalar::from(123),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        TestScalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        TestScalar::from(789),
        builder.consume_post_result_challenge()
    );
}
