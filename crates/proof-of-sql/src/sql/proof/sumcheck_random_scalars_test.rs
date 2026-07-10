use super::SumcheckRandomScalars;
use crate::base::scalar::test_scalar::TestScalar;
use alloc::vec;
use num_traits::One;

#[test]
fn we_split_sumcheck_random_scalars_into_multipliers_and_entrywise_point() {
    let scalars = vec![
        TestScalar::from(11u64),
        TestScalar::from(12u64),
        TestScalar::from(13u64),
        TestScalar::from(3u64),
        TestScalar::from(4u64),
    ];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 7, 2);

    assert_eq!(random_scalars.subpolynomial_multipliers, &scalars[..3]);
    assert_eq!(random_scalars.entrywise_point, &scalars[3..]);
    assert_eq!(random_scalars.table_length, 7);
}

#[test]
fn we_compute_entrywise_multipliers_from_the_entrywise_point() {
    let three = TestScalar::from(3u64);
    let four = TestScalar::from(4u64);
    let scalars = vec![TestScalar::from(11u64), three, four];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 3, 2);

    let expected_multipliers = vec![
        (TestScalar::one() - four) * (TestScalar::one() - three),
        (TestScalar::one() - four) * three,
        four * (TestScalar::one() - three),
    ];

    assert_eq!(
        random_scalars.compute_entrywise_multipliers(),
        expected_multipliers
    );
}

#[test]
fn we_compute_entrywise_multipliers_for_an_empty_entrywise_point() {
    let scalars = vec![TestScalar::from(11u64), TestScalar::from(12u64)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 1, 0);

    assert_eq!(
        random_scalars.compute_entrywise_multipliers(),
        vec![TestScalar::one()]
    );
}
