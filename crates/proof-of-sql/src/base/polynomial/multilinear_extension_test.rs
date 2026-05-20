use super::MultilinearExtension;
use crate::base::{
    database::Column,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, Scalar},
};
use bumpalo::Bump;

fn exercise_column_methods(column: Column<'_, TestScalar>, expected_values: &[TestScalar]) {
    let evaluation_vec: Vec<TestScalar> = (1..=expected_values.len())
        .map(|value| TestScalar::from(value as u64))
        .collect();
    let expected_inner_product = expected_values
        .iter()
        .zip(&evaluation_vec)
        .map(|(&value, &eval)| value * eval)
        .sum();
    assert_eq!(
        column.inner_product(&evaluation_vec),
        expected_inner_product
    );

    let mut res = vec![TestScalar::from(3); expected_values.len()];
    column.mul_add(&mut res, &TestScalar::from(2));
    assert_eq!(
        res,
        expected_values
            .iter()
            .map(|&value| TestScalar::from(3) + TestScalar::from(2) * value)
            .collect::<Vec<_>>()
    );

    let mut expected_term = expected_values.to_vec();
    expected_term.resize(8, TestScalar::ZERO);
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&column, 3),
        expected_term
    );

    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&column).1,
        expected_values.len()
    );
}

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
fn we_can_use_multilinear_extension_methods_for_i64_array() {
    let array = [2_i64, 3, 4, 5];
    let evaluation_vec: Vec<TestScalar> = vec![101.into(), 102.into(), 103.into(), 104.into()];
    assert_eq!(
        (&array).inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104).into()
    );
    let mut res = evaluation_vec.clone();
    (&array).mul_add(&mut res, &10.into());
    assert_eq!(res, vec![121.into(), 132.into(), 143.into(), 154.into()]);
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&&array, 2),
        vec![2.into(), 3.into(), 4.into(), 5.into()]
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_all_column_variants() {
    let bool_values = [true, false, true];
    exercise_column_methods(
        Column::Boolean(&bool_values),
        &[TestScalar::ONE, TestScalar::ZERO, TestScalar::ONE],
    );

    let u8_values = [2_u8, 3, 4];
    exercise_column_methods(
        Column::Uint8(&u8_values),
        &[
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
        ],
    );

    let i8_values = [-2_i8, 0, 4];
    exercise_column_methods(
        Column::TinyInt(&i8_values),
        &[
            TestScalar::from(-2_i64),
            TestScalar::ZERO,
            TestScalar::from(4),
        ],
    );

    let i16_values = [-200_i16, 0, 400];
    exercise_column_methods(
        Column::SmallInt(&i16_values),
        &[
            TestScalar::from(-200_i64),
            TestScalar::ZERO,
            TestScalar::from(400),
        ],
    );

    let i32_values = [-20_000_i32, 0, 40_000];
    exercise_column_methods(
        Column::Int(&i32_values),
        &[
            TestScalar::from(-20_000_i64),
            TestScalar::ZERO,
            TestScalar::from(40_000),
        ],
    );

    let i128_values = [-20_000_000_000_i128, 0, 40_000_000_000];
    exercise_column_methods(
        Column::Int128(&i128_values),
        &[
            TestScalar::from(-20_000_000_000_i64),
            TestScalar::ZERO,
            TestScalar::from(40_000_000_000_i64),
        ],
    );

    let scalar_values = [
        TestScalar::from(5),
        TestScalar::from(8),
        TestScalar::from(13),
    ];
    exercise_column_methods(Column::Scalar(&scalar_values), &scalar_values);
    exercise_column_methods(
        Column::Decimal75(Precision::new(20).unwrap(), 4, &scalar_values),
        &scalar_values,
    );

    let string_values = ["a", "b", "c"];
    let string_scalars = [
        TestScalar::from(17),
        TestScalar::from(19),
        TestScalar::from(23),
    ];
    exercise_column_methods(
        Column::VarChar((&string_values, &string_scalars)),
        &string_scalars,
    );

    let binary_values: [&[u8]; 3] = [b"a".as_slice(), b"b".as_slice(), b"c".as_slice()];
    let binary_scalars = [
        TestScalar::from(29),
        TestScalar::from(31),
        TestScalar::from(37),
    ];
    exercise_column_methods(
        Column::VarBinary((&binary_values, &binary_scalars)),
        &binary_scalars,
    );

    let timestamp_values = [-3_i64, 0, 9];
    exercise_column_methods(
        Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &timestamp_values,
        ),
        &[
            TestScalar::from(-3_i64),
            TestScalar::ZERO,
            TestScalar::from(9),
        ],
    );
}

#[test]
fn we_can_evaluate_multilinear_extension_at_a_point() {
    let values = [2_i64, 4, 6, 8];
    let point = [TestScalar::from(3), TestScalar::from(5)];
    let one_minus_x = TestScalar::ONE - point[0];
    let one_minus_y = TestScalar::ONE - point[1];
    let expected = TestScalar::from(2) * one_minus_x * one_minus_y
        + TestScalar::from(4) * point[0] * one_minus_y
        + TestScalar::from(6) * one_minus_x * point[1]
        + TestScalar::from(8) * point[0] * point[1];

    assert_eq!((&values).evaluate_at_point(&point), expected);
}
