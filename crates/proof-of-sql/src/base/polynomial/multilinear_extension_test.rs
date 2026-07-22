use super::MultilinearExtension;
use crate::base::{
    database::Column,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use num_traits::{One, Zero};

#[test]
fn allocated_slices_must_have_different_ids_even_when_one_is_empty() {
    let alloc = Bump::new();
    let foo = alloc.alloc_slice_fill_default(5) as &[TestScalar];
    let bar = alloc.alloc_slice_fill_default(0) as &[TestScalar];
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&foo),
        MultilinearExtension::<TestScalar>::id(&bar)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_slice() {
    let slice: &[i64] = &[2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_column() {
    let slice = Column::BigInt(&[2, 3, 4, 5, 6]);
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_vec() {
    let slice: &Vec<i64> = &vec![2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_evaluate_multilinear_extensions_at_a_point() {
    let values: &[i64] = &[2, 4, 6, 8];
    assert_eq!(
        values.evaluate_at_point(&[TestScalar::zero(), TestScalar::one()]),
        6.into()
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_all_column_variants() {
    let evaluation_vec: Vec<TestScalar> = vec![2.into(), 3.into(), 5.into()];
    let multiplier = TestScalar::from(7);

    let booleans = [true, false, true];
    let uint8s = [1_u8, 2, 3];
    let tiny_ints = [-1_i8, 2, -3];
    let small_ints = [-10_i16, 20, -30];
    let ints = [-100_i32, 200, -300];
    let big_ints = [-1000_i64, 2000, -3000];
    let int128s = [-10000_i128, 20000, -30000];
    let scalar_values = [11.into(), 12.into(), 13.into()];
    let decimal_values = [21.into(), 22.into(), 23.into()];
    let timestamp_values = [31_i64, 32, 33];
    let varchar_values = ["alpha", "beta", "gamma"];
    let varchar_scalars = [41.into(), 42.into(), 43.into()];
    let binary_a: &[u8] = &[1, 2];
    let binary_b: &[u8] = &[3, 4];
    let binary_c: &[u8] = &[5, 6];
    let binary_values = [binary_a, binary_b, binary_c];
    let binary_scalars = [51.into(), 52.into(), 53.into()];

    let cases = [
        (
            Column::Boolean(&booleans),
            vec![1.into(), 0.into(), 1.into()],
        ),
        (Column::Uint8(&uint8s), vec![1.into(), 2.into(), 3.into()]),
        (
            Column::TinyInt(&tiny_ints),
            vec![(-1).into(), 2.into(), (-3).into()],
        ),
        (
            Column::SmallInt(&small_ints),
            vec![(-10).into(), 20.into(), (-30).into()],
        ),
        (
            Column::Int(&ints),
            vec![(-100).into(), 200.into(), (-300).into()],
        ),
        (
            Column::BigInt(&big_ints),
            vec![(-1000).into(), 2000.into(), (-3000).into()],
        ),
        (
            Column::Int128(&int128s),
            vec![(-10000).into(), 20000.into(), (-30000).into()],
        ),
        (Column::Scalar(&scalar_values), scalar_values.to_vec()),
        (
            Column::Decimal75(Precision::new(75).unwrap(), 2, &decimal_values),
            decimal_values.to_vec(),
        ),
        (
            Column::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &timestamp_values,
            ),
            vec![31.into(), 32.into(), 33.into()],
        ),
        (
            Column::VarChar((&varchar_values, &varchar_scalars)),
            varchar_scalars.to_vec(),
        ),
        (
            Column::VarBinary((&binary_values, &binary_scalars)),
            binary_scalars.to_vec(),
        ),
    ];

    for (column, expected_values) in cases {
        assert_eq!(
            column.inner_product(&evaluation_vec),
            expected_inner_product(&expected_values, &evaluation_vec)
        );

        let mut res = evaluation_vec.clone();
        column.mul_add(&mut res, &multiplier);
        assert_eq!(
            res,
            expected_mul_add(&expected_values, &evaluation_vec, multiplier)
        );

        let mut expected_sumcheck_term = expected_values.clone();
        expected_sumcheck_term.push(TestScalar::zero());
        assert_eq!(column.to_sumcheck_term(2), expected_sumcheck_term);

        assert_eq!(
            MultilinearExtension::<TestScalar>::id(&column).1,
            column.len()
        );
    }
}

fn expected_inner_product(values: &[TestScalar], evaluation_vec: &[TestScalar]) -> TestScalar {
    values
        .iter()
        .zip(evaluation_vec)
        .fold(TestScalar::zero(), |acc, (value, evaluation)| {
            acc + *value * *evaluation
        })
}

fn expected_mul_add(
    values: &[TestScalar],
    base: &[TestScalar],
    multiplier: TestScalar,
) -> Vec<TestScalar> {
    values
        .iter()
        .zip(base)
        .map(|(value, base)| *base + *value * multiplier)
        .collect()
}
