use super::{
    DenseProvableResultColumn, MultilinearExtensionImpl, ProofBuilder, SumcheckRandomScalars,
    SumcheckSubpolynomial,
};
use crate::{
    base::{
        database::{ColumnField, ColumnType},
        polynomial::CompositePolynomial,
        scalar::{compute_commitment_for_testing, ArkScalar},
    },
    sql::proof::{
        compute_evaluation_vector, Indexes, MultilinearExtension, ResultBuilder,
        SumcheckSubpolynomialType,
    },
};
use arrow::{
    array::Int64Array,
    datatypes::{Field, Schema},
    record_batch::RecordBatch,
};
use num_traits::{One, Zero};
use std::sync::Arc;

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let offset_generators = 0_usize;
    let commitments = builder.commit_intermediate_mles(offset_generators);
    assert_eq!(
        commitments,
        [compute_commitment_for_testing(&mle2, offset_generators).compress()]
    );
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let offset_generators = 123_usize;
    let commitments = builder.commit_intermediate_mles(offset_generators);
    assert_eq!(
        commitments,
        [compute_commitment_for_testing(&mle2, offset_generators).compress()]
    );
}

#[test]
fn we_can_evaluate_pre_result_mles() {
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let evaluation_vec = [ArkScalar::from(100u64), ArkScalar::from(10u64)];
    let evals = builder.evaluate_pre_result_mles(&evaluation_vec);
    let expected_evals = [ArkScalar::from(120u64), ArkScalar::from(1200u64)];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_form_an_aggregated_sumcheck_polynomial() {
    let mle1 = [1, 2, -1];
    let mle2 = [10u32, 20, 100, 30];
    let mle3 = [2000u32, 3000, 5000, 7000];
    let mut builder = ProofBuilder::new(4, 2, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    builder.produce_intermediate_mle(&mle3);

    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(
        SumcheckSubpolynomialType::Identity,
        vec![(
            -ArkScalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(&mle1))],
        )],
    ));
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(
        SumcheckSubpolynomialType::Identity,
        vec![(
            -ArkScalar::from(10u64),
            vec![Box::new(MultilinearExtensionImpl::new(&mle2))],
        )],
    ));
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(
        SumcheckSubpolynomialType::ZeroSum,
        vec![(
            ArkScalar::from(9876u64),
            vec![Box::new(MultilinearExtensionImpl::new(&mle3))],
        )],
    ));

    let multipliers = [
        ArkScalar::from(5u64),
        ArkScalar::from(2u64),
        ArkScalar::from(50u64),
        ArkScalar::from(25u64),
        ArkScalar::from(11u64),
    ];

    let mut evaluation_vector = vec![Zero::zero(); 4];
    compute_evaluation_vector(&mut evaluation_vector, &multipliers[..2]);

    let poly = builder.make_sumcheck_polynomial(&SumcheckRandomScalars::new(&multipliers, 4, 2));
    let mut expected_poly = CompositePolynomial::new(2);
    let fr = MultilinearExtensionImpl::new(&evaluation_vector).to_sumcheck_term(2);
    expected_poly.add_product(
        [
            fr.clone(),
            MultilinearExtensionImpl::new(&mle1).to_sumcheck_term(2),
        ],
        -ArkScalar::from(1u64) * multipliers[2],
    );
    expected_poly.add_product(
        [fr, MultilinearExtensionImpl::new(&mle2).to_sumcheck_term(2)],
        -ArkScalar::from(10u64) * multipliers[3],
    );
    expected_poly.add_product(
        [MultilinearExtensionImpl::new(&mle3).to_sumcheck_term(2)],
        ArkScalar::from(9876u64) * multipliers[4],
    );
    let random_point = [ArkScalar::from(123u64), ArkScalar::from(101112u64)];
    let eval = poly.evaluate(&random_point);
    let expected_eval = expected_poly.evaluate(&random_point);
    assert_eq!(eval, expected_eval);
}

#[test]
fn we_can_form_the_provable_query_result() {
    let result_indexes = Indexes::Sparse(vec![1, 2]);
    let col1 = [10, 11, 12];
    let col2 = [-2, -3, -4];
    let mut builder = ResultBuilder::new(3);
    builder.set_result_indexes(result_indexes);
    builder.produce_result_column(Box::new(DenseProvableResultColumn::<i64>::new(&col1)));
    builder.produce_result_column(Box::new(DenseProvableResultColumn::<i64>::new(&col2)));

    let res = builder.make_provable_query_result();

    let column_fields = vec![
        ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
    ];
    let res = RecordBatch::try_from(res.into_owned_table(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(|v| v.into()).collect();
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
fn we_can_fold_pre_result_mles() {
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let multipliers = [ArkScalar::from(100u64), ArkScalar::from(2u64)];
    let z = builder.fold_pre_result_mles(&multipliers);
    let expected_z = [ArkScalar::from(120u64), ArkScalar::from(240u64)];
    assert_eq!(z, expected_z);
}

#[test]
fn we_can_consume_post_result_challenges_in_proof_builder() {
    let mut builder = ProofBuilder::new(
        0,
        0,
        vec![
            ArkScalar::from(123),
            ArkScalar::from(456),
            ArkScalar::from(789),
        ],
    );
    assert_eq!(
        ArkScalar::from(789),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        ArkScalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        ArkScalar::from(123),
        builder.consume_post_result_challenge()
    );
}
