use super::SumcheckRandomScalars;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn sumcheck_random_scalars_splits_multipliers_from_entrywise_point() {
    let scalars = [
        TestScalar::from(11u8),
        TestScalar::from(22u8),
        TestScalar::from(33u8),
        TestScalar::from(44u8),
        TestScalar::from(55u8),
    ];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 7, 2);

    assert_eq!(
        random_scalars.subpolynomial_multipliers,
        &[
            TestScalar::from(11u8),
            TestScalar::from(22u8),
            TestScalar::from(33u8)
        ]
    );
    assert_eq!(
        random_scalars.entrywise_point,
        &[TestScalar::from(44u8), TestScalar::from(55u8)]
    );
    assert_eq!(random_scalars.table_length, 7);
}

#[test]
fn sumcheck_random_scalars_uses_all_scalars_as_entrywise_point_when_there_are_no_multipliers() {
    let scalars = [TestScalar::from(3u8), TestScalar::from(5u8)];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 4, scalars.len());

    assert!(random_scalars.subpolynomial_multipliers.is_empty());
    assert_eq!(random_scalars.entrywise_point, scalars);
}

#[test]
fn compute_entrywise_multipliers_respects_truncated_table_length() {
    let scalars = [
        TestScalar::from(99u8),
        TestScalar::from(2u8),
        TestScalar::from(3u8),
    ];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 3, 2);

    let multipliers = random_scalars.compute_entrywise_multipliers();

    assert_eq!(
        multipliers,
        vec![
            (TestScalar::from(1u8) - TestScalar::from(2u8))
                * (TestScalar::from(1u8) - TestScalar::from(3u8)),
            TestScalar::from(2u8) * (TestScalar::from(1u8) - TestScalar::from(3u8)),
            (TestScalar::from(1u8) - TestScalar::from(2u8)) * TestScalar::from(3u8),
        ]
    );
}

#[test]
fn compute_entrywise_multipliers_can_return_an_empty_vector() {
    let scalars = [TestScalar::from(2u8), TestScalar::from(3u8)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 0, 2);

    assert!(random_scalars.compute_entrywise_multipliers().is_empty());
}
