#[cfg(feature = "arrow")]
use super::ProvableQueryResult;
use super::{FinalRoundBuilder, SumcheckSubpolynomialType};
#[cfg(feature = "arrow")]
use crate::base::database::{Column, ColumnField, ColumnType};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{naive_commitment::NaiveCommitment, Commitment, CommittableColumn},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
};
#[cfg(feature = "arrow")]
use alloc::sync::Arc;
use alloc::{boxed::Box, collections::VecDeque};
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
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 0_usize;
    let commitments: Vec<NaiveCommitment> =
        builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        NaiveCommitment::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )
    );
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 123_usize;
    let commitments: Vec<NaiveCommitment> =
        builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        NaiveCommitment::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )
    );
}

#[test]
fn we_can_evaluate_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let evaluation_vec = [
        Curve25519Scalar::from(100u64),
        Curve25519Scalar::from(10u64),
    ];
    let evals = builder.evaluate_pcs_proof_mles(&evaluation_vec);
    let expected_evals = [
        Curve25519Scalar::from(120u64),
        Curve25519Scalar::from(1200u64),
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_track_builder_metadata_and_sumcheck_terms() {
    let mle = [1_i64, 2];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(3, VecDeque::new());
    assert_eq!(builder.num_sumcheck_variables(), 3);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
    assert!(builder.pcs_proof_mles().is_empty());
    assert!(builder.bit_distributions().is_empty());

    let bit_distribution = BitDistribution::new::<Curve25519Scalar, _>(&[0_i64, 1]);
    builder.produce_bit_distribution(bit_distribution.clone());
    assert_eq!(builder.bit_distributions(), &[bit_distribution]);

    builder.produce_anchored_mle(&mle);
    assert_eq!(builder.pcs_proof_mles().len(), 1);

    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(Curve25519Scalar::from(7), vec![Box::new(&mle[..])])],
    );
    assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    assert_eq!(
        builder.sumcheck_subpolynomials()[0].subpolynomial_type(),
        SumcheckSubpolynomialType::Identity
    );
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_form_the_provable_query_result() {
    let col1: Column<Curve25519Scalar> = Column::BigInt(&[11_i64, 12]);
    let col2: Column<Curve25519Scalar> = Column::BigInt(&[-3_i64, -4]);
    let res = ProvableQueryResult::new(2, &[col1, col2]);

    let column_fields = vec![
        ColumnField::new("a".into(), ColumnType::BigInt),
        ColumnField::new("b".into(), ColumnType::BigInt),
    ];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
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
            Curve25519Scalar::from(123),
            Curve25519Scalar::from(456),
            Curve25519Scalar::from(789),
        ]
        .into(),
    );
    assert_eq!(
        Curve25519Scalar::from(123),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(789),
        builder.consume_post_result_challenge()
    );
}
