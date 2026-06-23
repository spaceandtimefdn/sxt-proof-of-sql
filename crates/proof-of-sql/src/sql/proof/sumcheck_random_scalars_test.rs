use super::SumcheckRandomScalars;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn random_scalars_are_split_into_multipliers_and_entrywise_point() {
    let scalars = [
        TestScalar::from(2_u64),
        TestScalar::from(3_u64),
        TestScalar::from(5_u64),
        TestScalar::from(7_u64),
    ];

    let random_scalars = SumcheckRandomScalars::new(&scalars, 3, 2);

    assert_eq!(random_scalars.subpolynomial_multipliers, &scalars[..2]);
    assert_eq!(random_scalars.entrywise_point, &scalars[2..]);
    assert_eq!(random_scalars.table_length, 3);
}

#[test]
fn entrywise_multipliers_are_truncated_to_the_table_length() {
    let scalars = [
        TestScalar::from(11_u64),
        TestScalar::from(2_u64),
        TestScalar::from(3_u64),
    ];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 3, 2);

    let multipliers = random_scalars.compute_entrywise_multipliers();

    assert_eq!(
        multipliers,
        vec![
            (TestScalar::from(1_u64) - TestScalar::from(3_u64))
                * (TestScalar::from(1_u64) - TestScalar::from(2_u64)),
            (TestScalar::from(1_u64) - TestScalar::from(3_u64)) * TestScalar::from(2_u64),
            TestScalar::from(3_u64) * (TestScalar::from(1_u64) - TestScalar::from(2_u64)),
        ]
    );
}

#[test]
fn empty_table_has_no_entrywise_multipliers() {
    let scalars = [TestScalar::from(13_u64)];
    let random_scalars = SumcheckRandomScalars::new(&scalars, 0, 1);

    assert!(random_scalars.compute_entrywise_multipliers().is_empty());
}
