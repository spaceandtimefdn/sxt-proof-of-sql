use super::SumcheckRandomScalars;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn new_splits_subpolynomial_multipliers_from_entrywise_point() {
    let scalars = [
        TestScalar::from(11_u64),
        TestScalar::from(22_u64),
        TestScalar::from(2_u64),
        TestScalar::from(3_u64),
    ];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 3, 2);

    assert_eq!(random_scalars.subpolynomial_multipliers, &scalars[..2]);
    assert_eq!(random_scalars.entrywise_point, &scalars[2..]);
    assert_eq!(random_scalars.table_length, 3);
}

#[test]
fn compute_entrywise_multipliers_returns_truncated_evaluation_vector() {
    let scalars = [TestScalar::from(9_u64), TestScalar::from(2_u64)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 2, 1);

    let multipliers = random_scalars.compute_entrywise_multipliers();

    assert_eq!(
        multipliers,
        vec![TestScalar::from(-1_i64), TestScalar::from(2_u64)]
    );
}

#[test]
fn compute_entrywise_multipliers_allows_empty_tables() {
    let scalars = [TestScalar::from(5_u64)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 0, 1);

    assert!(random_scalars.compute_entrywise_multipliers().is_empty());
}
