use super::SumcheckRandomScalars;
use crate::base::scalar::test_scalar::TestScalar;
use num_traits::One;

#[test]
fn new_splits_subpolynomial_multipliers_from_entrywise_point() {
    let scalars = [
        TestScalar::from(10_u64),
        TestScalar::from(11_u64),
        TestScalar::from(2_u64),
        TestScalar::from(3_u64),
    ];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 4, 2);

    assert_eq!(random_scalars.subpolynomial_multipliers, &scalars[..2]);
    assert_eq!(random_scalars.entrywise_point, &scalars[2..]);
    assert_eq!(random_scalars.table_length, 4);
}

#[test]
fn new_allows_no_subpolynomial_multipliers() {
    let scalars = [TestScalar::from(2_u64), TestScalar::from(3_u64)];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 4, 2);

    assert!(random_scalars.subpolynomial_multipliers.is_empty());
    assert_eq!(random_scalars.entrywise_point, scalars);
}

#[test]
fn compute_entrywise_multipliers_matches_two_variable_evaluation_vector() {
    let one = TestScalar::one();
    let x_0 = TestScalar::from(2_u64);
    let x_1 = TestScalar::from(3_u64);
    let scalars = [TestScalar::from(10_u64), x_0, x_1];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 4, 2);

    let entrywise_multipliers = random_scalars.compute_entrywise_multipliers();

    assert_eq!(
        entrywise_multipliers,
        vec![
            (one - x_1) * (one - x_0),
            (one - x_1) * x_0,
            x_1 * (one - x_0),
            x_1 * x_0,
        ]
    );
}

#[test]
fn compute_entrywise_multipliers_handles_empty_table() {
    let scalars = [TestScalar::from(2_u64), TestScalar::from(3_u64)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 0, 2);

    let entrywise_multipliers = random_scalars.compute_entrywise_multipliers();

    assert!(entrywise_multipliers.is_empty());
}
